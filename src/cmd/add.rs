use std::{error::Error, path::Path};

use clap::{Parser, ValueHint};
use colored::Colorize;
use git2::{IndexAddOption, Repository, Status};

use crate::{select, utils};

#[derive(Parser)]
#[clap(about = "Add file contents to the index")]
pub struct Opts {
    #[clap(value_hint = ValueHint::AnyPath)]
    targets: Vec<String>,
}

pub fn add_callback(path: &Path, _: &[u8]) -> i32 {
    println!(
        "{} {}",
        "+".green().bold(),
        path.to_str().unwrap_or_default().green()
    );

    0
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let targets = if opts.targets.is_empty() {
        let entries = utils::status_entries(&repo)?;
        let files = entries
            .into_iter()
            .filter(|e| e.status() != Status::CURRENT)
            .filter_map(|entry| entry.path().map(|p| p.to_string()))
            .collect::<Vec<_>>();

        select::multi(&files)?
    } else {
        opts.targets
    };

    let mut count = 0;
    let mut index = repo.index()?;

    index.add_all(
        targets,
        IndexAddOption::DEFAULT,
        Some(&mut |path, _| {
            count += 1;
            add_callback(path, &[])
        }),
    )?;
    index.write()?;

    if count > 0 {
        println!("{} file(s) added", count.to_string().bold());
    }

    Ok(())
}
