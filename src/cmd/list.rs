use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use clap::Parser;
use minus::Pager;

use crate::{
    git::{Commit, Repo},
    term::ui::{Attribute, Icon, Node, Status},
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

struct WrapFmt<T: Write>(T);

impl<T: Write> fmt::Write for WrapFmt<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.write_all(s.as_bytes()).map_err(|_| fmt::Error)
    }
}

fn list_remotes(repo: &mut Repo, mut stdout: impl fmt::Write) -> Result<(), Box<dyn Error>> {
    for remote in repo.remotes()? {
        let remote = remote?;
        writeln!(
            stdout,
            "{}\t{}",
            remote
                .name()?
                .map(|name| Node::Attribute(Attribute::Remote(name.to_string().into())).to_string())
                .unwrap_or_else(|| "<none>".to_string()),
            remote.url()?
        )?;
    }

    Ok(())
}

fn list_commits<'a>(
    walk: impl Iterator<Item = Result<Commit<'a>, git2::Error>>,
    short: bool,
    mut stdout: impl fmt::Write,
) -> Result<(), Box<dyn Error>> {
    for commit in walk {
        let commit = commit?;
        let mut line = vec![];

        if commit.is_signed() {
            line.push(Node::Block(vec![
                Node::Icon(Icon::Lock).with_status(Status::Success),
                Node::spacer(),
            ]));
        } else if short {
            line.push(Node::spacer());
        }

        line.push(Node::Attribute(Attribute::Commit(commit.id())));

        let message = commit.message().unwrap_or_default().trim();

        if short {
            line.push(Node::text_head_1(message));
            writeln!(stdout, "{}", Node::Block(line))?;
        } else {
            let node = Node::MultiLine(vec![
                Node::Block(line),
                Node::Dimmed(Box::new(commit.headers_ui())),
                Node::spacer(),
                Node::Text(commit.message_formatted().into()),
            ]);

            writeln!(stdout, "{node}\n")?;
        }
    }

    Ok(())
}

fn list_branches(repo: Repo, mut stdout: impl fmt::Write) -> Result<(), Box<dyn Error>> {
    for branch in repo.branches()? {
        let branch = branch?;
        writeln!(
            stdout,
            "{}",
            Node::Attribute(Attribute::Branch(branch.name()?.to_string().into()))
        )?;
    }

    Ok(())
}

fn render(mut repo: Repo, stdout: impl fmt::Write, opts: Opts) -> Result<(), Box<dyn Error>> {
    match opts.cmd {
        Some(cmd) => match cmd {
            Cmd::Branch => list_branches(repo, stdout),
            Cmd::Remote => list_remotes(&mut repo, stdout),
            Cmd::Stash => list_commits(repo.stashes()?, opts.short, stdout),
            Cmd::Commit { target } => {
                let target = match target {
                    Some(target) => repo.find_branch(&target).map(|b| b.into_ref()),
                    None => repo.head(),
                }?;

                list_commits(repo.commits(&target)?, opts.short, stdout)
            }
        },
        None => list_commits(repo.commits(&repo.head()?)?, opts.short, stdout),
    }
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.no_pager {
        render(repo, WrapFmt(io::stdout()), opts)
    } else {
        colored::control::set_override(true);

        let cmd = opts.cmd.as_ref().map(Cmd::name).unwrap_or("commit");
        let mut pager = Pager::new();
        pager.set_prompt(format!("list {cmd}s, q to quit"))?;

        render(repo, &mut pager, opts)?;
        minus::page_all(pager)?;

        Ok(())
    }
}
