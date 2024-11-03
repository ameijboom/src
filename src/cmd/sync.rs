use std::error::Error;

use clap::Parser;

use crate::git::{Ref, RemoteOpts, Repo};

#[derive(Parser)]
#[clap(about = "Synchronize changes")]
pub struct Opts {}

pub fn run(repo: Repo, _opts: Opts) -> Result<(), Box<dyn Error>> {
    // Find remote default branch
    let refname = {
        let mut remote = repo.find_remote("origin")?;
        remote.connect(RemoteOpts::default())?;
        remote.default_branch()?
    };

    let branch = refname.trim_start_matches("refs/heads/");

    // Checkout local branch with the same name
    let branch = repo.find_branch(branch)?;
    let reference: Ref<'_> = branch.into();

    repo.checkout(&reference)?;
    drop(reference);

    // Pull the latest changes
    super::pull::run(repo, super::pull::Opts::default())
}
