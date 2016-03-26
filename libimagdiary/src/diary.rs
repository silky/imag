use std::ops::Deref;

use libimagnotes::note::Note;
use libimagnotes::note::NoteIterator;
use libimagstore::store::Store;
use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagrt::runtime::Runtime;

use config::create_daily_entries;
use error::DiaryError as DE;
use error::DiaryErrorKind as DEK;
use result::Result;

pub type DiaryEntry<'a> = Note<'a>;

pub struct Diary<'a> {
    store: &'a Store,
    description: Note<'a>
}

impl<'a> Diary<'a> {

    pub fn new(store: &'a Store, name: String, description: String) -> Result<Diary<'a>> {
        Note::new(store, format!("{}/description", name), description)
            .map(|note| Diary { store: store, description: note })
            .map_err(|e| DE::new(DEK::StoreWriteError, Some(Box::new(e))))
    }

    pub fn name(&self) -> Option<&str> {
        self.description
            .deref()
            .deref()
            .get_location()
            .parent()
            .and_then(|p| p.to_str())
    }

    pub fn all_entries(store: &'a Store, diary_name: &'a str) -> Result<DiaryEntryIterator<'a>> {
        store.retrieve_for_module("diary")
            .map(|iter| NoteIterator::new(store, iter))
            .map(|iter| DiaryEntryIterator::new(diary_name, iter))
            .map_err(|e| DE::new(DEK::StoreReadError, Some(Box::new(e))))
    }

    pub fn new_entry(&self, rt: &Runtime) -> Result<DiaryEntry> {
        if create_daily_entries(rt) {
            unimplemented!()
        } else {
            unimplemented!()
        }
    }

}

pub struct DiaryEntryIterator<'a> {
    name: &'a str,
    iter: NoteIterator<'a>,
}

impl<'a> DiaryEntryIterator<'a> {

    pub fn new(diaryname: &'a str, iter: NoteIterator<'a>) -> DiaryEntryIterator<'a> {
        DiaryEntryIterator {
            name: diaryname,
            iter: iter,
        }
    }

}

impl<'a> Iterator for DiaryEntryIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        loop {
            let next = self.iter.next();
            if next.is_none() {
                return None;
            }
            let next = next.unwrap();
            if next.is_err() {
                return Some(Err(DE::new(DEK::StoreReadError, Some(Box::new(next.err().unwrap())))));
            }
            let next = next.unwrap();

            if next.deref().is_in_diary(self.name) {
                return Some(Ok(next))
            }
        }
    }

}

trait IsInDiary {

    fn is_in_diary(&self, name: &str) -> bool;

}

impl IsInDiary for Entry {

    fn is_in_diary(&self, name: &str) -> bool {
        self.get_location()
            .parent()
            .map(|parent| parent.is_dir() && parent.to_str().map(|l| l == name).unwrap_or(false))
            .unwrap_or(false)
    }

}

impl<'a> IsInDiary for FileLockEntry<'a> {

    fn is_in_diary(&self, name: &str) -> bool {
        self.deref().is_in_diary(name)
    }

}

