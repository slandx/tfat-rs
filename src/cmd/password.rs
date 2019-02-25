use clap::{App, SubCommand};
use config;

// `password` subcommand
pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("password")
        .about("Change password")
}

// Implementation for the `password` subcommand
pub fn run() {
    let mut cfg = match config::read_from_file() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    if config::init_pwd(&mut cfg).is_err() {
        eprintln!("Wrong password!");
        return;
    }

    match config::save_to_file(&cfg) {
        Ok(ret) => println!("Change password {}", if ret { "successfully" } else { "failed" }),
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    }
}

