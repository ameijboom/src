use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::Delta;

use crate::{
    git::{DiffOpts, RemoteOpts, Repo, Tree},
    term::render,
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

fn change_indicators(
    repo: &Repo,
    tree: &Tree<'_>,
    detailed: bool,
) -> Result<Vec<String>, git2::Error> {
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

    Ok(indicators)
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut head = repo.head()?;
    let head_branch = head.shorthand()?.to_string();
    let branch_name = opts.branch.as_deref().unwrap_or(&head_branch);

    let branch = repo.find_branch(branch_name)?;
    let upstream = branch.upstream()?;
    let remote = upstream.remote_name()?;

    let mut remote = repo.find_remote(remote)?;
    remote.fetch(RemoteOpts::default(), branch_name)?;

    let oid = branch.upstream()?.target()?;
    let old_tree = head.find_tree()?;
    let upstream = repo.find_annotated_commit(oid)?;
    let (analysis, _) = repo.merge_analysis(&upstream)?;

    if analysis.is_up_to_date() {
        println!("✓ up to date");
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

    println!(
        "Updated {} to {} {}{}{}",
        render::branch(branch_name),
        render::commit(oid),
        "(".bright_black(),
        change_indicators(&repo, &old_tree, opts.details)
            .map(|i| i.join(" "))
            .unwrap_or("<no changes>".to_string()),
        ")".bright_black(),
    );

    let head = repo.head()?;
    let commit = head.find_commit()?;

    println!(
        "\n{}\n\n{}",
        commit
            .headers_formatted()
            .with_color(colored::Color::BrightBlack),
        commit.message_formatted()
    );

    Ok(())
}
