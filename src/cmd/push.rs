use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::ErrorCode;

use crate::{
    git::{Branch, Config, RemoteOpts, Repo},
    utils,
};

#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("missing target")]
    MissingTarget,
    #[error("failed to parse remote in upstream")]
    ParseRemote(#[from] utils::ParseRemoteError),
}

#[derive(Parser)]
#[clap(about = "Update remote refs along with associated objects")]
pub struct Opts {}

fn set_tracking_branch(
    repo: &Repo,
    remote: &str,
    branch: &mut Branch<'_>,
) -> Result<(), Box<dyn Error>> {
    let name = branch.name()?;
    let reference = repo.create_ref(
        &format!("refs/remotes/{remote}/{name}"),
        branch.target().ok_or(PushError::MissingTarget)?,
    )?;

    branch.set_upstream(reference.shorthand()?)?;

    Ok(())
}

pub fn run(repo: Repo, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let head_name = head.shorthand()?;
    let mut branch = repo.find_branch(head_name)?;
    let branch_name = branch.name()?.to_string();

    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(e) if e.code() == ErrorCode::NotFound => {
            let config = Config::open_default()?;

            if !config.push.auto_setup_remote {
                println!("{}", "No remote branch found".red());
                return Ok(());
            }

            set_tracking_branch(&repo, "origin", &mut branch)?;
            branch.upstream()?
        }
        Err(e) => return Err(e.into()),
    };

    let remote_name = utils::parse_remote(upstream.name()?)?;
    let mut remote = repo.find_remote(remote_name)?;

    println!(
        "Pushing to: {} / {}",
        format!("⬡ {remote_name}").cyan(),
        format!(" {branch_name}").purple(),
    );

    let reply = remote.push(RemoteOpts::default(), head.name()?)?;

    println!("✓ done");

    if let Ok(msg) = std::str::from_utf8(&reply.stdout)
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\0'))
    {
        if !msg.is_empty() {
            println!("\nReply:");
            println!("{}", msg.bright_black());
        }
    }

    Ok(())
}
