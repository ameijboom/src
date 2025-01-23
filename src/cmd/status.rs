use std::error::Error;
use std::fmt::Write;

use clap::Parser;
use git2::{ErrorCode, RepositoryState};
use minus::Pager;

use crate::{
    git::{Change, EntryStatus, Repo},
    term::ui::{Attribute, Builder, Icon, Indicator, Node, Status},
};

#[derive(Parser, Default)]
#[clap(about = "Show status")]
pub struct Opts {
    #[clap(long, help = "Disable the pager")]
    no_pager: bool,
}

fn render_branch(
    repo: &Repo,
    state: Option<(git2::Oid, git2::Oid)>,
) -> Result<Node, Box<dyn Error>> {
    let mut group = vec![];

    match repo.head() {
        Ok(head) => {
            let commit = head.find_commit()?;

            group.push(Node::Attribute(Attribute::from_ref(&head)?));
            group.push(Node::spacer());

            if let Some(indicators) =
                state.and_then(|state| remote_state_indicators(repo, state).ok().flatten())
            {
                group.push(Node::Label(Box::new(indicators)));
                group.push(Node::spacer());
            };

            group.push(Node::text_capped(
                commit
                    .message()?
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                75,
            ));
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => {
            group.push(Node::Attribute(Attribute::Branch("[no branch]".into())));
        }
        Err(e) => return Err(e.into()),
    };

    Ok(Node::Block(group))
}

fn remote_state_indicators(
    repo: &Repo,
    state: (git2::Oid, git2::Oid),
) -> Result<Option<Node>, Box<dyn Error>> {
    let (local, remote) = state;
    let (ahead, behind) = repo.graph_ahead_behind(local, remote)?;

    Ok(if ahead == 0 && behind == 0 {
        None
    } else if ahead == 0 && behind != 0 {
        Some(Node::Block(vec![
            Node::Icon(Icon::ArrowDown).with_status(Status::Error),
            Node::spacer(),
            Node::Text(behind.to_string().into()),
        ]))
    } else if behind == 0 && ahead != 0 {
        Some(Node::Block(vec![
            Node::Icon(Icon::ArrowUp).with_status(Status::Success),
            Node::spacer(),
            Node::Text(ahead.to_string().into()),
        ]))
    } else {
        Some(Node::Block(vec![
            Node::Icon(Icon::ArrowUp).with_status(Status::Success),
            Node::spacer(),
            Node::Text(ahead.to_string().into()),
            Node::spacer(),
            Node::Icon(Icon::ArrowDown).with_status(Status::Error),
            Node::Text(behind.to_string().into()),
        ]))
    })
}

fn render_state(repo: &Repo) -> Result<Option<Node>, Box<dyn Error>> {
    Ok(match repo.state() {
        RepositoryState::Merge => Some(Node::Text("In merge".into())),
        RepositoryState::Revert | RepositoryState::RevertSequence => {
            Some(Node::Text("In revert".into()))
        }
        RepositoryState::CherryPick | RepositoryState::CherryPickSequence => {
            Some(Node::Text("In cherrypick".into()))
        }
        RepositoryState::Bisect => todo!(),
        // See: https://github.com/libgit2/libgit2/issues/6332
        RepositoryState::Rebase
        | RepositoryState::RebaseInteractive
        | RepositoryState::RebaseMerge => Some(Node::Text("In rebase".into())),
        _ => None,
    })
}

fn render_commits(
    repo: &Repo,
    local: git2::Oid,
    remote: git2::Oid,
) -> Result<Node, Box<dyn Error>> {
    let mut children = vec![];
    let (ahead, behind) = repo.commits_ahead_behind(local, remote)?;
    let groups = [
        ("Unmerged into remote", ahead),
        ("Unpulled from remote", behind),
    ];

    for (name, commits) in groups {
        if commits.is_empty() {
            continue;
        }

        let count = commits.len();
        let mut lines = vec![];

        for commit in commits {
            let id = commit.id().to_string();

            lines.push(Node::Block(vec![
                if commit.is_signed() {
                    Node::Icon(Icon::Lock).with_status(Status::Success)
                } else {
                    Node::spacer()
                },
                Node::spacer(),
                Node::Dimmed(Box::new(Node::Text(id[..6].to_string().into()))),
                Node::spacer(),
                Node::text_head_1(commit.message().unwrap_or_default()),
            ]));
        }

        children.push(Node::Group(
            name.into(),
            Some(count),
            Box::new(Node::MultiLine(lines)),
        ));
    }

    Ok(Node::MultiLine(children))
}

fn render_changes(repo: &Repo) -> Result<Node, Box<dyn Error>> {
    let mut children = vec![];
    let status = repo.status()?;
    let entries = status.entries().collect::<Vec<_>>();
    let (staged, unstaged): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.is_staged());
    let groups = [("Staged Changes", staged), ("Unstaged Changes", unstaged)];

    for (name, entries) in groups {
        if entries.is_empty() {
            continue;
        }

        let count = entries.len();
        let mut lines = vec![];

        for entry in entries {
            let change = match entry.status() {
                EntryStatus::Unknown => None,
                EntryStatus::WorkTree(change) => Some(change),
                EntryStatus::Index(change) => Some(change),
            };
            let indicator = match change {
                Some(Change::New) => Indicator::New,
                Some(Change::Modified) => Indicator::Modified,
                Some(Change::Renamed) => Indicator::Renamed,
                Some(Change::Deleted) => Indicator::Deleted,
                None | Some(Change::Type) => Indicator::Unknown,
            };

            lines.push(Node::Block(vec![
                Node::spacer(),
                Node::spacer(),
                Node::Indicator(indicator),
                Node::spacer(),
                Node::Text(entry.path()?.to_string().into()),
            ]));
        }

        children.push(Node::Group(
            name.into(),
            Some(count),
            Box::new(Node::MultiLine(lines)),
        ));
    }

    Ok(Node::MultiLine(children))
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

fn render(repo: Repo) -> Result<Node, Box<dyn Error>> {
    let state = find_state(&repo)?;

    Ok(Builder::default()
        .and(render_branch(&repo, state)?)
        .and(render_state(&repo)?)
        .and(render_changes(&repo)?)
        .and(
            state
                .map(|(local, remote)| render_commits(&repo, local, remote))
                .transpose()?,
        )
        .build())
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let node = render(repo)?;

    if opts.no_pager {
        println!("{node}");
    } else {
        let mut pager = Pager::new();
        pager.set_prompt("status, q to quit")?;

        writeln!(&mut pager, "{node}")?;
        minus::page_all(pager)?;
    }

    Ok(())
}
