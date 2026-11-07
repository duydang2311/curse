use clap::Command;

fn cli() -> Command {
    Command::new("curse-daemon")
        .about("curse music player daemon")
        .version("v0.0.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("start").about("Start the daemon (detached)"))
        .subcommand(Command::new("stop").about("Stop the daemon"))
        .subcommand(Command::new("run").about("Start and run the daemon in foreground"))
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", _)) => {
            println!("start");
        }
        Some(("stop", _)) => {
            println!("stop");
        }
        Some(("run", _)) => {
            println!("run");
        }
        _ => unreachable!(),
    }
}
