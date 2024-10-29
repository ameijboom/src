use std::error::Error;

use clap::Parser;
use git2::{build::CheckoutBuilder, Repository};

use crate::named::Named;

#[derive(Parser)]
#[clap(about = "Create a branch")]
pub struct Opts {
    #[clap(help = "Branch name")]
    branch: String,
}

pub fn create_branch_checkout(repo: &Repository, name: &str) -> Result<(), Box<dyn Error>> {
    let target = repo.head()?.peel_to_commit()?;
    let branch = repo.branch(name, &target, false)?;
    let reference = branch.into_reference();
    let tree = reference.peel_to_tree()?;

    repo.checkout_tree(&tree.into_object(), Some(CheckoutBuilder::default().safe()))?;
    repo.set_head(reference.name_checked()?)?;

    Ok(())
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    create_branch_checkout(&repo, &opts.branch)?;
    super::status::run(repo)
}
