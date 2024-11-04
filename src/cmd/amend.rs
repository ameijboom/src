use std::error::Error;

use clap::Parser;
use colored::Colorize;

use crate::{cmd::add::add_callback, git::Repo, utils};

#[derive(Parser)]
#[clap(about = "Amend recorded changes to the repository")]
pub struct Opts {
    #[clap(short, long, help = "Add all changes")]
    add_all: bool,

    #[clap(help = "Commit message")]
    message: Option<String>,
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut index = repo.index()?;

    if opts.add_all {
        index.add(["."], add_callback)?;
        index.write()?;
    }

    let oid = index.write_tree()?;
    let mut head = repo.head()?;
    let (reflog, oid) = {
        let tree = repo.find_tree(oid)?;
        let latest = head.find_commit()?;
        let parent = latest.parent()?.ok_or("unable to amend empty commit")?;
        let message = opts
            .message
            .as_deref()
            .map(Ok)
            .unwrap_or_else(|| latest.message())?;

        (
            format!("commit amended: {message}"),
            repo.create_commit(&tree, message, Some(&parent))?,
        )
    };

    head.set_target(oid, &reflog)?;

    println!("Created {}", utils::short_hash(oid).yellow());

    Ok(())
}
