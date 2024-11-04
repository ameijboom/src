use std::error::Error;

use clap::Parser;
use colored::Colorize;

use crate::{
    cmd::add::add_callback,
    git::{DiffOpts, Repo},
    utils,
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
            prefix.trim().replace(' ', "-"),
            name.trim().replace(' ', "-")
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

    let diff = repo.diff(&old_tree, DiffOpts::default())?;
    let stats = diff.stats()?;
    let mut indicators = vec![];

    if stats.insertions() > 0 {
        indicators.push(format!("+{}", stats.insertions()).green().to_string());
    }

    if stats.deletions() > 0 {
        indicators.push(format!("-{}", stats.deletions()).red().to_string());
    }

    println!(
        "Created {} {}{}{}",
        utils::short_hash(oid).yellow(),
        "(".bright_black(),
        indicators.join(" "),
        ")".bright_black()
    );

    Ok(())
}
