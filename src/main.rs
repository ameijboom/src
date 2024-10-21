use std::{io, path::PathBuf};

use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Shell};
use colored::Colorize;
use git2::Repository;

mod callbacks;
mod cmd;
mod named;
mod select;
mod utils;

#[derive(Parser)]
struct Opts {
    #[clap(short, long, default_value = ".", value_hint = ValueHint::DirPath)]
    dir: PathBuf,

    #[clap(subcommand)]
    cmd: Option<Cmd>,

    #[clap(help = "Branch name to checkout")]
    branch: Option<String>,

    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,
}

#[derive(Parser)]
enum Cmd {
    Add(cmd::add::Opts),
    Commit(cmd::commit::Opts),
    Push(cmd::push::Opts),
    Fetch(cmd::fetch::Opts),
    Pull(cmd::pull::Opts),
    List(cmd::list::Opts),
    Diff(cmd::diff::Opts),
    Checkout(cmd::checkout::Opts),
}

fn main() {
    let opts = Opts::parse();

    if let Some(generator) = opts.generator {
        let mut cmd = Opts::command();
        let bin_name = cmd.get_name().to_string();
        generate(generator, &mut cmd, bin_name, &mut io::stdout());
        return;
    }

    let app = || {
        let repo = Repository::open(opts.dir)?;

        match opts.cmd {
            Some(Cmd::Add(opts)) => cmd::add::run(repo, opts),
            Some(Cmd::Commit(opts)) => cmd::commit::run(repo, opts),
            Some(Cmd::Push(opts)) => cmd::push::run(repo, opts),
            Some(Cmd::Fetch(opts)) => cmd::fetch::run(repo, opts),
            Some(Cmd::Pull(opts)) => cmd::pull::run(repo, opts),
            Some(Cmd::List(opts)) => cmd::list::run(repo, opts),
            Some(Cmd::Diff(opts)) => cmd::diff::run(repo, opts),
            Some(Cmd::Checkout(opts)) => cmd::checkout::run(repo, opts),
            None => match opts.branch {
                Some(branch) => cmd::checkout::run(repo, cmd::checkout::Opts::with_branch(branch)),
                None => cmd::status::run(repo),
            },
        }
    };

    if let Err(e) = app() {
        eprintln!("{}", format!("⚠️ {e}").red());
    }
}
