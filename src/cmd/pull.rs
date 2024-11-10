use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::Delta;

use crate::{
    git::{DiffOpts, RemoteOpts, Repo, Tree},
    term::render,
    utils,
};

#[derive(Parser, Default)]
#[clap(about = "Pull changes")]
pub struct Opts {
    #[clap(short, long, help = "Show detailed output")]
    details: bool,
}

pub fn show_changes(repo: &Repo, tree: &Tree<'_>, detailed: bool) -> Result<(), git2::Error> {
    let diff = repo.diff(DiffOpts::default().with_all(tree))?;
    let stats = diff.stats()?;
    let mut indicators = vec![];

    if stats.insertions() > 0 {
        indicators.push(format!("+{}", stats.insertions()).green().to_string());
    }

    if stats.deletions() > 0 {
        indicators.push(format!("-{}", stats.deletions()).red().to_string());
    }

    if detailed {
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
    }

    println!(
        "Changes {}{}{}",
        "(".bright_black(),
        indicators.join(" "),
        ")".bright_black()
    );

    Ok(())
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut head = repo.head()?;
    let branch_name = head.shorthand()?.to_string();

    let branch = repo.find_branch(&branch_name)?;
    let upstream = branch.upstream()?;
    let remote = utils::parse_remote(upstream.name()?)?;

    let mut remote = repo.find_remote(remote)?;
    remote.fetch(RemoteOpts::default(), &branch_name)?;

    let Some(oid) = branch.upstream()?.target() else {
        return Err("invalid oid for upstream".into());
    };

    let commit = repo.find_annotated_commit(oid)?;
    let (analysis, _) = repo.merge_analysis(&commit)?;

    if analysis.is_up_to_date() {
        println!("Already up to date");
        return Ok(());
    } else if !analysis.is_fast_forward() {
        return Err("unsupported operation (no fast-forward)".into());
    }

    let old_tree = head.find_tree()?;
    let target = head.set_target(oid, "fast-forward")?;

    repo.checkout_tree(&target.find_tree()?, true)?;

    println!(
        "Updated {} to {}: {}",
        render::branch(&branch_name),
        render::commit(oid),
        utils::shorten(repo.find_commit(oid)?.message()?, 50),
    );

    Ok(show_changes(&repo, &old_tree, opts.details)?)
}
