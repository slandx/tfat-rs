use clap::{App, Arg, ArgMatches, SubCommand};
use config;
use std::io::{self, Write};

// Create arguments for `delete` subcommand
pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("delete")
        .about("Delete an account")
        .arg(
            Arg::with_name("account")
                .required(true)
                .help("Name of the account"),
        )
}

// Implementation for the `delete` subcommand
pub fn run(args: &ArgMatches) {
    let mut cfg = match config::read_from_file() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let account_name = args.value_of("account").unwrap();
    if !cfg.accounts.contains_key(account_name) {
        eprintln!("Account {} is not found!", account_name);
        return;
    }
    print!("Are you sure you want to delete {} [N/y]? ", account_name);
    io::stdout().flush().unwrap();
    let mut answer = String::new();
    match io::stdin().read_line(&mut answer) {
        Ok(_) => {
            if answer.trim().to_lowercase() == "y" {
                cfg.accounts.remove(account_name);
                match config::save_to_file(&cfg) {
                    Ok(ret) => println!("Account {} {}.", account_name,
                                        if ret { "has been deleted" } else { "deleted failed" }),
                    Err(err) => {
                        eprintln!("{}", err);
                        return;
                    }
                }
            } else {
                println!("Abort.");
            }
        }
        Err(_) => eprintln!("Failed to read input"),
    };
}
