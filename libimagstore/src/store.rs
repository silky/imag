use std::collections::HashMap;
use std::fs::{File, remove_file};
use std::ops::Drop;
use std::path::PathBuf;
use std::result::Result as RResult;
use std::sync::Arc;
use std::sync::RwLock;
use std::collections::BTreeMap;
use std::io::{Seek, SeekFrom};
use std::convert::From;
use std::convert::Into;

use toml::{Table, Value};
use regex::Regex;

use error::{ParserErrorKind, ParserError};
use error::{StoreError, StoreErrorKind};
use storeid::{StoreId, StoreIdIterator};
use lazyfile::LazyFile;

/// The Result Type returned by any interaction with the store that could fail
pub type Result<T> = RResult<T, StoreError>;


#[derive(PartialEq)]
enum StoreEntryStatus {
    Present,
    Borrowed
}

/// A store entry, depending on the option type it is either borrowed currently
/// or not.
struct StoreEntry {
    id: StoreId,
    file: LazyFile,
    status: StoreEntryStatus,
}

impl StoreEntry {

    fn new(id: StoreId) -> StoreEntry {
        StoreEntry {
            id: id.clone(),
            file: LazyFile::Absent(id),
            status: StoreEntryStatus::Present,
        }
    }

    /// The entry is currently borrowed, meaning that some thread is currently
    /// mutating it
    fn is_borrowed(&self) -> bool {
        self.status == StoreEntryStatus::Borrowed
    }

    fn get_entry(&mut self) -> Result<Entry> {
        if !self.is_borrowed() {
            let file = self.file.get_file_mut();
            if let Err(err) = file {
                if err.err_type() == StoreErrorKind::FileNotFound {
                    Ok(Entry::new(self.id.clone()))
                } else {
                    Err(err)
                }
            } else {
                // TODO:
                let mut file = file.unwrap();
                let entry = Entry::from_file(self.id.clone(), &mut file);
                file.seek(SeekFrom::Start(0));
                entry
            }
        } else {
            return Err(StoreError::new(StoreErrorKind::EntryAlreadyBorrowed, None))
        }
    }

    fn write_entry(&mut self, entry: &Entry) -> Result<()> {
        if self.is_borrowed() {
            use std::io::Write;
            let file = try!(self.file.create_file());

            assert_eq!(self.id, entry.location);
            file.write_all(entry.to_str().as_bytes());
        }

        Ok(())
    }
}

/// The Store itself, through this object one can interact with IMAG's entries
pub struct Store {
    location: PathBuf,

    /**
     * Internal Path->File cache map
     *
     * Caches the files, so they remain flock()ed
     *
     * Could be optimized for a threadsafe HashMap
     */
    entries: Arc<RwLock<HashMap<StoreId, StoreEntry>>>,
}

impl Store {

    /// Create a new Store object
    pub fn new(location: PathBuf) -> Result<Store> {
        use std::fs::create_dir_all;

        debug!("Building new Store object");
        if !location.exists() {
            debug!("Creating store path");
            let c = create_dir_all(location.clone());
            if c.is_err() {
                debug!("Failed");
                return Err(StoreError::new(StoreErrorKind::StorePathCreate,
                                           Some(Box::new(c.err().unwrap()))));
            }
        } else {
            if location.is_file() {
                debug!("Store path exists as file");
                return Err(StoreError::new(StoreErrorKind::StorePathExists, None));
            }
        }

        debug!("Store building succeeded");
        Ok(Store {
            location: location,
            entries: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Creates the Entry at the given location (inside the entry)
    pub fn create<'a>(&'a self, id: StoreId) -> Result<FileLockEntry<'a>> {
        if !self.id_in_store(&id) {
            debug!("'{:?}' seems not to be in '{:?}'", id, self.location);
            return Err(StoreError::new(StoreErrorKind::StorePathOutsideStore, None));
        }

        let hsmap = self.entries.write();
        if hsmap.is_err() {
            return Err(StoreError::new(StoreErrorKind::LockPoisoned, None))
        }
        let mut hsmap = hsmap.unwrap();
        if hsmap.contains_key(&id) {
            return Err(StoreError::new(StoreErrorKind::EntryAlreadyExists, None))
        }
        hsmap.insert(id.clone(), {
            let mut se = StoreEntry::new(id.clone());
            se.status = StoreEntryStatus::Borrowed;
            se
        });
        Ok(FileLockEntry::new(self, Entry::new(id.clone()), id))
    }

    /// Borrow a given Entry. When the `FileLockEntry` is either `update`d or
    /// dropped, the new Entry is written to disk
    pub fn retrieve<'a>(&'a self, id: StoreId) -> Result<FileLockEntry<'a>> {
        if !self.id_in_store(&id) {
            debug!("'{:?}' seems not to be in '{:?}'", id, self.location);
            return Err(StoreError::new(StoreErrorKind::StorePathOutsideStore, None));
        }

        self.entries
            .write()
            .map_err(|_| StoreError::new(StoreErrorKind::LockPoisoned, None))
            .and_then(|mut es| {
                let mut se = es.entry(id.clone()).or_insert_with(|| StoreEntry::new(id.clone()));
                let entry = se.get_entry();
                se.status = StoreEntryStatus::Borrowed;
                entry
            })
            .map(|e| FileLockEntry::new(self, e, id))
   }

    /// Iterate over all StoreIds for one module name
    pub fn retrieve_for_module(&self, mod_name: &str) -> StoreIdIterator {
        unimplemented!();
    }

    /// Return the `FileLockEntry` and write to disk
    pub fn update<'a>(&'a self, entry: FileLockEntry<'a>) -> Result<()> {
        self._update(&entry)
    }

    /// Internal method to write to the filesystem store.
    ///
    /// # Assumptions
    /// This method assumes that entry is dropped _right after_ the call, hence
    /// it is not public.
    fn _update<'a>(&'a self, entry: &FileLockEntry<'a>) -> Result<()> {
        let hsmap = self.entries.write();
        if hsmap.is_err() {
            return Err(StoreError::new(StoreErrorKind::LockPoisoned, None))
        }
        let mut hsmap = hsmap.unwrap();
        let mut se = try!(hsmap.get_mut(&entry.key)
              .ok_or(StoreError::new(StoreErrorKind::IdNotFound, None)));

        assert!(se.is_borrowed(), "Tried to update a non borrowed entry.");

        debug!("Verifying Entry");
        try!(entry.entry.verify());

        debug!("Writing Entry");
        try!(se.write_entry(&entry.entry));
        se.status = StoreEntryStatus::Present;

        Ok(())
    }

    /// Retrieve a copy of a given entry, this cannot be used to mutate
    /// the one on disk
    pub fn retrieve_copy(&self, id: StoreId) -> Result<Entry> {
        let mut entries_lock = self.entries.write();
        if entries_lock.is_err() {
            return Err(StoreError::new(StoreErrorKind::LockPoisoned, None))
        }

        let mut entries = entries_lock.unwrap();

        // if the entry is currently modified by the user, we cannot drop it
        if entries.get(&id).map(|e| e.is_borrowed()).unwrap_or(false) {
            return Err(StoreError::new(StoreErrorKind::IdLocked, None));
        }

        StoreEntry::new(id).get_entry()
    }

    /// Delete an entry
    pub fn delete(&self, id: StoreId) -> Result<()> {
        if !self.id_in_store(&id) {
            debug!("'{:?}' seems not to be in '{:?}'", id, self.location);
            return Err(StoreError::new(StoreErrorKind::StorePathOutsideStore, None));
        }

        let mut entries_lock = self.entries.write();
        if entries_lock.is_err() {
            return Err(StoreError::new(StoreErrorKind::LockPoisoned, None))
        }

        let mut entries = entries_lock.unwrap();

        // if the entry is currently modified by the user, we cannot drop it
        if entries.get(&id).map(|e| e.is_borrowed()).unwrap_or(false) {
            return Err(StoreError::new(StoreErrorKind::IdLocked, None));
        }

        // remove the entry first, then the file
        entries.remove(&id);
        remove_file(&id).map_err(|e| StoreError::new(StoreErrorKind::FileError, Some(Box::new(e))))
    }

    fn id_in_store(&self, path: &StoreId) -> bool {
        path.canonicalize()
            .map(|can| {
                can.starts_with(&self.location)
            })
            .unwrap_or(path.starts_with(&self.location))
            // we return false, as fs::canonicalize() returns an Err(..) on filesystem errors
    }

    /// Gets the path where this store is on the disk
    pub fn path(&self) -> &PathBuf {
        &self.location
    }
}

impl Drop for Store {

    /**
     * Unlock all files on drop
     *
     * TODO: Unlock them
     */
    fn drop(&mut self) {
        debug!("Dropping store");
    }

}

/// A struct that allows you to borrow an Entry
pub struct FileLockEntry<'a> {
    store: &'a Store,
    entry: Entry,
    key: StoreId,
}

impl<'a> FileLockEntry<'a, > {
    fn new(store: &'a Store, entry: Entry, key: StoreId) -> FileLockEntry<'a> {
        FileLockEntry {
            store: store,
            entry: entry,
            key: key,
        }
    }
}

impl<'a> ::std::ops::Deref for FileLockEntry<'a> {
    type Target = Entry;

    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl<'a> ::std::ops::DerefMut for FileLockEntry<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry
    }
}

impl<'a> Drop for FileLockEntry<'a> {
    /// This will silently ignore errors, use `Store::update` if you want to catch the errors
    fn drop(&mut self) {
        let _ = self.store._update(self);
    }
}

/**
 * EntryContent type
 */
pub type EntryContent = String;

/**
 * EntryHeader
 *
 * This is basically a wrapper around toml::Table which provides convenience to the user of the
 * librray.
 */
#[derive(Debug, Clone)]
pub struct EntryHeader {
    header: Value,
}

pub type EntryResult<V> = RResult<V, ParserError>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Key(String),
    Index(usize),
}

/**
 * Wrapper type around file header (TOML) object
 */
impl EntryHeader {

    pub fn new() -> EntryHeader {
        EntryHeader {
            header: build_default_header()
        }
    }

    fn from_table(t: Table) -> EntryHeader {
        EntryHeader {
            header: Value::Table(t)
        }
    }

    pub fn parse(s: &str) -> EntryResult<EntryHeader> {
        use toml::Parser;

        let mut parser = Parser::new(s);
        parser.parse()
            .ok_or(ParserError::new(ParserErrorKind::TOMLParserErrors, None))
            .and_then(verify_header_consistency)
            .map(EntryHeader::from_table)
    }

    pub fn verify(&self) -> Result<()> {
        match &self.header {
            &Value::Table(ref t) => verify_header(&t),
            _ => Err(StoreError::new(StoreErrorKind::HeaderTypeFailure, None)),
        }
    }

    /**
     * Insert a header field by a string-spec
     *
     * ```ignore
     *  insert("something.in.a.field", Boolean(true));
     * ```
     *
     * If an array field was accessed which is _out of bounds_ of the array available, the element
     * is appended to the array.
     *
     * Inserts a Boolean in the section "something" -> "in" -> "a" -> "field"
     * A JSON equivalent would be
     *
     *  {
     *      something: {
     *          in: {
     *              a: {
     *                  field: true
     *              }
     *          }
     *      }
     *  }
     *
     * Returns true if header field was set, false if there is already a value
     */
    pub fn insert(&mut self, spec: &str, v: Value) -> Result<bool> {
        self.insert_with_sep(spec, '.', v)
    }

    pub fn insert_with_sep(&mut self, spec: &str, sep: char, v: Value) -> Result<bool> {
        let tokens = EntryHeader::tokenize(spec, sep);
        if tokens.is_err() { // return parser error if any
            return tokens.map(|_| false);
        }
        let tokens = tokens.unwrap();

        let destination = tokens.iter().last();
        if destination.is_none() {
            return Err(StoreError::new(StoreErrorKind::HeaderPathSyntaxError, None));
        }
        let destination = destination.unwrap();

        let path_to_dest = tokens[..(tokens.len() - 1)].into(); // N - 1 tokens
        let mut value = EntryHeader::walk_header(&mut self.header, path_to_dest); // walk N-1 tokens
        if value.is_err() {
            return value.map(|_| false);
        }
        let mut value = value.unwrap();

        // There is already an value at this place
        if EntryHeader::extract(value, destination).is_ok() {
            return Ok(false);
        }

        match destination {
            &Token::Key(ref s) => { // if the destination shall be an map key
                match value {
                    /*
                     * Put it in there if we have a map
                     */
                    &mut Value::Table(ref mut t) => {
                        t.insert(s.clone(), v);
                    }

                    /*
                     * Fail if there is no map here
                     */
                    _ => return Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None)),
                }
            },

            &Token::Index(i) => { // if the destination shall be an array
                match value {

                    /*
                     * Put it in there if we have an array
                     */
                    &mut Value::Array(ref mut a) => {
                        a.push(v); // push to the end of the array

                        // if the index is inside the array, we swap-remove the element at this
                        // index
                        if a.len() < i {
                            a.swap_remove(i);
                        }
                    },

                    /*
                     * Fail if there is no array here
                     */
                    _ => return Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None)),
                }
            },
        }

        Ok(true)
    }

    /**
     * Set a header field by a string-spec
     *
     * ```ignore
     *  set("something.in.a.field", Boolean(true));
     * ```
     *
     * Sets a Boolean in the section "something" -> "in" -> "a" -> "field"
     * A JSON equivalent would be
     *
     *  {
     *      something: {
     *          in: {
     *              a: {
     *                  field: true
     *              }
     *          }
     *      }
     *  }
     *
     * If there is already a value at this place, this value will be overridden and the old value
     * will be returned
     */
    pub fn set(&mut self, spec: &str, v: Value) -> Result<Option<Value>> {
        self.set_with_sep(spec, '.', v)
    }

    pub fn set_with_sep(&mut self, spec: &str, sep: char, v: Value) -> Result<Option<Value>> {
        let tokens = EntryHeader::tokenize(spec, sep);
        if tokens.is_err() { // return parser error if any
            return Err(tokens.err().unwrap());
        }
        let tokens = tokens.unwrap();
        trace!("tokens = {:?}", tokens);

        let destination = tokens.iter().last();
        if destination.is_none() {
            return Err(StoreError::new(StoreErrorKind::HeaderPathSyntaxError, None));
        }
        let destination = destination.unwrap();
        trace!("destination = {:?}", destination);

        let path_to_dest = tokens[..(tokens.len() - 1)].into(); // N - 1 tokens
        let mut value = EntryHeader::walk_header(&mut self.header, path_to_dest); // walk N-1 tokens
        if value.is_err() {
            return Err(value.err().unwrap());
        }
        let mut value = value.unwrap();
        trace!("walked value = {:?}", value);

        match destination {
            &Token::Key(ref s) => { // if the destination shall be an map key->value
                match value {
                    /*
                     * Put it in there if we have a map
                     */
                    &mut Value::Table(ref mut t) => {
                        trace!("Matched Key->Table");
                        return Ok(t.insert(s.clone(), v));
                    }

                    /*
                     * Fail if there is no map here
                     */
                    _ => {
                        trace!("Matched Key->NON-Table");
                        return Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None));
                    }
                }
            },

            &Token::Index(i) => { // if the destination shall be an array
                match value {

                    /*
                     * Put it in there if we have an array
                     */
                    &mut Value::Array(ref mut a) => {
                        trace!("Matched Index->Array");
                        a.push(v); // push to the end of the array

                        // if the index is inside the array, we swap-remove the element at this
                        // index
                        if a.len() > i {
                            trace!("Swap-Removing in Array {:?}[{:?}] <- {:?}", a, i, a[a.len()-1]);
                            return Ok(Some(a.swap_remove(i)));
                        }

                        trace!("Appended");
                        return Ok(None);
                    },

                    /*
                     * Fail if there is no array here
                     */
                    _ => {
                        trace!("Matched Index->NON-Array");
                        return Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None));
                    },
                }
            },
        }

        Ok(None)
    }

    /**
     * Read a header field by a string-spec
     *
     * ```ignore
     *  let value = read("something.in.a.field");
     * ```
     *
     * Reads a Value in the section "something" -> "in" -> "a" -> "field"
     * A JSON equivalent would be
     *
     *  {
     *      something: {
     *          in: {
     *              a: {
     *                  field: true
     *              }
     *          }
     *      }
     *  }
     *
     * If there is no a value at this place, None will be returned. This also holds true for Arrays
     * which are accessed at an index which is not yet there, even if the accessed index is much
     * larger than the array length.
     */
    pub fn read(&self, spec: &str) -> Result<Option<Value>> {
        self.read_with_sep(spec, '.')
    }

    pub fn read_with_sep(&self, spec: &str, splitchr: char) -> Result<Option<Value>> {
        let tokens = EntryHeader::tokenize(spec, splitchr);
        if tokens.is_err() { // return parser error if any
            return Err(tokens.err().unwrap());
        }
        let tokens = tokens.unwrap();

        let mut header_clone = self.header.clone(); // we clone as READing is simpler this way
        let mut value = EntryHeader::walk_header(&mut header_clone, tokens); // walk N-1 tokens
        if value.is_err() {
            let e = value.err().unwrap();
            match e.err_type() {
                // We cannot find the header key, as there is no path to it
                StoreErrorKind::HeaderKeyNotFound => return Ok(None),
                _ => return Err(e),
            }
            return Err(e);
        }
        Ok(Some(value.unwrap().clone()))
    }

    pub fn delete(&mut self, spec: &str) -> Result<Option<Value>> {
        let tokens = EntryHeader::tokenize(spec, '.');
        if tokens.is_err() { // return parser error if any
            return Err(tokens.err().unwrap());
        }
        let tokens = tokens.unwrap();

        let destination = tokens.iter().last();
        if destination.is_none() {
            return Err(StoreError::new(StoreErrorKind::HeaderPathSyntaxError, None));
        }
        let destination = destination.unwrap();
        trace!("destination = {:?}", destination);

        let path_to_dest = tokens[..(tokens.len() - 1)].into(); // N - 1 tokens
        let mut value = EntryHeader::walk_header(&mut self.header, path_to_dest); // walk N-1 tokens
        if value.is_err() {
            return Err(value.err().unwrap());
        }
        let mut value = value.unwrap();
        trace!("walked value = {:?}", value);

        match destination {
            &Token::Key(ref s) => { // if the destination shall be an map key->value
                match value {
                    &mut Value::Table(ref mut t) => {
                        trace!("Matched Key->Table, removing {:?}", s);
                        return Ok(t.remove(s));
                    },
                    _ => {
                        trace!("Matched Key->NON-Table");
                        return Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None));
                    }
                }
            },

            &Token::Index(i) => { // if the destination shall be an array
                match value {
                    &mut Value::Array(ref mut a) => {
                        // if the index is inside the array, we swap-remove the element at this
                        // index
                        if a.len() > i {
                            trace!("Removing in Array {:?}[{:?}]", a, i);
                            return Ok(Some(a.remove(i)));
                        } else {
                            return Ok(None);
                        }
                    },
                    _ => {
                        trace!("Matched Index->NON-Array");
                        return Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None));
                    },
                }
            },
        }

        Ok(None)
    }

    fn tokenize(spec: &str, splitchr: char) -> Result<Vec<Token>> {
        use std::str::FromStr;

        spec.split(splitchr)
            .map(|s| {
                usize::from_str(s)
                    .map(Token::Index)
                    .or_else(|_| Ok(Token::Key(String::from(s))))
            })
            .collect()
    }

    fn walk_header(v: &mut Value, tokens: Vec<Token>) -> Result<&mut Value> {
        use std::vec::IntoIter;

        fn walk_iter<'a>(v: Result<&'a mut Value>, i: &mut IntoIter<Token>) -> Result<&'a mut Value> {
            let next = i.next();
            v.and_then(move |value| {
                if let Some(token) = next {
                    walk_iter(EntryHeader::extract(value, &token), i)
                } else {
                    Ok(value)
                }
            })
        }

        walk_iter(Ok(v), &mut tokens.into_iter())
    }

    fn extract_from_table<'a>(v: &'a mut Value, s: &String) -> Result<&'a mut Value> {
        match v {
            &mut Value::Table(ref mut t) => {
                t.get_mut(&s[..])
                    .ok_or(StoreError::new(StoreErrorKind::HeaderKeyNotFound, None))
            },
            _ => Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None)),
        }
    }

    fn extract_from_array(v: &mut Value, i: usize) -> Result<&mut Value> {
        match v {
            &mut Value::Array(ref mut a) => {
                if a.len() < i {
                    Err(StoreError::new(StoreErrorKind::HeaderKeyNotFound, None))
                } else {
                    Ok(&mut a[i])
                }
            },
            _ => Err(StoreError::new(StoreErrorKind::HeaderPathTypeFailure, None)),
        }
    }

    fn extract<'a>(v: &'a mut Value, token: &Token) -> Result<&'a mut Value> {
        match token {
            &Token::Key(ref s)  => EntryHeader::extract_from_table(v, s),
            &Token::Index(i)    => EntryHeader::extract_from_array(v, i),
        }
    }

}

impl Into<Table> for EntryHeader {

    fn into(self) -> Table {
        match self.header {
            Value::Table(t) => t,
            _ => panic!("EntryHeader is not a table!"),
        }
    }

}

impl From<Table> for EntryHeader {

    fn from(t: Table) -> EntryHeader {
        EntryHeader { header: Value::Table(t) }
    }

}

fn build_default_header() -> Value { // BTreeMap<String, Value>
    let mut m = BTreeMap::new();

    m.insert(String::from("imag"), {
        let mut imag_map = BTreeMap::<String, Value>::new();

        imag_map.insert(String::from("version"), Value::String(String::from(version!())));
        imag_map.insert(String::from("links"), Value::Array(vec![]));

        Value::Table(imag_map)
    });

    Value::Table(m)
}
fn verify_header(t: &Table) -> Result<()> {
    if !has_main_section(t) {
        Err(StoreError::from(ParserError::new(ParserErrorKind::MissingMainSection, None)))
    } else if !has_imag_version_in_main_section(t) {
        Err(StoreError::from(ParserError::new(ParserErrorKind::MissingVersionInfo, None)))
    } else if !has_only_tables(t) {
        debug!("Could not verify that it only has tables in its base table");
        Err(StoreError::from(ParserError::new(ParserErrorKind::NonTableInBaseTable, None)))
    } else {
        Ok(())
    }
}

fn verify_header_consistency(t: Table) -> EntryResult<Table> {
    use std::error::Error;
    if let Err(e) = verify_header(&t) {
        Err(ParserError::new(ParserErrorKind::HeaderInconsistency, None))
    } else {
        Ok(t)
    }
}

fn has_only_tables(t: &Table) -> bool {
    debug!("Verifying that table has only tables");
    t.iter().all(|(_, x)| if let &Value::Table(_) = x { true } else { false })
}

fn has_main_section(t: &Table) -> bool {
    t.contains_key("imag") &&
        match t.get("imag") {
            Some(&Value::Table(_)) => true,
            Some(_)                => false,
            None                   => false,
        }
}

fn has_imag_version_in_main_section(t: &Table) -> bool {
    use semver::Version;

    match t.get("imag").unwrap() {
        &Value::Table(ref sec) => {
            sec.get("version")
                .and_then(|v| {
                    match v {
                        &Value::String(ref s) => {
                            Some(Version::parse(&s[..]).is_ok())
                        },
                        _                 => Some(false),
                    }
                })
            .unwrap_or(false)
        }
        _                  => false,
    }
}

/**
 * An Entry of the store
 *
 * Contains location, header and content part.
 */
#[derive(Debug, Clone)]
pub struct Entry {
    location: StoreId,
    header: EntryHeader,
    content: EntryContent,
}

impl Entry {

    pub fn new(loc: StoreId) -> Entry {
        Entry {
            location: loc,
            header: EntryHeader::new(),
            content: EntryContent::new()
        }
    }

    pub fn from_file(loc: StoreId, file: &mut File) -> Result<Entry> {
        let text = {
            use std::io::Read;
            let mut s = String::new();
            try!(file.read_to_string(&mut s));
            s
        };
        Self::from_str(loc, &text[..])
    }

    pub fn from_str(loc: StoreId, s: &str) -> Result<Entry> {
        debug!("Building entry from string");
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?smx)
                ^---$
                (?P<header>.*) # Header
                ^---$\n
                (?P<content>.*) # Content
            ").unwrap();
        }

        let matches = RE.captures(s);

        if matches.is_none() {
            return Err(StoreError::new(StoreErrorKind::MalformedEntry, None));
        }

        let matches = matches.unwrap();

        let header = matches.name("header");
        let content = matches.name("content").unwrap_or("");

        if header.is_none() {
            return Err(StoreError::new(StoreErrorKind::MalformedEntry, None));
        }

        debug!("Header and content found. Yay! Building Entry object now");
        Ok(Entry {
            location: loc,
            header: try!(EntryHeader::parse(header.unwrap())),
            content: content.into(),
        })
    }

    pub fn to_str(&self) -> String {
        format!("---{header}---\n{content}",
                header  = ::toml::encode_str(&self.header.header),
                content = self.content)
    }

    pub fn get_location(&self) -> &StoreId {
        &self.location
    }

    pub fn get_header(&self) -> &EntryHeader {
        &self.header
    }

    pub fn get_header_mut(&mut self) -> &mut EntryHeader {
        &mut self.header
    }

    pub fn get_content(&self) -> &EntryContent {
        &self.content
    }

    pub fn get_content_mut(&mut self) -> &mut EntryContent {
        &mut self.content
    }

    pub fn verify(&self) -> Result<()> {
        self.header.verify()
    }

}


#[cfg(test)]
mod test {
    extern crate env_logger;

    use std::collections::BTreeMap;
    use super::EntryHeader;
    use super::Token;

    use toml::Value;

    #[test]
    fn test_imag_section() {
        use super::has_main_section;

        let mut map = BTreeMap::new();
        map.insert("imag".into(), Value::Table(BTreeMap::new()));

        assert!(has_main_section(&map));
    }

    #[test]
    fn test_imag_invalid_section_type() {
        use super::has_main_section;

        let mut map = BTreeMap::new();
        map.insert("imag".into(), Value::Boolean(false));

        assert!(!has_main_section(&map));
    }

    #[test]
    fn test_imag_abscent_main_section() {
        use super::has_main_section;

        let mut map = BTreeMap::new();
        map.insert("not_imag".into(), Value::Boolean(false));

        assert!(!has_main_section(&map));
    }

    #[test]
    fn test_main_section_without_version() {
        use super::has_imag_version_in_main_section;

        let mut map = BTreeMap::new();
        map.insert("imag".into(), Value::Table(BTreeMap::new()));

        assert!(!has_imag_version_in_main_section(&map));
    }

    #[test]
    fn test_main_section_with_version() {
        use super::has_imag_version_in_main_section;

        let mut map = BTreeMap::new();
        let mut sub = BTreeMap::new();
        sub.insert("version".into(), Value::String("0.0.0".into()));
        map.insert("imag".into(), Value::Table(sub));

        assert!(has_imag_version_in_main_section(&map));
    }

    #[test]
    fn test_main_section_with_version_in_wrong_type() {
        use super::has_imag_version_in_main_section;

        let mut map = BTreeMap::new();
        let mut sub = BTreeMap::new();
        sub.insert("version".into(), Value::Boolean(false));
        map.insert("imag".into(), Value::Table(sub));

        assert!(!has_imag_version_in_main_section(&map));
    }

    #[test]
    fn test_verification_good() {
        use super::verify_header_consistency;

        let mut header = BTreeMap::new();
        let sub = {
            let mut sub = BTreeMap::new();
            sub.insert("version".into(), Value::String(String::from("0.0.0")));

            Value::Table(sub)
        };

        header.insert("imag".into(), sub);

        assert!(verify_header_consistency(header).is_ok());
    }

    #[test]
    fn test_verification_invalid_versionstring() {
        use super::verify_header_consistency;

        let mut header = BTreeMap::new();
        let sub = {
            let mut sub = BTreeMap::new();
            sub.insert("version".into(), Value::String(String::from("000")));

            Value::Table(sub)
        };

        header.insert("imag".into(), sub);

        assert!(!verify_header_consistency(header).is_ok());
    }


    #[test]
    fn test_verification_current_version() {
        use version;

        use super::verify_header_consistency;

        let mut header = BTreeMap::new();
        let sub = {
            let mut sub = BTreeMap::new();
            sub.insert("version".into(), Value::String(String::from(version!())));

            Value::Table(sub)
        };

        header.insert("imag".into(), sub);

        assert!(verify_header_consistency(header).is_ok());
    }

    static TEST_ENTRY : &'static str = "---
[imag]
version = \"0.0.3\"
---
Hai";

    #[test]
    fn test_entry_from_str() {
        use super::Entry;
        use std::path::PathBuf;
        println!("{}", TEST_ENTRY);
        let entry = Entry::from_str(PathBuf::from("/test/foo~1.3"),
                                    TEST_ENTRY).unwrap();

        assert_eq!(entry.content, "Hai");
    }

    #[test]
    fn test_entry_to_str() {
        use super::Entry;
        use std::path::PathBuf;
        println!("{}", TEST_ENTRY);
        let entry = Entry::from_str(PathBuf::from("/test/foo~1.3"),
                                    TEST_ENTRY).unwrap();
        let string = entry.to_str();

        assert_eq!(TEST_ENTRY, string);
    }

    #[test]
    fn test_walk_header_simple() {
        let tokens = EntryHeader::tokenize("a", '.').unwrap();
        assert!(tokens.len() == 1, "1 token was expected, {} were parsed", tokens.len());
        assert!(tokens.iter().next().unwrap() == &Token::Key(String::from("a")),
                "'a' token was expected, {:?} was parsed", tokens.iter().next());

        let mut header = BTreeMap::new();
        header.insert(String::from("a"), Value::Integer(1));

        let mut v_header = Value::Table(header);
        let res = EntryHeader::walk_header(&mut v_header, tokens);
        assert_eq!(&mut Value::Integer(1), res.unwrap());
    }

    #[test]
    fn test_walk_header_with_array() {
        let tokens = EntryHeader::tokenize("a.0", '.').unwrap();
        assert!(tokens.len() == 2, "2 token was expected, {} were parsed", tokens.len());
        assert!(tokens.iter().next().unwrap() == &Token::Key(String::from("a")),
                "'a' token was expected, {:?} was parsed", tokens.iter().next());

        let mut header = BTreeMap::new();
        let ary = Value::Array(vec![Value::Integer(1)]);
        header.insert(String::from("a"), ary);


        let mut v_header = Value::Table(header);
        let res = EntryHeader::walk_header(&mut v_header, tokens);
        assert_eq!(&mut Value::Integer(1), res.unwrap());
    }

    #[test]
    fn test_walk_header_extract_array() {
        let tokens = EntryHeader::tokenize("a", '.').unwrap();
        assert!(tokens.len() == 1, "1 token was expected, {} were parsed", tokens.len());
        assert!(tokens.iter().next().unwrap() == &Token::Key(String::from("a")),
                "'a' token was expected, {:?} was parsed", tokens.iter().next());

        let mut header = BTreeMap::new();
        let ary = Value::Array(vec![Value::Integer(1)]);
        header.insert(String::from("a"), ary);

        let mut v_header = Value::Table(header);
        let res = EntryHeader::walk_header(&mut v_header, tokens);
        assert_eq!(&mut Value::Array(vec![Value::Integer(1)]), res.unwrap());
    }

    /**
     * Creates a big testing header.
     *
     * JSON equivalent:
     *
     * ```json
     * {
     *      "a": {
     *          "array": [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 ]
     *      },
     *      "b": {
     *          "array": [ "string1", "string2", "string3", "string4" ]
     *      },
     *      "c": {
     *          "array": [ 1, "string2", 3, "string4" ]
     *      },
     *      "d": {
     *          "array": [
     *              {
     *                  "d1": 1
     *              },
     *              {
     *                  "d2": 2
     *              },
     *              {
     *                  "d3": 3
     *              },
     *          ],
     *
     *          "something": "else",
     *
     *          "and": {
     *              "something": {
     *                  "totally": "different"
     *              }
     *          }
     *      }
     * }
     * ```
     *
     * The sections "a", "b", "c", "d" are created in the respective helper functions
     * create_header_section_a, create_header_section_b, create_header_section_c and
     * create_header_section_d.
     *
     * These functions can also be used for testing.
     *
     */
    fn create_header() -> Value {
        let a = create_header_section_a();
        let b = create_header_section_b();
        let c = create_header_section_c();
        let d = create_header_section_d();

        let mut header = BTreeMap::new();
        header.insert(String::from("a"), a);
        header.insert(String::from("b"), b);
        header.insert(String::from("c"), c);
        header.insert(String::from("d"), d);

        Value::Table(header)
    }

    fn create_header_section_a() -> Value {
        // 0..10 is exclusive 10
        let a_ary = Value::Array((0..10).map(|x| Value::Integer(x)).collect());

        let mut a_obj = BTreeMap::new();
        a_obj.insert(String::from("array"), a_ary);
        Value::Table(a_obj)
    }

    fn create_header_section_b() -> Value {
        let b_ary = Value::Array((0..9)
                                 .map(|x| Value::String(format!("string{}", x)))
                                 .collect());

        let mut b_obj = BTreeMap::new();
        b_obj.insert(String::from("array"), b_ary);
        Value::Table(b_obj)
    }

    fn create_header_section_c() -> Value {
        let c_ary = Value::Array(
            vec![
                Value::Integer(1),
                Value::String(String::from("string2")),
                Value::Integer(3),
                Value::String(String::from("string4"))
            ]);

        let mut c_obj = BTreeMap::new();
        c_obj.insert(String::from("array"), c_ary);
        Value::Table(c_obj)
    }

    fn create_header_section_d() -> Value {
        let d_ary = Value::Array(
            vec![
                {
                    let mut tab = BTreeMap::new();
                    tab.insert(String::from("d1"), Value::Integer(1));
                    tab
                },
                {
                    let mut tab = BTreeMap::new();
                    tab.insert(String::from("d2"), Value::Integer(2));
                    tab
                },
                {
                    let mut tab = BTreeMap::new();
                    tab.insert(String::from("d3"), Value::Integer(3));
                    tab
                },
            ].into_iter().map(Value::Table).collect());

        let and_obj = Value::Table({
            let mut tab = BTreeMap::new();
            let something_tab = Value::Table({
                let mut tab = BTreeMap::new();
                tab.insert(String::from("totally"), Value::String(String::from("different")));
                tab
            });
            tab.insert(String::from("something"), something_tab);
            tab
        });

        let mut d_obj = BTreeMap::new();
        d_obj.insert(String::from("array"), d_ary);
        d_obj.insert(String::from("something"), Value::String(String::from("else")));
        d_obj.insert(String::from("and"), and_obj);
        Value::Table(d_obj)
    }

    #[test]
    fn test_walk_header_big_a() {
        test_walk_header_extract_section("a", &create_header_section_a());
    }

    #[test]
    fn test_walk_header_big_b() {
        test_walk_header_extract_section("b", &create_header_section_b());
    }

    #[test]
    fn test_walk_header_big_c() {
        test_walk_header_extract_section("c", &create_header_section_c());
    }

    #[test]
    fn test_walk_header_big_d() {
        test_walk_header_extract_section("d", &create_header_section_d());
    }

    fn test_walk_header_extract_section(secname: &str, expected: &Value) {
        let tokens = EntryHeader::tokenize(secname, '.').unwrap();
        assert!(tokens.len() == 1, "1 token was expected, {} were parsed", tokens.len());
        assert!(tokens.iter().next().unwrap() == &Token::Key(String::from(secname)),
                "'{}' token was expected, {:?} was parsed", secname, tokens.iter().next());

        let mut header = create_header();
        let res = EntryHeader::walk_header(&mut header, tokens);
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_walk_header_extract_numbers() {
        test_extract_number("a", 0, 0);
        test_extract_number("a", 1, 1);
        test_extract_number("a", 2, 2);
        test_extract_number("a", 3, 3);
        test_extract_number("a", 4, 4);
        test_extract_number("a", 5, 5);
        test_extract_number("a", 6, 6);
        test_extract_number("a", 7, 7);
        test_extract_number("a", 8, 8);
        test_extract_number("a", 9, 9);

        test_extract_number("c", 0, 1);
        test_extract_number("c", 2, 3);
    }

    fn test_extract_number(sec: &str, idx: usize, exp: i64) {
        let tokens = EntryHeader::tokenize(&format!("{}.array.{}", sec, idx)[..], '.').unwrap();
        assert!(tokens.len() == 3, "3 token was expected, {} were parsed", tokens.len());
        {
            let mut iter = tokens.iter();

            let tok = iter.next().unwrap();
            let exp = Token::Key(String::from(sec));
            assert!(tok == &exp, "'{}' token was expected, {:?} was parsed", sec, tok);

            let tok = iter.next().unwrap();
            let exp = Token::Key(String::from("array"));
            assert!(tok == &exp, "'array' token was expected, {:?} was parsed", tok);

            let tok = iter.next().unwrap();
            let exp = Token::Index(idx);
            assert!(tok == &exp, "'{}' token was expected, {:?} was parsed", idx, tok);
        }

        let mut header = create_header();
        let res = EntryHeader::walk_header(&mut header, tokens);
        assert_eq!(&mut Value::Integer(exp), res.unwrap());
    }

    #[test]
    fn test_header_read() {
        let v = create_header();
        let h = match v {
            Value::Table(t) => EntryHeader::from_table(t),
            _ => panic!("create_header() doesn't return a table!"),
        };

        assert!(if let Ok(Some(Value::Table(_)))  = h.read("a") { true } else { false });
        assert!(if let Ok(Some(Value::Array(_)))   = h.read("a.array") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(_))) = h.read("a.array.1") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(_))) = h.read("a.array.9") { true } else { false });

        assert!(if let Ok(Some(Value::Table(_))) = h.read("c") { true } else { false });
        assert!(if let Ok(Some(Value::Array(_)))  = h.read("c.array") { true } else { false });
        assert!(if let Ok(Some(Value::String(_))) = h.read("c.array.1") { true } else { false });
        assert!(if let Ok(None) = h.read("c.array.9") { true } else { false });

        assert!(if let Ok(Some(Value::Integer(_))) = h.read("d.array.0.d1") { true } else { false });
        assert!(if let Ok(None) = h.read("d.array.0.d2") { true } else { false });
        assert!(if let Ok(None) = h.read("d.array.0.d3") { true } else { false });

        assert!(if let Ok(None) = h.read("d.array.1.d1") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(_))) = h.read("d.array.1.d2") { true } else { false });
        assert!(if let Ok(None) = h.read("d.array.1.d3") { true } else { false });

        assert!(if let Ok(None) = h.read("d.array.2.d1") { true } else { false });
        assert!(if let Ok(None) = h.read("d.array.2.d2") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(_))) = h.read("d.array.2.d3") { true } else { false });

        assert!(if let Ok(Some(Value::String(_))) = h.read("d.something") { true } else { false });
        assert!(if let Ok(Some(Value::Table(_))) = h.read("d.and") { true } else { false });
        assert!(if let Ok(Some(Value::Table(_))) = h.read("d.and.something") { true } else { false });
        assert!(if let Ok(Some(Value::String(_))) = h.read("d.and.something.totally") { true } else { false });
    }

    #[test]
    fn test_header_set_override() {
        let _ = env_logger::init();
        let v = create_header();
        let mut h = match v {
            Value::Table(t) => EntryHeader::from_table(t),
            _ => panic!("create_header() doesn't return a table!"),
        };

        println!("Testing index 0");
        assert_eq!(h.read("a.array.0").unwrap().unwrap(), Value::Integer(0));

        println!("Altering index 0");
        assert_eq!(h.set("a.array.0", Value::Integer(42)).unwrap().unwrap(), Value::Integer(0));

        println!("Values now: {:?}", h);

        println!("Testing all indexes");
        assert_eq!(h.read("a.array.0").unwrap().unwrap(), Value::Integer(42));
        assert_eq!(h.read("a.array.1").unwrap().unwrap(), Value::Integer(1));
        assert_eq!(h.read("a.array.2").unwrap().unwrap(), Value::Integer(2));
        assert_eq!(h.read("a.array.3").unwrap().unwrap(), Value::Integer(3));
        assert_eq!(h.read("a.array.4").unwrap().unwrap(), Value::Integer(4));
        assert_eq!(h.read("a.array.5").unwrap().unwrap(), Value::Integer(5));
        assert_eq!(h.read("a.array.6").unwrap().unwrap(), Value::Integer(6));
        assert_eq!(h.read("a.array.7").unwrap().unwrap(), Value::Integer(7));
        assert_eq!(h.read("a.array.8").unwrap().unwrap(), Value::Integer(8));
        assert_eq!(h.read("a.array.9").unwrap().unwrap(), Value::Integer(9));
    }

    #[test]
    fn test_header_set_new() {
        let _ = env_logger::init();
        let v = create_header();
        let mut h = match v {
            Value::Table(t) => EntryHeader::from_table(t),
            _ => panic!("create_header() doesn't return a table!"),
        };

        assert!(h.read("a.foo").is_ok());
        assert!(h.read("a.foo").unwrap().is_none());

        {
            let v = h.set("a.foo", Value::Integer(42));
            assert!(v.is_ok());
            assert!(v.unwrap().is_none());

            assert!(if let Ok(Some(Value::Table(_))) = h.read("a") { true } else { false });
            assert!(if let Ok(Some(Value::Integer(_))) = h.read("a.foo") { true } else { false });
        }

        {
            let v = h.set("new", Value::Table(BTreeMap::new()));
            assert!(v.is_ok());
            assert!(v.unwrap().is_none());

            let v = h.set("new.subset", Value::Table(BTreeMap::new()));
            assert!(v.is_ok());
            assert!(v.unwrap().is_none());

            let v = h.set("new.subset.dest", Value::Integer(1337));
            assert!(v.is_ok());
            assert!(v.unwrap().is_none());

            assert!(if let Ok(Some(Value::Table(_))) = h.read("new") { true } else { false });
            assert!(if let Ok(Some(Value::Table(_))) = h.read("new.subset") { true } else { false });
            assert!(if let Ok(Some(Value::Integer(_))) = h.read("new.subset.dest") { true } else { false });
        }
    }


    #[test]
    fn test_header_insert_override() {
        let _ = env_logger::init();
        let v = create_header();
        let mut h = match v {
            Value::Table(t) => EntryHeader::from_table(t),
            _ => panic!("create_header() doesn't return a table!"),
        };

        println!("Testing index 0");
        assert_eq!(h.read("a.array.0").unwrap().unwrap(), Value::Integer(0));

        println!("Altering index 0");
        assert_eq!(h.insert("a.array.0", Value::Integer(42)).unwrap(), false);
        println!("...should have failed");

        println!("Testing all indexes");
        assert_eq!(h.read("a.array.0").unwrap().unwrap(), Value::Integer(0));
        assert_eq!(h.read("a.array.1").unwrap().unwrap(), Value::Integer(1));
        assert_eq!(h.read("a.array.2").unwrap().unwrap(), Value::Integer(2));
        assert_eq!(h.read("a.array.3").unwrap().unwrap(), Value::Integer(3));
        assert_eq!(h.read("a.array.4").unwrap().unwrap(), Value::Integer(4));
        assert_eq!(h.read("a.array.5").unwrap().unwrap(), Value::Integer(5));
        assert_eq!(h.read("a.array.6").unwrap().unwrap(), Value::Integer(6));
        assert_eq!(h.read("a.array.7").unwrap().unwrap(), Value::Integer(7));
        assert_eq!(h.read("a.array.8").unwrap().unwrap(), Value::Integer(8));
        assert_eq!(h.read("a.array.9").unwrap().unwrap(), Value::Integer(9));
    }

    #[test]
    fn test_header_insert_new() {
        let _ = env_logger::init();
        let v = create_header();
        let mut h = match v {
            Value::Table(t) => EntryHeader::from_table(t),
            _ => panic!("create_header() doesn't return a table!"),
        };

        assert!(h.read("a.foo").is_ok());
        assert!(h.read("a.foo").unwrap().is_none());

        {
            let v = h.insert("a.foo", Value::Integer(42));
            assert!(v.is_ok());
            assert_eq!(v.unwrap(), true);

            assert!(if let Ok(Some(Value::Table(_))) = h.read("a") { true } else { false });
            assert!(if let Ok(Some(Value::Integer(_))) = h.read("a.foo") { true } else { false });
        }

        {
            let v = h.insert("new", Value::Table(BTreeMap::new()));
            assert!(v.is_ok());
            assert_eq!(v.unwrap(), true);

            let v = h.insert("new.subset", Value::Table(BTreeMap::new()));
            assert!(v.is_ok());
            assert_eq!(v.unwrap(), true);

            let v = h.insert("new.subset.dest", Value::Integer(1337));
            assert!(v.is_ok());
            assert_eq!(v.unwrap(), true);

            assert!(if let Ok(Some(Value::Table(_))) = h.read("new") { true } else { false });
            assert!(if let Ok(Some(Value::Table(_))) = h.read("new.subset") { true } else { false });
            assert!(if let Ok(Some(Value::Integer(_))) = h.read("new.subset.dest") { true } else { false });
        }
    }

    #[test]
    fn test_header_delete() {
        let _ = env_logger::init();
        let v = create_header();
        let mut h = match v {
            Value::Table(t) => EntryHeader::from_table(t),
            _ => panic!("create_header() doesn't return a table!"),
        };

        assert!(if let Ok(Some(Value::Table(_)))   = h.read("a") { true } else { false });
        assert!(if let Ok(Some(Value::Array(_)))   = h.read("a.array") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(_))) = h.read("a.array.1") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(_))) = h.read("a.array.9") { true } else { false });

        assert!(if let Ok(Some(Value::Integer(1))) = h.delete("a.array.1") { true } else { false });
        assert!(if let Ok(Some(Value::Integer(9))) = h.delete("a.array.8") { true } else { false });
        assert!(if let Ok(Some(Value::Array(_)))   = h.delete("a.array") { true } else { false });
        assert!(if let Ok(Some(Value::Table(_)))   = h.delete("a") { true } else { false });

    }

}

