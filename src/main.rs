use std::path::PathBuf;

use clap::Parser;
use colored::Colorize;
use git2::Repository;

mod status;

macro_rules! print {
    ($($arg:tt)*) => {
        println!("{}", $($arg)*);
    };
}

pub(crate) use print;

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = ".")]
    dir: PathBuf,

    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Parser)]
enum Cmd {}

fn main() {
    let opts = Opts::parse();
    let app = || {
        let repo = Repository::open(opts.dir)?;

        match opts.cmd {
            Some(_) => todo!(),
            None => status::run(repo),
        }
    };

    if let Err(e) = app() {
        eprintln!("{}", format!("⚠️ {e}").red());
    }
}
