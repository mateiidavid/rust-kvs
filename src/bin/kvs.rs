use clap::{App, Arg};
fn main() {
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
                                        .arg(Arg::with_name("KEY").required(true))
                                  )
                                  .get_matches();

    match matches.subcommand_name() {
        Some("get") => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some("set") => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        Some("rm") => {
            eprintln!("unimplemented");
            std::process::exit(1);
        }
        _ => {
          std::process::exit(1);
        }
    }
}
