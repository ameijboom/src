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
}

fn is_signed(commit: &Commit) -> bool {
    commit
        .header_field_bytes("gpgsig")
        .map(|sig| !sig.is_empty())
        .unwrap_or(false)
}

fn check_write(result: Result<(), io::Error>) -> Result<(), io::Error> {
    match result {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e),
    }
}

fn _run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    for oid in revwalk {
        let id = oid?;
        let commit = repo.find_commit(id)?;
        let created_at = git::parse_local_time(commit.time());
        let signed = if is_signed(&commit) {
            "⚿ ".green()
        } else if opts.short {
            "  ".white()
        } else {
            "".white()
        };
        let message = commit.message().unwrap_or_default().trim();

        if opts.short {
            check_write(writeln!(
                stdout,
                "{signed}{} {}",
                utils::short(&id).yellow(),
                message.split('\n').next().unwrap_or_default()
            ))?;
        } else {
            check_write(writeln!(stdout, "{signed}{}", id.to_string().yellow()))?;
            check_write(write!(
                stdout,
                "{}\n{}\n\n",
                format!("Date: {}", created_at.format("%Y-%m-%d %H:%M")).bright_black(),
                format!("Author: {}", commit.author()).bright_black(),
            ))?;
            check_write(writeln!(
                stdout,
                "{}\n",
                message
                    .lines()
                    .map(|l| format!("  {l}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))?;
        }
    }

    Ok(())
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
