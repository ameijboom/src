use std::error::Error;

use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use colored::Colorize;
use git2::{Commit, Repository};

use crate::utils;

#[derive(Parser)]
#[clap(about = "Show commit logs")]
pub struct Opts {
    #[clap(long, short, default_value = "false")]
    short: bool,

    #[clap(long, short, default_value = "20")]
    limit: usize,
}

fn is_signed(commit: &Commit) -> bool {
    commit
        .header_field_bytes("gpgsig")
        .map(|sig| !sig.is_empty())
        .unwrap_or(false)
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    for oid in revwalk.take(opts.limit) {
        let id = oid?;
        let commit = repo.find_commit(id)?;
        let created_at = DateTime::from_timestamp(commit.time().seconds(), 0)
            .map(|dt| dt.naive_local())
            .map(|dt| Local.from_utc_datetime(&dt))
            .unwrap_or_default();
        let sig = commit.author();
        let author_name = sig.name().unwrap_or("<unknown>");
        let author = match sig.email() {
            Some(email) => format!("{} <{}>", author_name, email),
            None => author_name.to_owned(),
        };
        let signed = if is_signed(&commit) {
            "⚿ ".green()
        } else if opts.short {
            "  ".white()
        } else {
            "".white()
        };

        println!(
            "{signed}{} {}",
            utils::short(&id).yellow(),
            commit.message().unwrap_or_default().trim()
        );

        if !opts.short {
            println!(
                "{}\n{}\n",
                format!("Date: {}", created_at.format("%Y-%m-%d %H:%M")).black(),
                format!("Author: {}", author).black(),
            );
        }
    }

    Ok(())
}
