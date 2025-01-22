use std::error::Error;

use colored::{Color, Colorize};
use git2::{ErrorCode, RepositoryState};

use crate::{
    git::{Change, EntryStatus, Repo},
    term::render,
};

const HEADER: Color = Color::TrueColor {
    r: 225,
    g: 190,
    b: 120,
};

fn show_branch(repo: &Repo, state: Option<(git2::Oid, git2::Oid)>) -> Result<(), Box<dyn Error>> {
    let indicators = state
        .and_then(|state| {
            remote_state_indicators(repo, state)
                .ok()
                .flatten()
                .map(|s| format!("{}{s}{} ", "[".bright_black(), "]".bright_black()))
        })
        .unwrap_or_default();

    match repo.head() {
        Ok(head) => {
            let commit = head.find_commit()?;
            let message = commit.message()?.lines().next().unwrap_or_default().trim();

            println!(
                "{indicators}{} {}",
                render::reference(&head)?.with_bold(),
                if message.len() > 40 {
                    format!("{}...", &message[..37])
                } else {
                    message.to_string()
                }
                .bright_black(),
            );
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            println!("On {}{indicators}", "[no branch]".yellow());
        }
        Err(e) => return Err(e.into()),
    };

    Ok(())
}

fn remote_state_indicators(
    repo: &Repo,
    state: (git2::Oid, git2::Oid),
) -> Result<Option<String>, Box<dyn Error>> {
    let (local, remote) = state;
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

fn show_commits(repo: &Repo, local: git2::Oid, remote: git2::Oid) -> Result<(), Box<dyn Error>> {
    let (ahead, behind) = repo.commits_ahead_behind(local, remote)?;
    let groups = [
        ("Unmerged into remote", ahead),
        ("Unpulled from remote", behind),
    ];

    for (name, commits) in groups {
        if commits.is_empty() {
            continue;
        }

        println!(
            "\n{}",
            format!(
                "{} {}",
                format!("{name}").color(HEADER).bold(),
                format!("({})", commits.len()).bright_black(),
            ),
        );

        for commit in commits {
            let signed = if commit.is_signed() {
                "⚿ ".green()
            } else {
                "  ".white()
            };
            let message = commit.message().unwrap_or_default().trim();

            println!(
                "{signed}{} {}",
                render::commit(commit.id()).with_color(Color::BrightGreen),
                if message.len() > 40 {
                    format!("{}...", &message[..37])
                } else {
                    message.to_string()
                }
            );
        }
    }

    Ok(())
}

fn show_changes(repo: &Repo) -> Result<(), Box<dyn Error>> {
    let status = repo.status()?;
    let entries = status.entries().collect::<Vec<_>>();
    let (staged, unstaged): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.is_staged());
    let groups = [("Staged Changes", staged), ("Unstaged Changes", unstaged)];

    for (name, entries) in groups {
        if entries.is_empty() {
            continue;
        }

        println!(
            "\n{}",
            format!(
                "{} {}",
                format!("{name}").color(HEADER).bold(),
                format!("({})", entries.len()).bright_black(),
            ),
        );

        for entry in entries {
            let change = match entry.status() {
                EntryStatus::Unknown => None,
                EntryStatus::WorkTree(change) => Some(change),
                EntryStatus::Index(change) => Some(change),
            };
            let indicator = match change {
                Some(Change::New) => "+".green(),
                Some(Change::Modified) => "~".yellow(),
                Some(Change::Renamed) => ">".yellow(),
                Some(Change::Deleted) => "-".red(),
                None | Some(Change::Type) => "?".bright_black(),
            };

            println!("  {} {}", indicator.bold(), entry.path()?);
        }
    }

    Ok(())
}

fn find_state(repo: &Repo) -> Result<Option<(git2::Oid, git2::Oid)>, Box<dyn Error>> {
    let head = repo.head()?;
    let local = head.target()?;
    let upstream = repo
        .find_upstream_branch(&head)?
        .map(|r| r.target())
        .transpose()?;
    let Some(remote) = upstream else {
        return Ok(None);
    };

    Ok(Some((local, remote)))
}

pub fn run(repo: Repo) -> Result<(), Box<dyn Error>> {
    let state = find_state(&repo)?;

    show_branch(&repo, state)?;
    show_state(&repo)?;
    show_changes(&repo)?;

    if let Some((local, remote)) = state {
        show_commits(&repo, local, remote)?;
    }

    Ok(())
}
