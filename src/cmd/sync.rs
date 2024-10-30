use std::error::Error;

use clap::Parser;
use git2::{BranchType, Direction, Repository};
use indicatif::ProgressBar;

use crate::{callbacks::remote_callbacks, cmd::checkout::try_checkout, named::Named};

#[derive(Parser)]
#[clap(about = "Synchronize changes")]
pub struct Opts {}

pub fn run(repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    // Find remote default branch
    let mut remote = repo.find_remote("origin")?;
    let mut bar = ProgressBar::new_spinner();
    let mut stdout = vec![];

    bar.set_message("Finding branch");

    remote.connect_auth(
        Direction::Fetch,
        Some(remote_callbacks(&mut stdout, &mut bar)),
        None,
    )?;

    let refname = remote.default_branch()?;
    let branch = std::str::from_utf8(&refname)?.trim_start_matches("refs/heads/");

    // Checkout local branch with the same name
    let branch = repo.find_branch(branch, BranchType::Local)?;
    let reference = branch.into_reference();
    let refname = reference.name_checked()?.to_string();

    if !try_checkout(&repo, reference)? {
        return Err("checkout failed".into());
    }

    repo.set_head(&refname)?;

    drop(remote);

    bar.finish_and_clear();

    // Pull the latest changes
    super::pull::run(repo, super::pull::Opts::default())
}
