use std::error::Error;

use clap::Parser;
use git2::ErrorCode;

use crate::{
    cmd::add::add_callback,
    git::{DiffOpts, Repo},
    term::{
        node::prelude::*,
        render::{Render, TermRenderer},
    },
};

#[derive(Parser)]
#[clap(about = "Record changes to the repository")]
pub struct Opts {
    #[clap(short, long, help = "Add all changes")]
    add_all: bool,

    #[clap(short, long, help = "Create a branch")]
    branch: bool,

    #[clap(help = "Commit message")]
    pub message: String,
}

fn branch_name(message: &str) -> String {
    if let Some((prefix, name)) = message.split_once(':') {
        return format!(
            "{}/{}",
            prefix.trim().replace([' ', '/'], "-"),
            name.trim().replace([' ', '/'], "-"),
        );
    }

    message.trim().replace(' ', "-")
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    if opts.branch {
        let head = repo.head()?;
        let commit = head.find_commit()?;
        let branch = repo.create_branch(&branch_name(&opts.message), &commit)?;

        repo.checkout(&branch.into())?;
    }

    let old_tree = match repo.head() {
        Ok(head) => Some(head.find_tree()?),
        Err(e) if e.code() == ErrorCode::UnbornBranch => None,
        Err(e) => return Err(e.into()),
    };

    let mut index = repo.index()?;

    if opts.add_all {
        index.add(["."], add_callback)?;
        index.write()?;
    }

    let tree = repo.find_tree(index.write_tree()?)?;
    let oid = repo.create_commit(&tree, &opts.message, None)?;

    if old_tree.is_none() {
        repo.create_ref("refs/heads/main", oid)?;
    }

    repo.head()?
        .set_target(oid, &format!("commit: {}", opts.message))?;

    let mut opts = DiffOpts::default();

    if let Some(tree) = old_tree.as_ref() {
        opts = opts.with_all(tree);
    }

    let diff = repo.diff(opts)?;
    let stats = diff.stats()?;

    let mut ui = TermRenderer::default();
    let mut children = vec![];

    if stats.insertions() > 0 {
        children.push(
            block!(
                Node::Indicator(Indicator::New),
                text!(stats.insertions().to_string())
            )
            .with_status(Status::Success),
        );
    }

    if stats.deletions() > 0 {
        if !children.is_empty() {
            children.push(spacer!());
        }

        children.push(
            block!(
                Node::Indicator(Indicator::Deleted),
                text!(stats.deletions().to_string())
            )
            .with_status(Status::Error),
        );
    }

    if !children.is_empty() {
        children = vec![label!(Node::Block(children)), spacer!()];
    }

    ui.renderln(&continued!(block!(
        text!("Created"),
        spacer!(),
        Node::Block(children)
    )))?;

    Ok(())
}

pub fn with_prefix(prefix: &str, repo: Repo, mut opts: Opts) -> Result<(), Box<dyn Error>> {
    opts.message = format!("{prefix}: {}", opts.message);
    run(repo, opts)
}
