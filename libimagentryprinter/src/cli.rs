use clap::{Arg, App, SubCommand};

/**
 * Build a commandline interface subcommand to configure the printer backend
 */
pub fn build_cli<'a>() -> App<'a, 'a> {
    SubCommand::with_name("printer-config")
        .about("Configure the printer for printing entries")
        .version("0.1")
        .subcommand(build_plain_printer_cli())
        .subcommand(build_abbrev_printer_cli())
        .subcommand(build_table_printer_cli())
}

/**
 * Build a commandline interface subcommand to configure the printer backend "plain",
 * which is supposed to print the entries as-is
 */
fn build_plain_printer_cli<'a>() -> App<'a, 'a> {
    unimplemented!()
}

/**
 * Build a commandline interface subcommand to configure the printer backend "plain",
 * which is supposed to print the entries with only certain header fields and content abbreviated
 */
fn build_abbrev_printer_cli<'a>() -> App<'a, 'a> {
    unimplemented!()
}


/**
 * Build a commandline interface subcommand to configure the printer backend "plain",
 * which is supposed to print the entries in an ASCII-Table where the shown columns can be
 * configured
 */
fn build_table_printer_cli<'a>() -> App<'a, 'a> {
    unimplemented!()
}

