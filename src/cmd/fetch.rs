use std::error::Error;

use clap::Parser;
use git2::{BranchType, FetchOptions, Repository};
use indicatif::ProgressBar;

use crate::{callbacks::remote_callbacks, named::Named, utils};

#[derive(Parser)]
#[clap(about = "Download objects and refs")]
pub struct Opts {}

pub fn run(repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut stdout = vec![];
    let mut bar = ProgressBar::new_spinner();

    let head = repo.head()?;
    let Some(branch) = head.shorthand() else {
        return Err("invalid name for HEAD".into());
    };

    let branch = repo.find_branch(branch, BranchType::Local)?;
    let upstream = branch.upstream()?;
    let remote = utils::parse_remote(upstream.name_checked()?)?;

    let mut remote = repo.find_remote(remote)?;
    let callbacks = remote_callbacks(&mut stdout, &mut bar);

    remote.fetch(
        &[branch.name_checked()?],
        Some(FetchOptions::new().remote_callbacks(callbacks)),
        None,
    )?;

    bar.finish_and_clear();

    Ok(())
}
