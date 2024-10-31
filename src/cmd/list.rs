use std::{
    error::Error,
    io::{self, ErrorKind, Write},
};

use clap::Parser;
use colored::Colorize;
use git2::{Commit, Repository};
use pager::Pager;

use crate::{git, utils};

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

fn is_signed(commit: &Commit) -> bool {
    commit
        .header_field_bytes("gpgsig")
        .map(|sig| !sig.is_empty())
        .unwrap_or(false)
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

fn list_commits(
    repo: &Repository,
    walk: impl Iterator<Item = Result<git2::Oid, git2::Error>>,
    short: bool,
    mut stdout: impl Write,
) -> Result<(), Box<dyn Error>> {
    for oid in walk {
        let id = oid?;
        let commit = repo.find_commit(id)?;
        let created_at = git::parse_local_time(commit.time());
        let signed = if is_signed(&commit) {
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
                utils::short(&id).yellow(),
                message.split('\n').next().unwrap_or_default()
            )?;
        } else {
            check_writeln!(stdout, "{signed}{}", id.to_string().yellow())?;
            check_writeln!(
                stdout,
                "{}\n{}\n",
                format!("Date: {}", created_at.format("%Y-%m-%d %H:%M")).bright_black(),
                format!("Author: {}", commit.author()).bright_black(),
            )?;
            check_writeln!(
                stdout,
                "{}\n",
                message
                    .lines()
                    .map(|l| format!("  {l}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
    }

    Ok(())
}

fn _run(mut repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();

    match opts.cmd {
        Some(cmd) => match cmd {
            Cmd::Stash => {
                let mut stashes = vec![];

                repo.stash_foreach(|_, _, oid| {
                    stashes.push(*oid);
                    true
                })?;

                list_commits(&repo, stashes.into_iter().map(Ok), opts.short, &mut stdout)
            }
        },
        None => {
            let mut revwalk = repo.revwalk()?;
            revwalk.push_head()?;

            list_commits(&repo, revwalk, opts.short, &mut stdout)
        }
    }
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.no_pager {
        _run(repo, opts)
    } else {
        colored::control::set_override(true);
        Pager::with_default_pager("less -R").setup();
        _run(repo, opts)
    }
}
