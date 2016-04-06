#[macro_use] extern crate log;
extern crate clap;
#[macro_use] extern crate version;

extern crate libimagrt;
extern crate libimagdiary;
extern crate libimagutil;

use std::process::exit;

use clap::ArgMatches;

use libimagdiary::diary::Diary;
use libimagdiary::error::DiaryError as DE;
use libimagdiary::error::DiaryErrorKind as DEK;
use libimagrt::edit::Edit;
use libimagrt::runtime::Runtime;
use libimagutil::trace::trace_error;

mod ui;

use ui::build_ui;

fn main() {
    let name = "imag-diary";
    let version = &version!()[..];
    let about = "Personal Diary/Diaries";
    let ui = build_ui(Runtime::get_default_cli_builder(name, version, about));
    let rt = {
        let rt = Runtime::new(ui);
        if rt.is_ok() {
            rt.unwrap()
        } else {
            println!("Could not set up Runtime");
            println!("{:?}", rt.err().unwrap());
            exit(1);
        }
    };

    debug!("Hello. Logging was just enabled");
    debug!("I already set up the Runtime object and build the commandline interface parser.");
    debug!("Lets get rollin' ...");

    match rt.cli().subcommand_name() {
        Some("create") => create(&rt),
        Some("delete") => delete(&rt),
        Some("diary")  => diary(&rt),
        Some("edit")   => edit(&rt),
        Some("list")   => list(&rt),
        None           => {
            info!("No command, calling 'create'");
            create(&rt);
        },
        _ => {
            warn!("Unknown command"); // More error handling
        },
    }
}

fn create(rt: &Runtime) {
    let diaryname = get_diary_name(rt);
    if diaryname.is_none() {
        warn!("No diary selected. Use either the configuration file or the commandline option");
        exit(1);
    }
    let diaryname = diaryname.unwrap();

    let prevent_edit = rt.cli().subcommand_matches("create").unwrap().is_present("no-edit");

    let res = Diary::retrieve(rt.store(), diaryname)
        .and_then(|diary| diary.new_entry(rt))
        .and_then(|mut entry| {
            if prevent_edit {
                debug!("Not editing new diary entry");
                Ok(())
            } else {
                debug!("Editing new diary entry");
                entry.edit_content(rt)
                    .map_err(|e| DE::new(DEK::DiaryEditError, Some(Box::new(e))))
            }
        });

    match res {
        Err(e) => {
            trace_error(&e);
            exit(1);
        },
        Ok(_) => info!("Ok"),
    };
}

fn delete(rt: &Runtime) {
    unimplemented!()
}

fn edit(rt: &Runtime) {
    unimplemented!()
}

fn list(rt: &Runtime) {
    use std::str::FromStr;
    use std::ops::Deref;

    let diary_name = get_diary_name(rt);
    if diary_name.is_none() {
        exit(1);
    }
    let diary_name = diary_name.unwrap();

    Diary::all_entries(rt.store(), &diary_name[..])
        .map(|iter| {
            let scmd  = rt.cli().subcommand_matches("list").unwrap();
            let iter = match scmd.value_of("year").and_then(|s| FromStr::from_str(s).ok()) {
                Some(year) => iter.year(year),
                None       => iter,
            };

            let iter = match scmd.value_of("month").and_then(|s| FromStr::from_str(s).ok()) {
                Some(month) => iter.month(month),
                None        => iter,
            };

            let iter = match scmd.value_of("day").and_then(|s| FromStr::from_str(s).ok()) {
                Some(day) => iter.day(day),
                None      => iter,
            };

            // TODO implement fancy listing using libimagentrylist
            info!("Note: Fancy listing functionality is not yet implemented!");
            for entry in iter {
                match entry.map(|e| e.deref().get_location().to_str().map(String::from)) {
                    Err(e) => trace_error(&e),
                    Ok(None) => warn!("Could not convert location to path"),
                    Ok(Some(location)) => println!("{}", location),
                }
            }
        })
        .map_err(|e| trace_error(&e));
}

fn diary(rt: &Runtime) {
    let cmd = rt.cli().subcommand_matches("diary").unwrap();

    match cmd.subcommand_name() {
        Some("create") => diary_create(rt, &cmd),
        Some("delete") => diary_delete(rt, &cmd),
        Some("edit")   => diary_edit(rt, &cmd),
        Some("list")   => diary_list(rt, &cmd),
        Some(other)    => {
            // cannot happen due to clap
            unreachable!()
        },
        None => {
            warn!("No diary subcommand, falling back to 'list'");
            diary_list(rt, &cmd)
        },
    }
}

fn diary_create(rt: &Runtime, cmd: &ArgMatches) {
    let name = cmd.subcommand_matches("create").unwrap().value_of("name");
    if name.is_none() {
        warn!("No diary name");
        exit(1);
    }
    let name = String::from(name.unwrap());

    Diary::new(rt.store(), name.clone(), String::from(""))
        .map(|diary| {
            debug!("Diary created: {:?}", name);
            info!("Ok");
        })
        .map_err(|e| {
            trace_error(&e);
            warn!("Error");
        });
}

fn diary_delete(rt: &Runtime, cmd: &ArgMatches) {
    let name = cmd.subcommand_matches("delete").unwrap().value_of("name");
    if name.is_none() {
        warn!("No diary name");
        exit(1);
    }
    let name = String::from(name.unwrap());

    Diary::delete(rt.store(), name.clone())
        .map(|diary| {
            debug!("Diary deleted: {:?}", name);
            info!("Ok");
        })
        .map_err(|e| {
            trace_error(&e);
            warn!("Error");
        });
}

fn diary_edit(rt: &Runtime, cmd: &ArgMatches) {
    let name = cmd.subcommand_matches("edit").unwrap().value_of("name");
    if name.is_none() {
        warn!("No diary name");
        exit(1);
    }
    let name = name.unwrap();

    unimplemented!()
}

fn diary_list(rt: &Runtime, cmd: &ArgMatches) {
    unimplemented!()
}

fn get_diary_name(rt: &Runtime) -> Option<String> {
    use libimagdiary::config::get_default_diary_name;

    get_default_diary_name(rt)
        .or(rt.cli().value_of("diaryname").map(String::from))
}
