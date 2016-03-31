extern crate chrono;
#[macro_use] extern crate log;
extern crate semver;
extern crate toml;

#[macro_use] extern crate libimagstore;
extern crate libimagrt;
extern crate libimagnotes;

module_entry_path_mod!("diary", "0.1.0");

pub mod config;
pub mod diary;
pub mod error;
pub mod result;

