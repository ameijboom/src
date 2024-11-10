use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use clap::Parser;
use colored::{Color, Colorize};
use minus::Pager;

use crate::{
    git::{Commit, Repo},
    term::render,
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
}

impl Cmd {
    pub fn name(&self) -> &str {
        match self {
            Cmd::Stash => "stash",
            Cmd::Commit { .. } => "commit",
            Cmd::Remote => "remote",
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
                .map(|name| render::remote(name).to_string())
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
        let signed = if commit.is_signed() {
            "⚿ ".green()
        } else if short {
            "  ".white()
        } else {
            "".white()
        };
        let message = commit.message().unwrap_or_default().trim();

        if short {
            writeln!(
                stdout,
                "{signed}{} {}",
                render::commit(commit.id()),
                message.split('\n').next().unwrap_or_default()
            )?;
        } else {
            writeln!(stdout, "{signed}{}", commit.id().to_string().yellow())?;
            writeln!(
                stdout,
                "{}\n",
                commit.headers_formatted().with_color(Color::BrightBlack)
            )?;
            writeln!(stdout, "{}\n", commit.message_formatted())?;
        }
    }

    Ok(())
}

fn render(mut repo: Repo, stdout: impl fmt::Write, opts: Opts) -> Result<(), Box<dyn Error>> {
    match opts.cmd {
        Some(cmd) => match cmd {
            Cmd::Remote => list_remotes(&mut repo, stdout),
            Cmd::Stash => {
                let stashes = repo.stashes()?;
                list_commits(stashes, opts.short, stdout)
            }
            Cmd::Commit { target } => {
                let target = match target {
                    Some(target) => repo.find_branch(&target).map(|b| b.into_ref()),
                    None => repo.head(),
                }?;
                let commits = repo.commits(&target)?;

                list_commits(commits, opts.short, stdout)
            }
        },
        None => {
            let commits = repo.commits(&repo.head()?)?;
            list_commits(commits, opts.short, stdout)
        }
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
