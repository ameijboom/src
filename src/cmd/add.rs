use std::{error::Error, path::Path};

use clap::{Parser, ValueHint};

use crate::{
    git::Repo,
    term::{
        node::prelude::*,
        render::{Render, TermRenderer},
        select,
    },
};

#[derive(Parser)]
#[clap(about = "Add file contents to the index")]
pub struct Opts {
    #[clap(value_hint = ValueHint::AnyPath)]
    targets: Vec<String>,
}

fn file_added(path: &Path) -> Node {
    block!(
        Node::Indicator(Indicator::New),
        spacer!(),
        text!(path.to_str().unwrap_or_default().to_string())
    )
    .with_status(Status::Success)
}

pub fn add_callback(path: &Path) {
    let _ = TermRenderer::default().renderln(&file_added(path));
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let targets = if opts.targets.is_empty() {
        let files = repo
            .status()?
            .entries()
            .map(|p| p.path().map(|p| p.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        select::multi(&files, Some("src diff {} --all"))?
    } else {
        opts.targets
    };

    if targets.is_empty() {
        return Err("No targets specified".into());
    }

    let mut index = repo.index()?;

    let count = index.add(targets, add_callback)?;
    index.write()?;

    if count > 0 {
        println!("{} file(s) added", count);
    }

    Ok(())
}
