#[macro_use] extern crate log;
extern crate clap;

extern crate libimagrt;
extern crate libimagdiary;
extern crate libimagutil;

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
}
