use std::error::Error;

use clap::Parser;

use crate::{git::Repo, utils};

#[derive(Parser)]
#[clap(about = "Stash the changes in a dirty working directory away")]
pub struct Opts {}

pub fn run(mut repo: Repo, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let message = {
        let head = repo.head()?;
        let commit = head.find_commit()?;
        let message = commit.message().unwrap_or_default();

        format!("{} {message}", utils::short(&commit.id()))
    };

    repo.save_stash(&message)?;

    println!("✓ Changes stashed");

    Ok(())
}
