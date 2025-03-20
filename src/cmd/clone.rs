use std::{env::current_dir, error::Error, fs};

use clap::Parser;
use gix::{bstr::BStr, progress::DoOrDiscard, remote::Direction};

use crate::{
    progress,
    term::{
        node::prelude::*,
        render::{Render, TermRenderer},
    },
};

#[derive(Parser, Default)]
#[clap(about = "Clone a repository")]
pub struct Opts {
    #[clap(help = "The URI of the repository to clone")]
    uri: String,
}

fn convert_uri(uri: &str) -> Option<String> {
    // Assuming this is a GitHub repository
    if !uri.contains('@') && !uri.contains(':') && !uri.contains("://") {
        return Some(format!("git@github.com:{uri}.git"));
    }

    None
}

pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let root = progress::tree();
    let sub_progress = root.add_child("Clone");
    let handle = progress::setup_line_renderer(&root);
    let mut progress = DoOrDiscard::from(Some(sub_progress));

    let uri = convert_uri(&opts.uri).unwrap_or(opts.uri);
    let name = uri
        .split('/')
        .last()
        .map(|component| component.trim_end_matches(".git"))
        .unwrap_or_default();

    let path = current_dir()?.join(name);

    if path.exists() {
        return Err(format!("Directory already exists: {}", path.display()).into());
    }

    fs::create_dir_all(&path)?;

    let url = gix::url::parse(BStr::new(uri.as_bytes()))?;
    let mut prepared = gix::prepare_clone(url, &path)?;
    let (mut prepare_checkout, _) =
        prepared.fetch_then_checkout(&mut progress, &gix::interrupt::IS_INTERRUPTED)?;
    let (repo, _) =
        prepare_checkout.main_worktree(&mut progress, &gix::interrupt::IS_INTERRUPTED)?;

    handle.shutdown_and_wait();

    repo.find_default_remote(Direction::Fetch)
        .transpose()?
        .ok_or("remote not present")?;

    let mut ui = TermRenderer::default();
    ui.renderln(&message_with_icon(
        Icon::Check,
        format!("Repository cloned to {}", path.display()),
    ))?;

    Ok(())
}
