use clipboard::{ClipboardContext, ClipboardProvider};
use config;
use std::{thread, time};
use std::io::{self, Write};
use totp::TOTP;

pub fn run(in_loop: bool) {
    let cfg = match config::read_from_file() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };
    if cfg.accounts.len() <= 0 {
        eprintln!("Add an account before viewing");
        return;
    }
    for (i, account) in cfg.accounts.iter().enumerate() {
        println!("{}. {}", i + 1, account.0);
    }
    let mut act_idx: usize = 1;
    while cfg.accounts.len() > 1 {
        let mut input_text = String::new();
        print!("select: ");
        io::stdout().flush().unwrap();
        if io::stdin().read_line(&mut input_text).is_err() {
            eprintln!("Failed to read input");
            return;
        }

        act_idx = match input_text.trim().parse::<usize>() {
            Ok(i) => i,
            Err(..) => {
                println!("Invalid number: {}", input_text);
                std::usize::MAX
            }
        };
        if act_idx > 0 && act_idx <= cfg.accounts.len() {
            break;
        } else {
            println!("Number should be in {}..{}", 1, cfg.accounts.len());
        }
    }

    match cfg.accounts.iter().nth(act_idx - 1) {
        Some(account) => {
            let (_, secret) = account;
            match TOTP::new(secret, None) {
                Ok(otp) => {
                    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                    let mut last_code = String::new();
                    loop {
                        let (code, remain) = otp.generate();
                        if last_code.ne(&code) {
                            last_code.clone_from(&code);
                            ctx.set_contents(code.to_owned()).unwrap();
                        }
                        print!("\r{:06} (remain {}s) ", code, remain);
                        if !in_loop {
                            println!();
                            break;
                        }
                        io::stdout().flush().unwrap();
                        thread::sleep(time::Duration::from_secs(1));
                    }
                }
                Err(err) => eprintln!("{}", err),
            }
        }
        None => println!("Account at {} is not found.", act_idx),
    }
}
