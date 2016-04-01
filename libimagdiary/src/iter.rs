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
}

impl<'a> DiaryEntryIterator<'a> {

    pub fn new(diaryname: &'a str, iter: NoteIterator<'a>) -> DiaryEntryIterator<'a> {
        DiaryEntryIterator {
            name: diaryname,
            iter: iter,
        }
    }

    // Filter by year, get all diary entries for this year
    pub fn year(self, year: i32) -> DiaryYearIterator<'a> {
        DiaryYearIterator {
            iter: self,
            year: year,
        }
    }

    // Filter by month, get all diary entries for this month (every year)
    pub fn month(self, month: u32) -> DiaryMonthIterator<'a> {
        DiaryMonthIterator {
            iter: self,
            month: month,
        }
    }

    // Filter by day, get all diary entries for this day (every year, every year)
    pub fn day(self, day: u32) -> DiaryDayIterator<'a> {
        DiaryDayIterator {
            iter: self,
            day: day,
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

/// Iterator for Iterating over diary entries per year
pub struct DiaryYearIterator<'a> {
    iter: DiaryEntryIterator<'a>,
    year: i32,
}

impl<'a> Iterator for DiaryYearIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        loop {
            match self.iter.next() {
                None           => return None,
                Some(Err(e))   => return Some(Err(e)),
                Some(Ok(note)) => {
                    if note.deref()
                        .get_location()
                        .to_str()
                        .and_then(DiaryId::parse)
                        .map(|id| id.year() == self.year)
                        .unwrap_or(false)
                    {
                        return Some(Ok(note))
                    }
                }
            }
        }
    }

}

/// Iterator for Iterating over diary entries per month
pub struct DiaryMonthIterator<'a> {
    iter: DiaryEntryIterator<'a>,
    month: u32,
}

impl<'a> Iterator for DiaryMonthIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        unimplemented!()
    }

}

/// Iterator for Iterating over diary entries per month for a year
pub struct DiaryYearMonthIterator<'a> {
    iter: DiaryYearIterator<'a>,
    month: u32,
}

impl<'a> Iterator for DiaryYearMonthIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        unimplemented!()
    }

}

/// Iterator for Iterating over diary entries per day
pub struct DiaryDayIterator<'a> {
    iter: DiaryEntryIterator<'a>,
    day: u32,
}

impl<'a> Iterator for DiaryDayIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        unimplemented!()
    }

}

/// Iterator for Iterating over diary entries per day for one year
pub struct DiaryYearDayIterator<'a> {
    iter: DiaryYearIterator<'a>,
    day: u32,
}

impl<'a> Iterator for DiaryYearDayIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        unimplemented!()
    }

}

/// Iterator for Iterating over diary entries per day for one month in one year
pub struct DiaryYearMonthDayIterator<'a> {
    iter: DiaryYearMonthIterator<'a>,
    day: u32,
}

impl<'a> Iterator for DiaryYearMonthDayIterator<'a> {
    type Item = Result<DiaryEntry<'a>>;

    fn next(&mut self) -> Option<Result<DiaryEntry<'a>>> {
        unimplemented!()
    }

}
