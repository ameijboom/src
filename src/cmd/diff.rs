use std::error::Error;

use clap::Parser;
use colored::Colorize;
use git2::{DiffFindOptions, DiffFormat, DiffOptions, Repository};

#[derive(Parser)]
pub struct Opts {}

pub fn run(repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let tree = head.peel_to_tree()?;

    let mut diff_opts = DiffOptions::new();
    let mut diff = repo.diff_tree_to_workdir_with_index(
        Some(&tree),
        Some(
            diff_opts
                .force_text(true)
                .ignore_whitespace(true)
                .ignore_whitespace_change(false)
                .include_ignored(false)
                .include_untracked(true)
                .recurse_untracked_dirs(true),
        ),
    )?;

    let mut find_opts = DiffFindOptions::new();
    diff.find_similar(Some(find_opts.renames(true).copies(true)))?;

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or_default();

        match line.origin() {
            '+' => print!("{}", format!("+{content}").green()),
            '-' => print!("{}", format!("-{content}").red()),
            ' ' => print!(" {}", content),
            _ => print!("{}", content),
        }

        true
    })?;

    Ok(())
}
