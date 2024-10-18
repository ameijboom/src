use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;
use git2::Repository;

mod add;
mod commit;
mod push;
mod status;
mod utils;

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = ".")]
    dir: PathBuf,

    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Parser)]
enum Cmd {
    Add(add::Opts),
    Commit(commit::Opts),
    Push(push::Opts),
}

fn main() {
    let opts = Opts::parse();
    let app = || {
        let repo = Repository::open(opts.dir)?;

        match opts.cmd {
            Some(Cmd::Add(opts)) => add::run(repo, opts),
            Some(Cmd::Commit(opts)) => commit::run(repo, opts),
            Some(Cmd::Push(opts)) => push::run(repo, opts),
            None => status::run(repo),
        }
    };

    if let Err(e) = app() {
        eprintln!("{}", format!("⚠️ {e}").red());
    }
}
