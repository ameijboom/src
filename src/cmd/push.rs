use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::ErrorCode;

use crate::{
    git::{Branch, Config, RemoteOpts, Repo},
    term::{
        bar::Bar,
        node::{self, Attribute, Icon, Node},
        render::{Render, TermRenderer},
    },
};

#[derive(Parser)]
#[clap(about = "Update remote refs along with associated objects")]
pub struct Opts {
    #[clap(short, long, help = "Force push")]
    force: bool,
}

fn set_tracking_branch(
    repo: &Repo,
    remote: &str,
    branch: &mut Branch<'_>,
) -> Result<(), Box<dyn Error>> {
    let name = branch.name()?;
    let reference = repo.create_ref(&format!("refs/remotes/{remote}/{name}"), branch.target()?)?;

    branch.set_upstream(reference.shorthand()?)?;

    Ok(())
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let refname = head.name()?.to_string();
    let mut branch = head.into_branch()?;
    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(e) if e.code() == ErrorCode::NotFound => {
            let config = Config::open_default()?;

            if !config.push.auto_setup_remote {
                return Err("No remote branch found".into());
            }

            set_tracking_branch(&repo, "origin", &mut branch)?;
            branch.upstream()?
        }
        Err(e) => return Err(e.into()),
    };

    let target = branch.upstream()?.target()?;
    let remote_name = upstream.remote_name()?;
    let mut remote = repo.find_remote(remote_name)?;
    let mut ui = TermRenderer::default();
    let bar = Bar::default();

    ui.renderln(&Node::Block(vec![
        Node::Text("Pushing to: ".into()),
        Node::Breadcrumb(vec![
            Node::Attribute(Attribute::Remote(remote_name.to_string().into())),
            Node::Attribute(Attribute::Branch(branch.name()?.to_string().into())),
        ]),
    ]))?;

    let reply = remote.push(
        RemoteOpts::with_bar(bar).with_compare(target),
        &if opts.force {
            format!("+{refname}")
        } else {
            refname
        },
    )?;

    ui.renderln(&node::message_with_icon(Icon::Check, "done"))?;

    if let Ok(msg) = std::str::from_utf8(&reply.stdout)
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\0'))
    {
        if !msg.is_empty() {
            println!("\nReply:");
            println!("{}", msg.bright_black());
        }
    }

    Ok(())
}
