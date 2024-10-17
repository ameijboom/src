use std::error::Error;

use clap::Parser;
use git2::{Config, Repository, Signature};

#[derive(Parser)]
pub struct Opts {
    #[clap()]
    message: String,
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;
    let config = Config::open_default()?;
    let name = config.get_string("user.name")?;
    let email = config.get_string("user.email")?;
    let author = Signature::now(&name, &email)?;

    repo.commit(
        Some("HEAD"),
        &author,
        &author,
        &opts.message,
        &tree,
        &[&head.peel_to_commit()?],
    )?;

    Ok(())
}
