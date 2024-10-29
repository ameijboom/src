use std::{io, path::PathBuf};

use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Shell};
use colored::Colorize;
use git2::{Repository, RepositoryOpenFlags};

mod callbacks;
mod cmd;
mod git;
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
    Feat(cmd::commit::Opts),
    Fix(cmd::commit::Opts),
    Commit(cmd::commit::Opts),
    Amend(cmd::amend::Opts),
    Push(cmd::push::Opts),
    Fetch(cmd::fetch::Opts),
    Pull(cmd::pull::Opts),
    List(cmd::list::Opts),
    Diff(cmd::diff::Opts),
    Stash(cmd::stash::Opts),
    Branch(cmd::branch::Opts),
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
        let repo = Repository::open_ext(&opts.dir, RepositoryOpenFlags::empty(), [&opts.dir])?;

        match opts.cmd {
            Some(Cmd::Add(opts)) => cmd::add::run(repo, opts),
            Some(Cmd::Feat(mut opts)) => {
                opts.message = format!("feat: {}", opts.message);
                cmd::commit::run(repo, opts)
            }
            Some(Cmd::Fix(mut opts)) => {
                opts.message = format!("fix: {}", opts.message);
                cmd::commit::run(repo, opts)
            }
            Some(Cmd::Commit(opts)) => cmd::commit::run(repo, opts),
            Some(Cmd::Amend(opts)) => cmd::amend::run(repo, opts),
            Some(Cmd::Push(opts)) => cmd::push::run(repo, opts),
            Some(Cmd::Fetch(opts)) => cmd::fetch::run(repo, opts),
            Some(Cmd::Pull(opts)) => cmd::pull::run(repo, opts),
            Some(Cmd::List(opts)) => cmd::list::run(repo, opts),
            Some(Cmd::Diff(opts)) => cmd::diff::run(repo, opts),
            Some(Cmd::Stash(opts)) => cmd::stash::run(repo, opts),
            Some(Cmd::Branch(opts)) => cmd::branch::run(repo, opts),
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
