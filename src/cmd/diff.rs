use std::{
    error::Error,
    fmt::Write as _,
    io::{self, Write},
    process::{Command, Stdio},
};

use clap::{Parser, ValueHint};
use git2::{Diff, DiffFormat};
use minus::Pager;
use which::which;

use crate::{
    git::{DiffOpts, Optional, Repo},
    term::render,
};

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

    #[clap(long, help = "Disable the pager")]
    no_pager: bool,

    #[clap(short, long)]
    pub staged: bool,

    #[clap(short, long)]
    pub all: bool,
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let tree = head.find_tree()?;
    let mut diff_opts = DiffOpts::default();

    if opts.staged {
        diff_opts = diff_opts.with_staged(&tree);
    }

    if opts.all {
        diff_opts = diff_opts.with_all(&tree);
    }

    let (diff, dst) = if let Some(filter) = opts.filter {
        if let Some(reference) = repo.find_ref_by_shortname(&filter).optional()? {
            let tree = reference.find_tree()?;
            (repo.diff(diff_opts.with_all(&tree))?, Some(reference))
        } else {
            (repo.diff(diff_opts.with_pathspec(&filter))?, None)
        }
    } else {
        (repo.diff(diff_opts)?, None)
    };

    if opts.patch {
        print_patch(&diff)?;
        return Ok(());
    }

    match which("delta") {
        Ok(path) => {
            let mut child = Command::new(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;
            let stdin = child.stdin.as_mut().unwrap();

            write_diff(&diff, &mut *stdin)?;
            stdin.flush()?;

            let output = child.wait_with_output()?;
            let stdout = String::from_utf8(output.stdout)?;

            if opts.no_pager {
                println!("{}", stdout);
                return Ok(());
            }

            let summary = match dst {
                Some(dst) => format!("{}..{}", head.shorthand()?, dst.shorthand()?),
                None => format!("at {}", render::reference(&head)?.into_inner()),
            };
            let mut pager = Pager::new();

            pager.set_prompt(format!("diff {}, q to quit", summary))?;
            pager.write_str(&stdout)?;

            minus::page_all(pager)?;
        }
        Err(_) => print_patch(&diff)?,
    }

    Ok(())
}
