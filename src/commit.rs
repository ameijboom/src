use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::{Config, IndexAddOption, Repository, Signature};
use sha1::{Digest, Sha1};

use crate::add::add_callback;

#[derive(Parser)]
pub struct Opts {
    #[clap(short, long)]
    add_all: bool,

    #[clap()]
    message: String,
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut index = repo.index()?;

    if opts.add_all {
        index.add_all(
            ["."].iter(),
            IndexAddOption::DEFAULT,
            Some(&mut add_callback),
        )?;
    }

    let head = repo.head()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let config = Config::open_default()?;
    let name = config.get_string("user.name")?;
    let email = config.get_string("user.email")?;
    let author = Signature::now(&name, &email)?;

    let oid = repo.commit(
        Some("HEAD"),
        &author,
        &author,
        &opts.message,
        &tree,
        &[&head.peel_to_commit()?],
    )?;
    let mut hasher = Sha1::new();
    hasher.update(oid);
    let digest = hasher.finalize();

    println!("Created {}", hex::encode(digest).black());

    Ok(())
}
