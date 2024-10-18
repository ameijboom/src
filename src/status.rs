use core::str;
use std::error::Error;

use colored::Colorize;
use git2::{ErrorCode, Repository, RepositoryState, Status, StatusOptions};

fn show_branch(repo: &Repository) -> Result<(), Box<dyn Error>> {
    let indicators = remote_state_indicators(repo)?
        .map(|s| format!(" {}{s}{}", "(".black(), ")".black()))
        .unwrap_or_default();

    match repo.head() {
        Ok(head) => {
            let branch = head.shorthand().unwrap_or_default();
            println!("On: {}{indicators}", format!(" {branch}").purple());
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            println!("On: {}{indicators}", "[no branch]".yellow());
        }
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

fn remote_state_indicators(repo: &Repository) -> Result<Option<String>, Box<dyn Error>> {
    let Ok(head) = repo.head() else {
        return Ok(None);
    };
    let remote = repo.branch_upstream_name(head.name().unwrap_or_default())?;
    let remote = str::from_utf8(&remote)?;
    let remote = repo.find_reference(remote)?;

    let Some(remote) = remote.target() else {
        return Ok(None);
    };
    let Some(local) = head.target() else {
        return Ok(None);
    };

    let (ahead, behind) = repo.graph_ahead_behind(local, remote)?;

    if ahead == 0 && behind == 0 {
        Ok(None)
    } else if ahead == 0 && behind != 0 {
        Ok(Some(format!("{} {}", "↓".red(), behind)))
    } else if behind == 0 && ahead != 0 {
        Ok(Some(format!("{} {}", "↑".green(), ahead)))
    } else {
        Ok(Some(format!(
            "{} {} {} {}",
            "↑".green(),
            ahead,
            "↓".red(),
            behind
        )))
    }
}

fn show_state(repo: &Repository) -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

fn show_changes(repo: &Repository) -> Result<(), Box<dyn Error>> {
    let mut opts = StatusOptions::new();
    let statuses = repo.statuses(Some(
        opts.include_ignored(false)
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .exclude_submodules(true),
    ))?;

    let entries = statuses
        .iter()
        .filter(|e| e.status() != Status::CURRENT)
        .collect::<Vec<_>>();

    if entries.is_empty() {
        println!("No changes");
        return Ok(());
    }

    println!("Changes:");
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

pub fn run(repo: Repository) -> Result<(), Box<dyn Error>> {
    show_branch(&repo)?;
    show_state(&repo)?;
    show_changes(&repo)?;

    Ok(())
}
