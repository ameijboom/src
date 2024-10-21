use std::error::Error;

use clap::Parser;
use git2::{build::CheckoutBuilder, BranchType, Repository};

use crate::{named::Named, select};

#[derive(Parser)]
#[clap(about = "Checkout a branch")]
pub struct Opts {
    #[clap(help = "Branch name")]
    branch: Option<String>,
}

impl Opts {
    pub fn with_branch(branch: String) -> Self {
        Self {
            branch: Some(branch),
        }
    }
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let branches = repo.branches(Some(BranchType::Local))?;
    let names = branches
        .map(|result| result.map_err(|e| e.into()))
        .map(|result| result.and_then(|(branch, _)| branch.name_checked().map(ToOwned::to_owned)))
        .collect::<Result<Vec<_>, _>>()?;

    let branch = match opts.branch {
        Some(branch) => branch,
        None => match select::single(&names)? {
            Some(branch) => branch,
            None => return Err("No branch selected".into()),
        },
    };

    let branch = repo.find_branch(&branch, BranchType::Local)?;
    let reference = branch.into_reference();

    repo.set_head(reference.name_checked()?)?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;

    drop(reference);

    super::status::run(repo)
}
