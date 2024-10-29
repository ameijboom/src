use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::{Config, Repository};

use crate::{
    cmd::{add::add_callback, branch::create_branch_checkout},
    git::{commit::Commit, index::Index},
    utils,
};

#[derive(Parser)]
#[clap(about = "Record changes to the repository")]
pub struct Opts {
    #[clap(short, long, help = "Add all changes")]
    add_all: bool,

    #[clap(short, long, help = "Create a branch")]
    branch: bool,

    #[clap(help = "Commit message")]
    pub message: String,
}

fn branch_name(message: &str) -> String {
    if let Some((prefix, name)) = message.split_once(':') {
        return format!(
            "{}/{}",
            prefix.trim().replace(' ', "-"),
            name.trim().replace(' ', "-")
        );
    }

    message.trim().replace(' ', "-")
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.branch {
        create_branch_checkout(&repo, &branch_name(&opts.message))?;
    }

    let mut index = Index::build(&repo)?;

    if opts.add_all {
        index.add(["."].iter(), add_callback)?;
        index.write()?;
    }

    let tree = index.write_tree()?;
    let config = Config::open_default()?;

    let commit = Commit::build(&config, &repo, tree);
    let oid = commit.create(&opts.message, None, None)?;

    repo.head()?
        .set_target(oid, &format!("commit: {}", opts.message))?;

    println!("Created {}", utils::short(&oid).yellow());

    Ok(())
}
