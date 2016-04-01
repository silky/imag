use std::ops::Deref;

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::offset::local::Local;
use chrono::Timelike;
use chrono::Datelike;
use libimagnotes::note::Note;
use libimagnotes::note::NoteIterator;
use libimagstore::store::Store;
use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreId;
use libimagrt::runtime::Runtime;

use config::get_diary_type;
use error::DiaryError as DE;
use error::DiaryErrorKind as DEK;
use result::Result;
use config::DiaryType;
use module_path::ModuleEntryPath;

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

    pub fn new_entry(&self, rt: &'a Runtime) -> Result<DiaryEntry<'a>> {
        let diaryname = self.name().map(String::from);
        if diaryname.is_none() {
            return Err(DE::new(DEK::CannotFindDiary, None));
        }
        let diaryname  = diaryname.unwrap();
        let diary_type = get_diary_type(rt, &diaryname);
        let name = {
            let dt  = Local::now();
            let ndt = dt.naive_local();
            let m = ndt.month();

            match diary_type {
                DiaryType::Monthly  => build_filename(diaryname, ndt, m, 0, 0, 0),
                DiaryType::Daily    => build_filename(diaryname, ndt, m, ndt.day(), 0, 0),
                DiaryType::Hourly   => build_filename(diaryname, ndt, m, ndt.day(), ndt.hour(), 0),
                DiaryType::Minutely => build_filename(diaryname, ndt, m, ndt.day(), ndt.hour(), ndt.minute()),
            }
        };

        // TODO: get init-text from configuration. for "Dear Diary,\n" for example.

        // As DiaryEntry == Note, we use the Note::new() functionality here.
        Note::new(rt.store(), name, String::new())
            .map_err(|e| DE::new(DEK::CannotCreateNote, Some(Box::new(e))))
    }

    pub fn retrieve(store: &'a Store, name: String) -> Result<Diary<'a>> {
        Note::retrieve(store, name).map(|note| {
            Diary {
                store: store,
                description: note
            }
        })
        .map_err(|e| DE::new(DEK::StoreReadError, Some(Box::new(e))))
    }

    /// Delete a diary and all its entries by name
    pub fn delete(store: &'a Store, name: String) -> Result<()> {
        unimplemented!()
    }

}

fn build_filename(diaryname: String, ndt: NaiveDateTime, mon: u32, day: u32, hour: u32, minute: u32) -> String {
    format!("{}/{}/{}-{}-{}:{}", diaryname, ndt.year(), mon, day, hour, minute)
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

