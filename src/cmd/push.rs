use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::{Branch, BranchType, ErrorCode, PushOptions, Repository};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{callbacks::remote_callbacks, named::Named, utils};

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
    repo: &Repository,
    remote: &str,
    branch: &mut Branch<'_>,
) -> Result<(), Box<dyn Error>> {
    let name = branch.name_checked()?;
    let reference = repo.reference(
        &format!("refs/remotes/{remote}/{name}"),
        branch.get().target().ok_or(PushError::MissingTarget)?,
        true,
        "",
    )?;

    branch.set_upstream(reference.shorthand())?;

    Ok(())
}

pub fn run(repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let head_name = head.shorthand().unwrap_or_default();
    let mut branch = repo.find_branch(head_name, BranchType::Local)?;
    let branch_name = branch.name_checked()?.to_string();

    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(e) if e.code() == ErrorCode::NotFound => {
            let config = repo.config()?;
            let setup_remote = config.get_bool("push.autoSetupRemote")?;

            if !setup_remote {
                println!("{}", "No remote branch found".red());
                return Ok(());
            }

            set_tracking_branch(&repo, "origin", &mut branch)?;
            branch.upstream()?
        }
        Err(e) => return Err(e.into()),
    };

    let remote_name = utils::parse_remote(upstream.name_checked()?)?;
    let mut remote = repo.find_remote(remote_name)?;

    println!(
        "Pushing to: {} / {}",
        format!("⬡ {remote_name}").cyan(),
        format!(" {branch_name}").purple(),
    );

    let mut bar = ProgressBar::new_spinner().with_style(ProgressStyle::with_template(
        "{spinner} ({pos}/{len}) {msg}",
    )?);
    bar.set_message("Preparing");

    let mut out = vec![];
    let callbacks = remote_callbacks(&mut out, &mut bar);

    remote.push(
        &[head.name_checked()?],
        Some(PushOptions::new().remote_callbacks(callbacks)),
    )?;

    bar.finish_and_clear();
    println!("✓ done");

    if let Ok(msg) = std::str::from_utf8(&out).map(|s| s.trim()) {
        if !msg.is_empty() {
            println!("\nReply:");
            println!("{}", msg.bright_black());
        }
    }

    Ok(())
}
