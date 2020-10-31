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

    match matches.subcommand() {
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let current_dir = std::env::current_dir()?;
            let mut store = kvs::KvStore::open(current_dir)?;
            if let Some(v) = store.get(key.to_string())? {
                println!("{}", v);
            } else {
                println!("{}", "Key not found")
            }
        }
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let val = matches.value_of("VALUE").unwrap();
            let current_dir = std::env::current_dir()?;
            let mut store = kvs::KvStore::open(current_dir)?;
            store.set(key.to_string(), val.to_string())?;
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let current_dir = std::env::current_dir()?;
            let mut store = kvs::KvStore::open(current_dir)?;
            match store.remove(key.to_string()) {
                Ok(()) => {}
                Err(kvs::KvsError::Store(kvs::ErrorKind::NotFound)) => {
                    println!("{}", "Key not found");
                    std::process::exit(1)
                }

                Err(e) => return Err(e),
            }
        }
        _ => {
            std::process::exit(1);
        }
    }

    Ok(())
}
