#[macro_use] extern crate log;
extern crate clap;

extern crate libimagrt;
extern crate libimagdiary;
extern crate libimagutil;

use std::process::exit;

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

    rt.cli()
        .subcommand_name()
        .map(|name| {
            debug!("Call {}", name);
            match name {
                "create" => create(&rt),
                // "delete" => delete(&rt),
                // "edit" => edit(&rt),
                "list" => list(&rt),
                "diary" => diary(&rt),
                _        => {
                    debug!("Unknown command"); // More error handling
                },
            }
        })
}

fn create(rt: &Runtime) {
    let diaryname = get_diary_name(rt);
    if diaryname.is_none() {
        warn!("No diary selected. Use either the configuration file or the commandline option");
        exit(1);
    }
    let diaryname = diaryname.unwrap();

    let prevent_edit = rt.cli().subcommand_matches("create").unwrap().is_present("no-edit");

    Diary::retrieve(diaryname)
        .and_then(|diary| diary.new_entry(rt))
        .and_then(|entry| {
            if prevent_edit {
                debug!("Not editing new diary entry");
            } else {
                debug!("Editing new diary entry");
                entry.edit();
            }
        })

    unimplemented!()
}

fn delete(rt: &Runtime) {
    unimplemented!()
}

fn edit(rt: &Runtime) {
    unimplemented!()
}

fn list(rt: &Runtime) {
    unimplemented!()
}

fn diary(rt: &Runtime) {
    unimplemented!()
}


fn get_diary_name(rt: &Runtime) -> Option<String> {
    use libimagdiary::config::get_default_diary_name;

    get_default_diary_name(rt)
        .or(rt.cli().value_of("diaryname").map(String::from))
}
