use std::error::Error;

use clap::Parser;

use crate::{
    cmd::add::add_callback,
    git::{DiffOpts, Repo},
    term::{
        node::{Indicator, Node, Status},
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

    let old_tree = repo.head()?.find_tree()?;
    let mut index = repo.index()?;

    if opts.add_all {
        index.add(["."], add_callback)?;
        index.write()?;
    }

    let tree = repo.find_tree(index.write_tree()?)?;
    let oid = repo.create_commit(&tree, &opts.message, None)?;

    repo.head()?
        .set_target(oid, &format!("commit: {}", opts.message))?;

    let diff = repo.diff(DiffOpts::default().with_all(&old_tree))?;
    let stats = diff.stats()?;

    let mut ui = TermRenderer::default();
    let mut children = vec![];

    if stats.insertions() > 0 {
        children.push(
            Node::Block(vec![
                Node::Indicator(Indicator::New),
                Node::Text(stats.insertions().to_string().into()),
            ])
            .with_status(Status::Success),
        );
    }

    if stats.deletions() > 0 {
        if !children.is_empty() {
            children.push(Node::spacer());
        }

        children.push(
            Node::Block(vec![
                Node::Indicator(Indicator::Deleted),
                Node::Text(stats.deletions().to_string().into()),
            ])
            .with_status(Status::Error),
        );
    }

    if !children.is_empty() {
        children = vec![Node::Label(Box::new(Node::Block(children))), Node::spacer()];
    }

    ui.renderln(&Node::Continued(Box::new(Node::Block(children))))?;

    Ok(())
}

pub fn with_prefix(prefix: &str, repo: Repo, mut opts: Opts) -> Result<(), Box<dyn Error>> {
    opts.message = format!("{prefix}: {}", opts.message);
    run(repo, opts)
}
