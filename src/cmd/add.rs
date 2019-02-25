use clap::{App, Arg, ArgMatches, SubCommand};
use config;
use data_encoding::BASE32_NOPAD;

// Create arguments for `add` subcommand
pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("add")
        .about("Add a new account")
        .arg(
            Arg::with_name("account")
                .required(true)
                .help("Name of the account"),
        )
        .arg(
            Arg::with_name("key")
                .required(true)
                .help("Secret key of the OTP")
                .validator(is_base32_key),
        )
}

// Validate key provided in arguments is a valid base32 encoding
fn is_base32_key(value: String) -> Result<(), String> {
    let value = value.to_uppercase();
    match BASE32_NOPAD.decode(value.as_bytes()) {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("the key is not a valid base32 encoding")),
    }
}

pub fn run(args: &ArgMatches) {
    let mut cfg = match config::read_from_file() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    if cfg.empty && config::init_pwd(&mut cfg).is_err() {
        eprintln!("Wrong password!");
        return;
    }

    let account_name = args.value_of("account").unwrap();
    let key = args.value_of("key").unwrap().to_uppercase();

    cfg.accounts.insert(account_name.to_string(), key);
    match config::save_to_file(&cfg) {
        Ok(ret) => println!("Added {}", if ret { "successfully" } else { "failed" }),
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    }
}
