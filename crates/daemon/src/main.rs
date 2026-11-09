use clap::Command;

mod app;
mod audio;
mod cmds;
mod listener;

fn cli() -> Command {
    Command::new("curse-daemon")
        .about("curse music player daemon")
        .version("v0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("start").about("Start the daemon (detached)"))
        .subcommand(Command::new("stop").about("Stop the daemon"))
        .subcommand(Command::new("run").about("Start and run the daemon in foreground"))
}

fn main() -> eyre::Result<()> {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", _)) => cmds::start()?,
        Some(("stop", _)) => cmds::stop()?,
        Some(("run", _)) => cmds::run()?,
        _ => unreachable!(),
    }
    Ok(())
}
