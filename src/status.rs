use std::error::Error;

use colored::Colorize;
use git2::{ErrorCode, Repository, RepositoryState, Status, StatusOptions};

pub fn run(repo: Repository) -> Result<(), Box<dyn Error>> {
    match repo.head() {
        Ok(head) => {
            let branch = head.shorthand().unwrap_or_default();
            println!("At {}", format!(" {branch}").purple());
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            println!("At {}", "[no branch]".yellow());
        }
        Err(e) => return Err(e.into()),
    };

    let mut opts = StatusOptions::new();
    let statuses = repo.statuses(Some(
        opts.include_ignored(false)
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .exclude_submodules(true),
    ))?;

    match repo.state() {
        RepositoryState::Merge => println!("In merge"),
        RepositoryState::Revert | RepositoryState::RevertSequence => println!("In revert"),
        RepositoryState::CherryPick | RepositoryState::CherryPickSequence => {
            println!("In cherrypick")
        }
        RepositoryState::Bisect => todo!(),
        // See: https://github.com/libgit2/libgit2/issues/6332
        RepositoryState::Rebase
        | RepositoryState::RebaseInteractive
        | RepositoryState::RebaseMerge => println!("In rebase"),
        _ => {}
    }

    let entries = statuses
        .iter()
        .filter(|e| e.status() != Status::CURRENT)
        .collect::<Vec<_>>();

    if !entries.is_empty() {
        println!("Changes:");
    }

    for entry in entries {
        let status = entry.status();
        let indicator = match status {
            s if s.is_wt_new() || s.is_index_new() => "+".green(),
            s if s.is_wt_modified() || s.is_index_modified() => "~".yellow(),
            s if s.is_wt_renamed() || s.is_index_renamed() => ">".yellow(),
            _ => "?".black(),
        };
        let path = entry.path().unwrap_or_default();
        let indexed = status.is_index_deleted()
            || status.is_index_modified()
            || status.is_index_new()
            || status.is_index_renamed()
            || status.is_index_typechange();

        if indexed {
            println!("  {} {}", indicator.bold(), path.white());
        } else {
            println!("  {indicator} {}", path.black());
        }
    }

    Ok(())
}
