# tfat

Two factor authentication(2FA) command line tool.

Written with `Rust` for learning programming language, see `Go` version [here](https://github.com/slandx/tfat).

## Features

- Save config file with AES-GCM encrypt
- Save data with password.
- Generate random password if user set an empty password
- Auto refresh code per seconds
- Copy the code to clipboard if it generate a new one

## Build

```shell
cargo build --release
```

## Usage

### Get code

```bash
tfat
# output:
# 602887 (remain 5s)
```

This will keep refreshing and count down the remaining time.
When a new code is generated, it will copy to clipboard automatically.
Press `Ctrl+C` to quit.

```bash
tfat
# output:
# 1. one
# 2. two
# 3. three
# select: 1
# 698030 (remain 16s)
```

If there are more than one account, it will list all of them.
You can select any one you like to check its code.

### Add an account

```bash
tfat add <ACCOUNT> <SECRET>
```

If it is the first time to add an account, it will ask you to set a password.
Of course you can set it with an empty string, it still gonna generate a random password to protect your data.
In that case when you get code, it will decrypt automatically without asking password from you.
So anyone who use your computer may get a code from it. It is recommended to set a password manually.

### delete an account

```bash
tfat delete <ACCOUNT>
```

### Change password

```bash
tfat password
```

### Help

```bash
$ tfat help
tfat 0.1.0
Two Factor Authentication(2FA) Tool - Help to generate 2FA code

USAGE:
    tfat [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    add         Add a new account
    delete      Delete an account
    help        Prints this message or the help of the given subcommand(s)
    password    Change password
```
