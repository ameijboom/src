use std::error::Error;

use clap::Parser;
use git2::{
    build::CheckoutBuilder, BranchType, Config, ErrorClass, ErrorCode, Reference, Repository,
    StashFlags,
};

use crate::{git, named::Named, select};

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

fn try_checkout(repo: &Repository, reference: Reference<'_>) -> Result<bool, git2::Error> {
    let tree = reference.peel_to_tree()?.into_object();

    if let Err(e) = repo.checkout_tree(&tree, Some(CheckoutBuilder::default().safe())) {
        if e.code() != ErrorCode::Conflict && e.class() != ErrorClass::Checkout {
            return Err(e);
        }

        return Ok(false);
    }

    Ok(true)
}

pub fn run(mut repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let branches = repo.branches(Some(BranchType::Local))?;
    let names = branches
        .map(|result| result.map_err(|e| e.into()))
        .map(|result| result.and_then(|(branch, _)| branch.name_checked().map(ToOwned::to_owned)))
        .collect::<Result<Vec<_>, _>>()?;

    let branch_name = match opts.branch {
        Some(branch) => branch,
        None => match select::single(&names)? {
            Some(branch) => branch,
            None => return Err("No branch selected".into()),
        },
    };

    let branch = repo.find_branch(&branch_name, BranchType::Local)?;
    let reference = branch.into_reference();
    let refname = reference.name_checked()?.to_string();

    if !try_checkout(&repo, reference)? {
        let config = Config::open_default()?;
        let signature = git::signature(&config)?;

        repo.stash_save(
            &signature,
            &format!("auto stash before checkout to: {branch_name}"),
            Some(StashFlags::INCLUDE_UNTRACKED),
        )?;

        println!("✓ Changes stashed\n");

        let branch = repo.find_branch(&branch_name, BranchType::Local)?;
        let reference = branch.into_reference();

        if !try_checkout(&repo, reference)? {
            return Err("Checkout failed after stashing changes".into());
        }
    }

    repo.set_head(&refname)?;

    super::status::run(repo)
}
