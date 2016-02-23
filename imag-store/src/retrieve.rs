use std::path::PathBuf;
use std::ops::Deref;
use std::fmt::Display;

use clap::ArgMatches;
use toml::Value;

use libimagstore::store::FileLockEntry;
use libimagrt::runtime::Runtime;

use util::build_entry_path;

pub fn retrieve(rt: &Runtime) {
    rt.cli()
        .subcommand_matches("retrieve")
        .map(|scmd| {
            let path = scmd.value_of("id").map(|id| build_entry_path(rt, id)).unwrap();
            trace!("path = {:?}", path);
            rt.store()
                // "id" must be present, enforced via clap spec
                .retrieve(path)
                .map(|e| print_entry(rt, scmd, e))
                .map_err(|e| {
                    debug!("No entry.");
                    trace!("{}", e);
                })

        });
}

fn print_entry(rt: &Runtime, scmd: &ArgMatches, e: FileLockEntry) {
    if do_print_raw(scmd) {
        debug!("Printing raw content...");
        println!("{}", e.deref().to_str());
    } else if do_filter(scmd) {
        debug!("Filtering...");
        warn!("Filtering via header specs is currently now supported.");
        warn!("Will fail now!");
        unimplemented!()
    } else {
        debug!("Printing structured...");
        let entry = e.deref();
        if do_print_header(scmd) {
            debug!("Printing header...");
            if do_print_header_as_json(rt.cli()) {
                debug!("Printing header as json...");
                warn!("Printing as JSON currently not supported.");
                warn!("Will fail now!");
                unimplemented!()
            } else {
                debug!("Printing header as TOML...");
                // We have to Value::Table() for Display
                println!("{}", Value::Table(entry.get_header().clone().into()))
            }
        }

        if do_print_content(scmd) {
            debug!("Printing content...");
            println!("{}", entry.get_content());
        }

    }
}

fn do_print_header(m: &ArgMatches) -> bool {
    m.is_present("header")
}

fn do_print_header_as_json(m: &ArgMatches) -> bool {
    m.is_present("header-json")
}

fn do_print_content(m: &ArgMatches) -> bool {
    m.is_present("content")
}

fn do_print_raw(m: &ArgMatches) -> bool {
    m.is_present("raw")
}

fn do_filter(m: &ArgMatches) -> bool {
    m.subcommand_matches("filter-header").is_some()
}

