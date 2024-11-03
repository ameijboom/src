use std::{
    error::Error,
    io::{self, Write},
    process::{Command, Stdio},
};

use clap::{Parser, ValueHint};
use git2::{Diff, DiffFormat};
use which::which;

use crate::git::{DiffOpts, Repo};

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
#[clap(about = "Show changes")]
pub struct Opts {
    #[clap(short, long, default_value = "false")]
    pub patch: bool,

    #[clap(value_hint = ValueHint::AnyPath)]
    pub filter: Option<String>,

    #[clap(short, long)]
    pub staged: bool,
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let tree = head.find_tree()?;
    let mut diff_opts = DiffOpts::default().with_staged(opts.staged);

    if let Some(filter) = opts.filter {
        diff_opts = diff_opts.with_pathspec(&filter);
    }

    let diff = repo.diff(&tree, diff_opts)?;

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
