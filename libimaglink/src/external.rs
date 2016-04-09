/// External linking is a complex implementation to be able to serve a clean and easy-to-use
/// interface.
///
/// Internally, there are no such things as "external links" (plural). Each Entry in the store can
/// only have _one_ external link.
///
/// This library does the following therefor: It allows you to have several external links with one
/// entry, which are internally one file in the store for each link, linked with "internal
/// linking".
///
/// This helps us greatly with deduplication of URLs.
///

use std::convert::Into;
use std::ops::Deref;

use libimagstore::store::Entry;
use libimagstore::store::EntryHeader;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;

use error::LinkError as LE;
use error::LinkErrorKind as LEK;
use result::Result;
use internal::InternalLinker;

use toml::Value;
use toml::Table;
use url::Url;

/// "Link" Type, just an abstraction over FileLockEntry to have some convenience internally.
struct Link<'a> {
    link: FileLockEntry<'a>
}

impl<'a> Link<'a> {

    pub fn new(fle: FileLockEntry<'a>) -> Link<'a> {
        Link { link: fle }
    }

    /// For interal use only. Load an Link from a store id, if this is actually a Link
    fn retrieve(store: &'a Store, id: StoreId) -> Result<Option<Link<'a>>> {
        store.retrieve(id)
            .map(|fle| {
                if let Some(_) = Link::get_link_uri_from_filelockentry(&fle) {
                    Some(Link {
                        link: fle
                    })
                } else {
                    None
                }
            })
            .map_err(|e| LE::new(LEK::StoreReadError, Some(Box::new(e))))
    }

    /// Get a link Url object from a FileLockEntry, ignore errors.
    fn get_link_uri_from_filelockentry(file: &FileLockEntry<'a>) -> Option<Url> {
        file.deref()
            .get_header()
            .read("imag.content.uri")
            .ok()
            .and_then(|opt| {
                match opt {
                    Some(Value::String(s)) => Url::parse(&s[..]).ok(),
                    _ => None
                }
            })
    }

    pub fn get_url(&self) -> Result<Option<Url>> {
        let opt = self.link
            .deref()
            .get_header()
            .read("imag.content.uri");

        match opt {
            Ok(Some(Value::String(s))) => {
                Url::parse(&s[..])
                     .map(|s| Some(s))
                     .map_err(|e| LE::new(LEK::EntryHeaderReadError, Some(Box::new(e))))
            },
            Ok(None) => Ok(None),
            _ => Err(LE::new(LEK::EntryHeaderReadError, None))
        }
    }

}

pub trait ExternalLinker : InternalLinker {

    /// Get the external links from the implementor object
    fn get_external_links(&self, store: &Store) -> Result<Vec<Url>>;

    /// Set the external links for the implementor object
    fn set_external_links(&mut self, links: Vec<Url>) -> Result<Vec<Url>>;

    /// Add an external link to the implementor object
    fn add_external_link(&mut self, link: Url) -> Result<()>;

    /// Remove an external link from the implementor object
    fn remove_external_link(&mut self, link: Url) -> Result<()>;

}

/// Check whether the StoreId starts with `/link/external/`
fn is_link_store_id(id: &StoreId) -> bool {
    id.starts_with("/link/external/")
}

fn get_external_link_from_file(entry: &FileLockEntry) -> Result<Url> {
    Link::get_link_uri_from_filelockentry(entry) // TODO: Do not hide error by using this function
        .ok_or(LE::new(LEK::StoreReadError, None))
}

/// Implement ExternalLinker for Entry, hiding the fact that there is no such thing as an external
/// link in an entry, but internal links to other entries which serve as external links, as one
/// entry in the store can only have one external link.
impl ExternalLinker for Entry {

    /// Get the external links from the implementor object
    fn get_external_links(&self, store: &Store) -> Result<Vec<Url>> {
        // Iterate through all internal links and filter for FileLockEntries which live in
        // /link/external/<SHA> -> load these files and get the external link from their headers,
        // put them into the return vector.
        self.get_internal_links()
            .map(|vect| {
                vect.into_iter()
                    .filter(is_link_store_id)
                    .map(|id| {
                        match store.retrieve(id) {
                            Ok(f) => get_external_link_from_file(&f),
                            Err(e) => Err(LE::new(LEK::StoreReadError, Some(Box::new(e)))),
                        }
                    })
                    .filter_map(|x| x.ok()) // TODO: Do not ignore error here
                    .collect()
            })
            .map_err(|e| LE::new(LEK::StoreReadError, Some(Box::new(e))))
    }

    /// Set the external links for the implementor object
    fn set_external_links(&mut self, links: Vec<Url>) -> Result<Vec<Url>> {
        // Take all the links, generate a SHA sum out of each one, filter out the already existing
        // store entries and store the other URIs in the header of one FileLockEntry each, in
        // the path /link/external/<SHA of the URL>
        unimplemented!()
    }

    /// Add an external link to the implementor object
    fn add_external_link(&mut self, link: Url) -> Result<()> {
        // get external links, add this one, save them
        unimplemented!()
    }

    /// Remove an external link from the implementor object
    fn remove_external_link(&mut self, link: Url) -> Result<()> {
        // get external links, remove this one, save them
        unimplemented!()
    }

}

