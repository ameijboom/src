use std::{error::Error, path::Path};

use clap::{Parser, ValueHint};
use colored::Colorize;
use git2::Repository;

use crate::{
    git::{index::Index, status::Status},
    select,
};

#[derive(Parser)]
#[clap(about = "Add file contents to the index")]
pub struct Opts {
    #[clap(value_hint = ValueHint::AnyPath)]
    targets: Vec<String>,
}

pub fn add_callback(path: &Path) {
    println!(
        "{} {}",
        "+".green().bold(),
        path.to_str().unwrap_or_default().green()
    );
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let targets = if opts.targets.is_empty() {
        let files = Status::build(&repo)?
            .entries()
            .map(|p| p.path().map(|p| p.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        select::multi(&files)?
    } else {
        opts.targets
    };

    let mut index = Index::build(&repo)?;
    let count = index.add(targets.iter(), add_callback)?;
    index.write()?;

    if count > 0 {
        println!("{} file(s) added", count.to_string().bold());
    }

    Ok(())
}
