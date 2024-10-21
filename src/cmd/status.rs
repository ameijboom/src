use std::error::Error;

use colored::Colorize;
use git2::{ErrorCode, Repository, RepositoryState, Status};

use crate::{named::Named, utils};

fn show_branch(repo: &Repository) -> Result<(), Box<dyn Error>> {
    let indicators = remote_state_indicators(repo)
        .ok()
        .flatten()
        .map(|s| format!(" {}{s}{}", "(".black(), ")".black()))
        .unwrap_or_default();

    match repo.head() {
        Ok(head) => {
            let name = if head.is_branch() || head.is_tag() {
                head.shorthand().map(ToOwned::to_owned)
            } else {
                head.target().map(|oid| utils::short(&oid))
            }
            .unwrap_or_else(|| "<unknown>".to_owned());

            println!("On {}{indicators}", format!(" {name}").purple());
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            println!("On {}{indicators}", "[no branch]".yellow());
        }
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

fn remote_state_indicators(repo: &Repository) -> Result<Option<String>, Box<dyn Error>> {
    let head = repo.head()?;
    let remote = utils::find_remote_ref(repo, head.name_checked()?)?;
    let (Some(local), Some(remote)) = (head.target(), remote.target()) else {
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
    let statuses = utils::status_entries(repo)?;
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
            s if s.is_wt_deleted() || s.is_index_deleted() => "-".red(),
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
