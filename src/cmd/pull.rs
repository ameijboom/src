use std::error::Error;

use clap::Parser;

use crate::{
    git::{RemoteOpts, Repo},
    term::ui::{self, Icon},
};

#[derive(Parser, Default)]
#[clap(about = "Pull changes")]
pub struct Opts {
    #[clap(short, long, help = "Show detailed output")]
    details: bool,

    #[clap(short, long, help = "Enable (experimental) rebase mode")]
    rebase: bool,

    #[clap(help = "Branch to pull from")]
    branch: Option<String>,
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    {
        let mut head = repo.head()?;
        let head_branch = head.shorthand()?.to_string();
        let branch_name = opts.branch.as_deref().unwrap_or(&head_branch);

        let branch = repo.find_branch(branch_name)?;
        let upstream = branch.upstream()?;
        let remote = upstream.remote_name()?;

        let mut remote = repo.find_remote(remote)?;
        remote.fetch(RemoteOpts::default(), branch_name)?;

        let oid = branch.upstream()?.target()?;
        let upstream = repo.find_annotated_commit(oid)?;
        let (analysis, _) = repo.merge_analysis(&upstream)?;

        if analysis.is_up_to_date() {
            println!("{}", ui::message_with_icon(Icon::Check, "up to date"));
            return Ok(());
        } else if analysis.is_fast_forward() {
            let target = head.set_target(oid, "fast-forward")?;
            repo.checkout_tree(&target.find_tree()?, true)?;
        } else if opts.rebase {
            let oid = head.target()?;
            let local = repo.find_annotated_commit(oid)?;

            repo.rebase(&local, &upstream)?;

            let oid = repo.head()?.target().unwrap();
            let reference = repo.create_ref(head.name()?, oid)?;

            repo.checkout(&reference)?;
        } else {
            return Err("unable to fast-forward (rebase disabled)".into());
        }
    }

    super::status::run(repo, super::status::Opts::default())
}
