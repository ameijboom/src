use std::error::Error;

use clap::Parser;
use gix::{
    bstr::ByteSlice,
    progress,
    refs::Category,
    remote,
    state::InProgress,
    status::{index_worktree, Item, UntrackedFiles},
    Repository,
};
use minus::Pager;
use tracing::instrument;

use crate::{
    graph::Graph,
    rebase::{Rebase, RebaseOperationType},
    term::{
        node::prelude::*,
        render::{Render, TermRenderer},
    },
};

#[derive(Parser, Default)]
#[clap(about = "Show status")]
pub struct Opts {
    #[clap(long, help = "Disable the pager")]
    no_pager: bool,
}

#[instrument(skip(ui, repo, graph), ret(Debug))]
fn render_branch(
    ui: &mut impl Render,
    repo: &Repository,
    graph: Option<&Graph>,
) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;

    match &head.kind {
        gix::head::Kind::Symbolic(_) => {
            let mut group = vec![];
            let branch = head
                .referent_name()
                .and_then(|name| name.category_and_short_name())
                .and_then(|(category, short_name)| {
                    if category == Category::LocalBranch {
                        Some(short_name.to_string())
                    } else {
                        None
                    }
                });
            let object = head.into_peeled_object()?;

            group.push(Node::Attribute(match branch {
                Some(branch) => Attribute::Branch(branch.into()),
                _ => Attribute::from_object(&object)?,
            }));
            group.push(spacer!());

            if let Some(indicators) =
                graph.and_then(|graph| remote_state_indicators(graph).ok().flatten())
            {
                group.push(label!(indicators));
                group.push(spacer!());
            };

            let commit = object.into_commit();

            group.push(Node::text_capped(
                commit
                    .message()?
                    .title
                    .to_str()?
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                75,
            ));

            ui.renderln(&Node::Block(group))?;
            Ok(())
        }
        gix::head::Kind::Unborn { .. } => {
            ui.renderln(&Node::Attribute(Attribute::Branch("[no branch]".into())))?;
            Ok(())
        }
        gix::head::Kind::Detached { .. } => {
            ui.renderln(&Node::Attribute(Attribute::Branch("[detached]".into())))?;
            Ok(())
        }
    }
}

#[instrument(skip(graph), ret(Debug))]
fn remote_state_indicators(graph: &Graph) -> Result<Option<Node>, Box<dyn Error>> {
    let (ahead, behind) = (graph.ahead.len(), graph.behind.len());

    Ok(if ahead == 0 && behind == 0 {
        None
    } else if ahead == 0 && behind != 0 {
        Some(block!(
            icon!(ArrowDown).with_status(Status::Error),
            spacer!(),
            text!(behind.to_string())
        ))
    } else if behind == 0 && ahead != 0 {
        Some(block!(
            icon!(ArrowUp).with_status(Status::Success),
            spacer!(),
            text!(ahead.to_string())
        ))
    } else {
        Some(block!(
            icon!(ArrowUp).with_status(Status::Success),
            spacer!(),
            text!(ahead.to_string()),
            spacer!(),
            icon!(ArrowDown).with_status(Status::Error),
            text!(behind.to_string())
        ))
    })
}

#[instrument(skip(ui, repo), ret(Debug))]
fn render_rebase(ui: &mut impl Render, repo: &Repository) -> Result<(), Box<dyn Error>> {
    let rebase = Rebase::from_repo(repo)?;
    let mut children = vec![];

    for op in rebase.operations.iter() {
        let id = op.oid.to_string();
        let kind = match op.ty {
            RebaseOperationType::Pick => "pick",
            RebaseOperationType::Reword => "reword",
            RebaseOperationType::Edit => "edit",
            RebaseOperationType::Squash => "squash",
            RebaseOperationType::Fixup => "fixup",
            RebaseOperationType::Exec => "exec",
        };

        children.push(block!(
            spacer!(),
            spacer!(),
            Node::Attribute(Attribute::Operation(kind.into())),
            spacer!(),
            dimmed!(text!(id[..6].to_string())),
            spacer!(),
            Node::text_head_1(op.message.clone())
        ));
    }

    children.push(block!(
        spacer!(),
        spacer!(),
        continued!(text!("Fix conflicts and run 'git rebase --continue'"))
    ));

    ui.renderln(&Node::Group(
        "Rebase".into(),
        Some(rebase.operations.len()),
        Box::new(Node::MultiLine(children)),
    ))?;

    Ok(())
}

#[instrument(skip(ui, repo), ret(Debug))]
fn render_state(ui: &mut impl Render, repo: &Repository) -> Result<(), Box<dyn Error>> {
    match repo.state() {
        Some(state) => match state {
            InProgress::ApplyMailbox | InProgress::ApplyMailboxRebase => todo!(),
            InProgress::Bisect => {
                ui.renderln(&text!("Bisect in progress"))?;
                Ok(())
            }
            InProgress::CherryPick | InProgress::CherryPickSequence => {
                ui.renderln(&text!("Cherry-pick in progress"))?;
                Ok(())
            }
            InProgress::Merge => {
                ui.renderln(&text!("Merge in progress"))?;
                Ok(())
            }
            InProgress::Rebase | InProgress::RebaseInteractive => render_rebase(ui, repo),
            InProgress::Revert | InProgress::RevertSequence => {
                ui.renderln(&text!("Revert in progress"))?;
                Ok(())
            }
        },
        _ => Ok(()),
    }
}

#[instrument(skip(ui, graph), ret(Debug))]
fn render_commits(ui: &mut impl Render, graph: Graph) -> Result<(), Box<dyn Error>> {
    let mut children = vec![];
    let groups = [
        ("Unmerged into remote", graph.ahead),
        ("Unpulled from remote", graph.behind),
    ];

    for (name, platform) in groups {
        let commits = platform;

        if commits.is_empty() {
            continue;
        }

        let count = commits.len();
        let mut lines = vec![];

        for info in commits {
            let commit = info.object()?;
            let id = commit.id().to_string();

            lines.push(block!(
                if commit.signature()?.is_some() {
                    icon!(Lock).with_status(Status::Success)
                } else {
                    spacer!()
                },
                spacer!(),
                dimmed!(text!(id[..6].to_string())),
                spacer!(),
                Node::text_head_1(commit.message()?.title.to_string())
            ));
        }

        children.push(Node::Group(
            name.into(),
            Some(count),
            Box::new(Node::MultiLine(lines)),
        ));
    }

    if children.is_empty() {
        return Ok(());
    }

    Ok(ui.renderln(&Node::MultiLine(children))?)
}

#[instrument(skip(ui, repo), ret(Debug))]
fn render_changes(ui: &mut impl Render, repo: &Repository) -> Result<(), Box<dyn Error>> {
    let mut children = vec![];
    let status = repo
        .status(progress::Discard)?
        .untracked_files(UntrackedFiles::Files);
    let entries = status.into_iter([])?.collect::<Result<Vec<_>, _>>()?;
    let (staged, unstaged): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|e| matches!(e, Item::TreeIndex(_)));
    let groups = [("Staged Changes", staged), ("Unstaged Changes", unstaged)];

    for (name, items) in groups {
        if items.is_empty() {
            continue;
        }

        let count = items.len();
        let mut lines = vec![];

        for item in items {
            let indicator = match &item {
                Item::IndexWorktree(item) => match item {
                    index_worktree::Item::Modification { .. } => Indicator::Modified,
                    index_worktree::Item::DirectoryContents { entry, .. } => match entry.status {
                        gix::dir::entry::Status::Untracked => Indicator::New,
                        _ => Indicator::Modified,
                    },
                    index_worktree::Item::Rewrite { .. } => Indicator::Renamed,
                },
                Item::TreeIndex(change) => match change {
                    gix::diff::index::ChangeRef::Addition { .. } => Indicator::New,
                    gix::diff::index::ChangeRef::Deletion { .. } => Indicator::Deleted,
                    gix::diff::index::ChangeRef::Modification { .. } => Indicator::Modified,
                    gix::diff::index::ChangeRef::Rewrite { .. } => Indicator::Renamed,
                },
            };

            lines.push(block!(
                spacer!(),
                spacer!(),
                Node::Indicator(indicator),
                spacer!(),
                text!(item.location().to_string())
            ));
        }

        children.push(Node::Group(
            name.into(),
            Some(count),
            Box::new(Node::MultiLine(lines)),
        ));
    }

    if children.is_empty() {
        return Ok(());
    }

    Ok(ui.render(&Node::MultiLine(children))?)
}

#[instrument(skip(repo), ret(Debug))]
fn find_state(repo: &Repository) -> Result<Option<(gix::Id<'_>, gix::Id<'_>)>, Box<dyn Error>> {
    let Some(local) = repo.head_ref()? else {
        return Ok(None);
    };

    if local.name().category() != Some(gix::reference::Category::LocalBranch) {
        return Ok(None);
    }

    let Some(upstream) = local
        .remote_tracking_ref_name(remote::Direction::Fetch)
        .transpose()?
    else {
        return Ok(None);
    };

    let upstream = repo.find_reference(upstream.as_partial_name())?;

    Ok(Some((local.id(), upstream.id())))
}

#[instrument(skip(ui, repo), ret(Debug))]
fn render(mut ui: impl Render, repo: Repository) -> Result<(), Box<dyn Error>> {
    let graph = match find_state(&repo)? {
        Some((local, remote)) => Some(Graph::ahead_behind(&repo, local, remote)?),
        None => None,
    };

    render_branch(&mut ui, &repo, graph.as_ref())?;
    render_state(&mut ui, &repo)?;
    render_changes(&mut ui, &repo)?;

    graph
        .map(|graph| {
            ui.renderln(&Node::Empty)?;
            render_commits(&mut ui, graph)
        })
        .transpose()?;

    Ok(())
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.no_pager {
        render(TermRenderer::default(), repo)
    } else {
        let mut pager = Pager::new();
        pager.set_prompt("status, q to quit")?;

        render(TermRenderer::new(&mut pager), repo)?;
        minus::page_all(pager)?;

        Ok(())
    }
}
