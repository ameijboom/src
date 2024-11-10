use std::error::Error;

use clap::Parser;

use crate::git::{RemoteOpts, Repo};

#[derive(Parser)]
#[clap(about = "Download objects and refs")]
pub struct Opts {}

pub fn run(repo: Repo, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let branch = head.shorthand()?;

    let branch = repo.find_branch(branch)?;
    let upstream = branch.upstream()?;
    let remote = upstream.remote_name()?;

    let mut remote = repo.find_remote(remote)?;
    remote.fetch(RemoteOpts::default(), branch.name()?)?;

    Ok(())
}
