use std::error::Error;

use colored::Colorize;
use git2::{ErrorCode, RepositoryState};

use crate::{
    git::{Change, EntryStatus, Repo},
    term::render,
};

fn show_branch(repo: &Repo) -> Result<(), Box<dyn Error>> {
    let indicators = remote_state_indicators(repo)
        .ok()
        .flatten()
        .map(|s| format!(" {}{s}{}", "(".bright_black(), ")".bright_black()))
        .unwrap_or_default();

    match repo.head() {
        Ok(head) => println!("On {}{indicators}", render::reference(&head)?),
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            println!("On {}{indicators}", "[no branch]".yellow());
        }
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

fn remote_state_indicators(repo: &Repo) -> Result<Option<String>, Box<dyn Error>> {
    let head = repo.head()?;
    let local = head.target()?;
    let upstream = repo
        .find_upstream_branch(&head)?
        .map(|r| r.target())
        .transpose()?;
    let Some(remote) = upstream else {
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

fn show_state(repo: &Repo) -> Result<(), Box<dyn Error>> {
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

fn show_changes(repo: &Repo) -> Result<(), Box<dyn Error>> {
    let status = repo.status()?;
    let entries = status.entries().collect::<Vec<_>>();

    if entries.is_empty() {
        println!("No changes");
        return Ok(());
    }

    println!("Changes:");

    for entry in entries {
        let (indexed, change) = match entry.status() {
            EntryStatus::Unknown => (false, None),
            EntryStatus::WorkTree(change) => (false, Some(change)),
            EntryStatus::Index(change) => (true, Some(change)),
        };
        let indicator = match change {
            Some(Change::New) => "+".green(),
            Some(Change::Modified) => "~".yellow(),
            Some(Change::Renamed) => ">".yellow(),
            Some(Change::Deleted) => "-".red(),
            None | Some(Change::Type) => "?".bright_black(),
        };

        if indexed {
            println!("  {} {}", indicator.bold(), entry.path()?.white());
        } else {
            println!("  {indicator} {}", entry.path()?.bright_black());
        }
    }

    Ok(())
}

pub fn run(repo: Repo) -> Result<(), Box<dyn Error>> {
    show_branch(&repo)?;
    show_state(&repo)?;
    show_changes(&repo)?;

    Ok(())
}
