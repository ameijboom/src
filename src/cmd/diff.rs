use std::{
    error::Error,
    io::{self, Write},
    process::{Command, Stdio},
};

use clap::Parser;
use git2::{Diff, DiffFindOptions, DiffFormat, DiffOptions, Repository};
use which::which;

fn write_diff(diff: &Diff, mut stdout: impl Write) -> Result<(), git2::Error> {
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or_default();
        let _ = match line.origin() {
            '+' => write!(stdout, "+{content}"),
            '-' => write!(stdout, "-{content}"),
            ' ' => write!(stdout, " {content}"),
            'F' | 'H' => write!(stdout, "{content}"),
            _ => write!(stdout, "{content}"),
        };

        true
    })
}

fn print_patch(diff: &Diff) -> Result<(), git2::Error> {
    let mut stdout = io::stdout();
    write_diff(diff, &mut stdout)?;

    Ok(())
}

#[derive(Parser)]
pub struct Opts {
    #[clap(short, long, default_value = "false")]
    pub patch: bool,
    pub filter: Option<String>,
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let tree = head.peel_to_tree()?;

    let mut diff_opts = DiffOptions::new();
    diff_opts
        .force_text(true)
        .ignore_whitespace(true)
        .ignore_whitespace_change(false)
        .include_ignored(false)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);

    if let Some(filter) = opts.filter {
        diff_opts.pathspec(filter);
    }

    let mut diff = repo.diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_opts))?;

    let mut find_opts = DiffFindOptions::new();
    diff.find_similar(Some(find_opts.renames(true).copies(true)))?;

    if opts.patch {
        print_patch(&diff)?;
        return Ok(());
    }

    match which("delta") {
        Ok(path) => {
            let mut child = Command::new(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()?;
            let stdin = child.stdin.as_mut().unwrap();

            write_diff(&diff, &mut *stdin)?;

            stdin.flush()?;
            child.wait()?;
        }
        Err(_) => print_patch(&diff)?,
    }

    Ok(())
}
