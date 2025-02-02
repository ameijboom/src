use std::error::Error;

use clap::Parser;
use minus::Pager;

use crate::{
    git::{Commit, Repo},
    term::{
        node::{self, prelude::*},
        render::{Render, TermRenderer},
    },
};

#[derive(Parser)]
#[clap(about = "Show commit logs")]
pub struct Opts {
    #[clap(long, short, help = "Show logs in one line without metadata")]
    short: bool,

    #[clap(long, help = "Disable the pager")]
    no_pager: bool,

    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Parser)]
pub enum Cmd {
    #[clap(about = "List stashes")]
    Stash,

    #[clap(about = "List commits")]
    Commit {
        #[clap(help = "Target branch or tag")]
        target: Option<String>,
    },

    #[clap(about = "List remotes")]
    Remote,

    #[clap(about = "List branches")]
    Branch,
}

impl Cmd {
    pub fn name(&self) -> &str {
        match self {
            Cmd::Stash => "stash",
            Cmd::Commit { .. } => "commit",
            Cmd::Remote => "remote",
            Cmd::Branch => "branch",
        }
    }
}

fn list_remotes(ui: &mut impl Render, repo: &mut Repo) -> Result<(), Box<dyn Error>> {
    for remote in repo.remotes()? {
        let remote = remote?;
        ui.renderln(&node::column!(
            text!(remote.url()?.to_string()),
            remote
                .name()?
                .map(|name| Node::Attribute(Attribute::Remote(name.to_string().into())))
                .unwrap_or_else(|| text!("<none>"))
        ))?;
    }

    Ok(())
}

fn list_commits<'a>(
    ui: &mut impl Render,
    walk: impl Iterator<Item = Result<Commit<'a>, git2::Error>>,
    short: bool,
) -> Result<(), Box<dyn Error>> {
    for commit in walk {
        let commit = commit?;

        if commit.is_signed() {
            ui.render(&block!(icon!(Lock).with_status(Status::Success), spacer!()))?;
        } else if short {
            ui.render(&spacer!())?;
        }

        ui.render(&Node::Attribute(Attribute::Commit(commit.id())))?;

        let message = commit.message().unwrap_or_default().trim();

        if short {
            ui.renderln(&Node::text_head_1(message))?;
        } else {
            ui.renderln(&multi_line!(
                Node::Empty,
                dimmed!(commit.headers_ui()),
                spacer!(),
                text!(commit.message_formatted()),
                Node::Empty
            ))?;
        }
    }

    Ok(())
}

fn list_branches(ui: &mut impl Render, repo: Repo) -> Result<(), Box<dyn Error>> {
    for branch in repo.branches()? {
        ui.renderln(&Node::Attribute(Attribute::Branch(
            branch?.name()?.to_string().into(),
        )))?;
    }

    Ok(())
}

fn render(mut ui: impl Render, mut repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    match opts.cmd {
        Some(cmd) => match cmd {
            Cmd::Branch => list_branches(&mut ui, repo),
            Cmd::Remote => list_remotes(&mut ui, &mut repo),
            Cmd::Stash => list_commits(&mut ui, repo.stashes()?, opts.short),
            Cmd::Commit { target } => {
                let target = match target {
                    Some(target) => repo.find_branch(&target).map(|b| b.into_ref()),
                    None => repo.head(),
                }?;

                list_commits(&mut ui, repo.commits(&target)?, opts.short)
            }
        },
        None => list_commits(&mut ui, repo.commits(&repo.head()?)?, opts.short),
    }
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.no_pager {
        render(TermRenderer::default(), repo, opts)
    } else {
        colored::control::set_override(true);

        let cmd = opts.cmd.as_ref().map(Cmd::name).unwrap_or("commit");
        let mut pager = Pager::new();
        pager.set_prompt(format!("list {cmd}s, q to quit"))?;

        render(TermRenderer::new(&mut pager), repo, opts)?;
        minus::page_all(pager)?;

        Ok(())
    }
}
