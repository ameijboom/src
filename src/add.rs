use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::{IndexAddOption, Repository};

#[derive(Parser)]
pub struct Opts {
    #[clap()]
    targets: Vec<String>,
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    let mut index = repo.index()?;
    index.add_all(
        opts.targets,
        IndexAddOption::DEFAULT,
        Some(&mut |path, _| {
            count += 1;
            println!(
                "{} {}",
                "+".green().bold(),
                path.to_str().unwrap_or_default().green()
            );
            0
        }),
    )?;
    index.write()?;

    if count > 0 {
        println!("{} file(s) added", count.to_string().bold());
    }

    Ok(())
}
