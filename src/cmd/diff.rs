use std::{
    error::Error,
    fmt::Write as _,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    thread,
};

use clap::{Parser, ValueHint};
use git2::{Diff, DiffFormat};
use minus::Pager;
use which::which;

use crate::{
    git::{DiffOpts, Optional, Repo},
    term::render,
};

fn render_diff(diff: &Diff) -> Result<Vec<u8>, git2::Error> {
    let mut output = vec![];

    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or_default();
        let _ = match line.origin() {
            '+' => write!(output, "+{content}"),
            '-' => write!(output, "-{content}"),
            ' ' => write!(output, " {content}"),
            'F' | 'H' => write!(output, "{content}"),
            _ => write!(output, "{content}"),
        };

        true
    })?;

    Ok(output)
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
        println!("{}", String::from_utf8(render_diff(&diff)?)?);
        return Ok(());
    }

    match which("delta") {
        Ok(path) => {
            let mut child = Command::new(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;

            if opts.no_pager {
                let stdin = child.stdin.as_mut().unwrap();
                stdin.write_all(&render_diff(&diff)?)?;
                stdin.flush()?;

                let output = child.wait_with_output()?;
                println!("{}", String::from_utf8(output.stdout)?);

                return Ok(());
            }

            let pager = Pager::new();
            let summary = match dst {
                Some(dst) => format!("{}..{}", head.shorthand()?, dst.shorthand()?),
                None => format!("at {}", render::reference(&head)?.into_inner()),
            };

            pager.set_prompt(format!("diff {}, q to quit", summary))?;

            let mut stdin = child.stdin.take().unwrap();
            let diff = render_diff(&diff)?;

            thread::spawn(move || {
                stdin.write_all(&diff)?;
                stdin.flush()
            });

            let mut p = pager.clone();
            thread::spawn(move || {
                let stdout = BufReader::new(child.stdout.unwrap());
                let mut lines = stdout.lines();

                while let Some(Ok(line)) = lines.next() {
                    let _ = writeln!(p, "{}", line);
                }
            });

            minus::dynamic_paging(pager)?;
        }
        Err(_) => println!("{}", String::from_utf8(render_diff(&diff)?)?),
    }

    Ok(())
}
