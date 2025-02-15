use std::{env::current_dir, error::Error};

use clap::Parser;
use git2::{build::RepoBuilder, FetchOptions};

use crate::{
    git::RemoteOpts,
    term::{
        node::prelude::*,
        render::{Render, TermRenderer},
        setup_progress_bar,
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

    let (tx, rx) = std::sync::mpsc::channel();
    setup_progress_bar(rx);

    let mut remote = RemoteOpts::default().with_progress(tx);
    let mut fetch_opts = FetchOptions::new();

    fetch_opts.remote_callbacks(remote.callbacks()).depth(0);

    RepoBuilder::new()
        .fetch_options(fetch_opts)
        .clone(&uri, &path)?;

    remote.into_reply();

    let mut ui = TermRenderer::default();
    ui.renderln(&message_with_icon(
        Icon::Check,
        format!("Repository cloned to {}", path.display()),
    ))?;

    Ok(())
}
