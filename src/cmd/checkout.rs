use std::error::Error;

use clap::Parser;

use crate::{
    git::{CheckoutError, Ref, Repo},
    term::select,
};

#[derive(Parser)]
#[clap(about = "Checkout a branch", alias = "use")]
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

pub fn try_checkout(repo: &Repo, reference: &Ref<'_>) -> Result<bool, git2::Error> {
    match repo.checkout(reference) {
        Ok(()) => Ok(true),
        Err(CheckoutError::Conflict(_)) => Ok(false),
        Err(CheckoutError::Git(e)) => Err(e),
    }
}

fn branch_names(repo: &Repo) -> Result<Vec<String>, Box<dyn Error>> {
    let branches = repo.branches()?;
    Ok(branches
        .map(|result| {
            result
                .map_err(Into::into)
                .and_then(|branch| branch.name().map(ToOwned::to_owned))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

pub fn run(mut repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let branch_name = match opts.branch {
        Some(branch) => branch,
        None => match select::single(&branch_names(&repo)?, Some("src list commit {}"))? {
            Some(branch) => branch,
            None => return Err("No branch selected".into()),
        },
    };

    if !try_checkout(&repo, &repo.find_branch(&branch_name)?.into())? {
        repo.save_stash(&format!("auto stash before checkout to: {branch_name}"))?;

        println!("✓ Changes stashed\n");

        let branch = repo.find_branch(&branch_name)?;
        repo.checkout(&branch.into())?;
    }

    repo.set_head(&repo.find_branch(&branch_name)?.into())?;

    super::status::run(repo)
}
