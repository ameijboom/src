use std::error::Error;

use git2::{BranchType, Repository};

use crate::named::Named;

pub fn run(repo: Repository, branch: String) -> Result<(), Box<dyn Error>> {
    let branch = repo.find_branch(&branch, BranchType::Local)?;
    let reference = branch.into_reference();

    repo.set_head(reference.name_checked()?)?;
    drop(reference);

    super::status::run(repo)
}
