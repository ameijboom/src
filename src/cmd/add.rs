use std::{error::Error, path::Path};

use clap::Parser;
use colored::Colorize;
use git2::{IndexAddOption, Repository};

#[derive(Parser)]
pub struct Opts {
    #[clap()]
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
    if opts.targets.is_empty() {
        return Err("No files specified".into());
    }

    let mut count = 0;
    let mut index = repo.index()?;
    index.add_all(
        opts.targets,
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
