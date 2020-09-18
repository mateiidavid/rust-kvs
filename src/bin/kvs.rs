use clap::{App, Arg};
use kvs::Result;
//TODO: use structopt
fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            App::new("get")
                .about("Get the string value of given key")
                .arg(Arg::with_name("KEY").required(true)),
        )
        .subcommand(
            App::new("set")
                .about("Set a key to a value")
                .arg(Arg::with_name("KEY").required(true))
                .arg(Arg::with_name("VALUE").required(true)),
        )
        .subcommand(
            App::new("rm")
                .about("Remove a given key")
                .arg(Arg::with_name("KEY").required(true)),
        )
        .get_matches();

    match matches.subcommand_name() {
        Some("get") => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some("set") => {
            let matches = matches.subcommand_matches("set").unwrap();
            let key = matches.value_of("KEY").unwrap();
            let val = matches.value_of("VALUE").unwrap();
            let current_dir = std::env::current_dir()?;
            kvs::KvStore::open(current_dir)?.set(key.to_owned(), val.to_owned())?;
            println!("Set {} to value {}", key, val);
        }
        Some("rm") => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        _ => {
            std::process::exit(1);
        }
    }

    Ok(())
}
