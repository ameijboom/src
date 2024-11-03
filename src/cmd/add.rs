use std::{error::Error, path::Path};

use clap::{Parser, ValueHint};
use colored::Colorize;

use crate::{git::Repo, select};

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

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let targets = if opts.targets.is_empty() {
        let files = repo
            .status()?
            .entries()
            .map(|p| p.path().map(|p| p.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        select::multi(&files)?
    } else {
        opts.targets
    };

    let mut index = repo.index()?;

    let count = index.add(targets, add_callback)?;
    index.write()?;

    if count > 0 {
        println!("{} file(s) added", count.to_string().bold());
    }

    Ok(())
}
