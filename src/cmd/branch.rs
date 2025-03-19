use std::error::Error;

use clap::Parser;

use crate::git::Repo;

#[derive(Parser)]
#[clap(about = "Create a branch")]
pub struct Opts {
    #[clap(help = "Branch name")]
    branch: String,
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    {
        let head = repo.head()?;
        let target = head.find_commit()?;
        let branch = repo.create_branch(&opts.branch, &target)?;

        repo.checkout(&branch.into())?;
    }

    super::status::run(gix::open(repo.path())?, super::status::Opts::default())
}
