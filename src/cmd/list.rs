use std::{
    error::Error,
    io::{self, ErrorKind, Write},
};

use clap::Parser;
use colored::{Color, Colorize};
use pager::Pager;

use crate::{
    git::{Commit, Repo},
    utils,
};

#[derive(Parser)]
#[clap(about = "Show commit logs")]
pub struct Opts {
    #[clap(long, short, help = "Show logs in one line without metadata")]
    short: bool,

    #[clap(long, help = "Disable the pager")]
    no_pager: bool,

    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Parser)]
pub enum Cmd {
    #[clap(about = "List stashes")]
    Stash,
}

macro_rules! check_writeln {
    ($dst:expr, $($arg:tt)*) => {
        match std::writeln!($dst, $($arg)*) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    };
}

fn list_commits<'a>(
    walk: impl Iterator<Item = Result<Commit<'a>, git2::Error>>,
    short: bool,
    mut stdout: impl Write,
) -> Result<(), Box<dyn Error>> {
    for commit in walk {
        let commit = commit?;
        let signed = if commit.is_signed() {
            "⚿ ".green()
        } else if short {
            "  ".white()
        } else {
            "".white()
        };
        let message = commit.message().unwrap_or_default().trim();

        if short {
            check_writeln!(
                stdout,
                "{signed}{} {}",
                utils::short_hash(commit.id()).yellow(),
                message.split('\n').next().unwrap_or_default()
            )?;
        } else {
            check_writeln!(stdout, "{signed}{}", commit.id().to_string().yellow())?;
            check_writeln!(
                stdout,
                "{}\n",
                commit.headers_formatted().with_color(Color::BrightBlack)
            )?;
            check_writeln!(stdout, "{}\n", commit.message_formatted())?;
        }
    }

    Ok(())
}

fn _run(mut repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();

    match opts.cmd {
        Some(cmd) => match cmd {
            Cmd::Stash => {
                let stashes = repo.stashes()?;
                list_commits(stashes, opts.short, &mut stdout)
            }
        },
        None => {
            let commits = repo.commits()?;
            list_commits(commits, opts.short, &mut stdout)
        }
    }
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.no_pager {
        _run(repo, opts)
    } else {
        colored::control::set_override(true);
        Pager::with_default_pager("less -R").setup();
        _run(repo, opts)
    }
}
