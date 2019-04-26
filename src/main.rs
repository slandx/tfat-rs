extern crate argon2;
#[macro_use]
extern crate clap;
extern crate clipboard;
extern crate data_encoding;
extern crate dirs;
#[macro_use]
extern crate failure;
extern crate ring;
extern crate rpassword;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use clap::{App, Arg};

mod error;
mod cmd;
mod kdf_util;
mod config;
mod totp;

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(Arg::with_name("loop")
            .short("l")
            .long("loop")
            .help("Generate code in loop"))
        .subcommand(cmd::add::subcommand())
        .subcommand(cmd::delete::subcommand())
        .subcommand(cmd::password::subcommand())
        .get_matches();

    match matches.subcommand() {
        ("add", Some(sub_m)) => cmd::add::run(&sub_m),
        ("delete", Some(sub_m)) => cmd::delete::run(&sub_m),
        ("password", Some(_)) => cmd::password::run(),
        _ => cmd::view::run(matches.occurrences_of("loop") > 0),
    }
}
