use std::{env::current_dir, error::Error};

use clap::Parser;
use git2::{build::RepoBuilder, FetchOptions};

use crate::git::RemoteOpts;

#[derive(Parser, Default)]
#[clap(about = "Clone a repository")]
pub struct Opts {
    #[clap(help = "The URI of the repository to clone")]
    uri: String,
}

pub fn run(opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut remote = RemoteOpts::default().with_message("Cloning");
    let name = opts
        .uri
        .split('/')
        .last()
        .map(|component| component.trim_end_matches(".git"))
        .unwrap_or_default();
    let path = current_dir()?.join(name);

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(remote.callbacks()).depth(0);

    RepoBuilder::new()
        .fetch_options(fetch_opts)
        .clone(&opts.uri, &path)?;

    remote.into_reply();

    println!("✓ Repository cloned to {}", path.display());
    Ok(())
}
