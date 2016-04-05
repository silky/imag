use std::ops::Deref;

use libimagnotes::note::Note;
use libimagnotes::note::NoteIterator;

use diaryid::DiaryId;
use diary::IsInDiary;
use diary::DiaryEntry;
use error::DiaryError as DE;
use error::DiaryErrorKind as DEK;
use result::Result;

/// A iterator for iterating over diary entries
pub struct DiaryEntryIterator<'a> {
    name: &'a str,
    iter: NoteIterator<'a>,

    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
}

impl<'a> DiaryEntryIterator<'a> {

    pub fn new(diaryname: &'a str, iter: NoteIterator<'a>) -> DiaryEntryIterator<'a> {
        DiaryEntryIterator {
            name: diaryname,
            iter: iter,

            year: None,
            month: None,
            day: None,
        }
    }

    // Filter by year, get all diary entries for this year
    pub fn year(mut self, year: i32) -> DiaryEntryIterator<'a> {
        self.year = Some(year);
        self
    }

    // Filter by month, get all diary entries for this month (every year)
    pub fn month(mut self, month: u32) -> DiaryEntryIterator<'a> {
        self.month = Some(month);
        self
    }

    // Filter by day, get all diary entries for this day (every year, every year)
    pub fn day(mut self, day: u32) -> DiaryEntryIterator<'a> {
        self.day = Some(day);
        self
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
                let id = next.deref().get_location().to_str().and_then(DiaryId::parse);
                if id.is_none() {
                    return Some(Err(DE::new(DEK::PathConversionError, None)));
                }
                let id = id.unwrap();

                let y = match self.year  { None => true, Some(y) => y == id.year() };
                let m = match self.month { None => true, Some(m) => m == id.month() };
                let d = match self.day   { None => true, Some(d) => d == id.day() };

                if y && m && d {
                    return Some(Ok(next));
                }
            }
        }
    }

}

