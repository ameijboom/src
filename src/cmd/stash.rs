use std::error::Error;

use clap::Parser;

use crate::{
    git::Repo,
    term::{
        node::prelude::*,
        render::{Render, TermRenderer},
    },
};

#[derive(Parser)]
#[clap(about = "Stash the changes in a dirty working directory away")]
pub struct Opts {}

pub fn run(mut repo: Repo, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let message = {
        let head = repo.head()?;
        let commit = head.find_commit()?;
        let message = commit.message().unwrap_or_default();

        format!("{} {message}", commit.id())
    };

    repo.save_stash(&message)?;

    let mut ui = TermRenderer::default();
    ui.render(&message_with_icon(Icon::Check, "Changes stashed"))?;

    Ok(())
}
