use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::{build::CheckoutBuilder, BranchType, Delta, FetchOptions, Repository};
use indicatif::ProgressBar;

use crate::{callbacks::remote_callbacks, named::Named, utils};

#[derive(Parser)]
#[clap(about = "Pull changes")]
pub struct Opts {}

pub fn run(repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut stdout = vec![];
    let mut bar = ProgressBar::new_spinner();

    let mut head = repo.head()?;
    let tree = head.peel_to_tree()?;
    let Some(branch_name) = head.shorthand().map(ToOwned::to_owned) else {
        return Err("invalid name for HEAD".into());
    };

    let branch = repo.find_branch(&branch_name, BranchType::Local)?;
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

    let Some(oid) = branch.upstream()?.into_reference().target() else {
        return Err("invalid oid for upstream".into());
    };

    let commit = repo.find_annotated_commit(oid)?;
    let (analysis, _) = repo.merge_analysis(&[&commit])?;

    if analysis.is_up_to_date() {
        println!("Already up to date");
        return Ok(());
    } else if !analysis.is_fast_forward() {
        return Err("unsupported operation (no fast-forward)".into());
    }

    head.set_target(oid, "fast-forward")?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;

    println!(
        "Updated {} to {}",
        format!(" {branch_name}").purple(),
        utils::short(&oid).yellow()
    );

    let diff = repo.diff_tree_to_workdir_with_index(Some(&tree), None)?;
    let stats = diff.stats()?;
    let mut indicators = vec![];

    if stats.insertions() > 0 {
        indicators.push(format!("+{}", stats.insertions()).green().to_string());
    }

    if stats.deletions() > 0 {
        indicators.push(format!("-{}", stats.deletions()).red().to_string());
    }

    println!(
        "Changes {}{}{}:",
        "(".black(),
        indicators.join(" "),
        ")".black()
    );

    for delta in diff.deltas() {
        if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
            match delta.status() {
                Delta::Added => println!("  {}", format!("+ {path}").green()),
                Delta::Deleted => println!("  {}", format!("- {path}").red()),
                Delta::Modified => println!("  {}", format!("~ {path}").yellow()),
                Delta::Renamed => println!("  {}", format!("> {path}").yellow()),
                _ => continue,
            }
        }
    }

    Ok(())
}
