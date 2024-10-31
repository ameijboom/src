use std::error::Error;

use clap::Parser;
use git2::{Config, Repository, StashFlags};

use crate::{git, utils};

#[derive(Parser)]
#[clap(about = "Stash the changes in a dirty working directory away")]
pub struct Opts {}

pub fn run(mut repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;
    let message = commit.message().unwrap_or_default();
    let message = format!("{} {message}", utils::short(&commit.id()));

    drop(head);
    drop(commit);

    let config = Config::open_default()?;
    let signature = git::signature(&config)?;

    repo.stash_save(&signature, &message, Some(StashFlags::INCLUDE_UNTRACKED))?;

    println!("✓ Changes stashed");

    Ok(())
}
