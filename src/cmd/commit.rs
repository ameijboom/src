use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
};

use clap::Parser;
use colored::Colorize;
use git2::{Buf, Config, IndexAddOption, Repository, Signature};
use tempfile::NamedTempFile;

use crate::{cmd::add::add_callback, utils};

#[derive(Parser)]
#[clap(about = "Record changes to the repository")]
pub struct Opts {
    #[clap(short, long, help = "Add all changes")]
    add_all: bool,

    #[clap(help = "Commit message")]
    message: String,
}

fn sign_commit(config: &Config, content: &Buf) -> Result<String, Box<dyn Error>> {
    match utils::config_opt(config.get_string("gpg.format"))?.as_deref() {
        Some("ssh") => {
            // Aparently, we have to write this to a file
            let key_file = config.get_string("user.signingkey")?;
            let mut tmp = NamedTempFile::new()?;
            tmp.write_all(key_file.as_bytes())?;
            tmp.flush()?;

            let program = utils::config_opt(config.get_string("gpg.ssh.program"))?
                .unwrap_or("ssh".to_string());

            // See: https://github.com/git/git/blob/34b6ce9b30747131b6e781ff718a45328aa887d0/gpg-interface.c#L1072
            let mut child = Command::new(program)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .args(["-Y", "sign", "-n", "git", "-f"])
                .arg(tmp.path())
                .spawn()?;

            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all(content)?;
            stdin.flush()?;

            let output = child.wait_with_output()?;
            Ok(String::from_utf8(output.stdout)?)
        }
        Some(format) => Err(format!("Unsupported gpg.format: {format}").into()),
        None => Err("gpg.format not set".into()),
    }
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut index = repo.index()?;

    if opts.add_all {
        index.add_all(
            ["."].iter(),
            IndexAddOption::DEFAULT,
            Some(&mut add_callback),
        )?;
        index.write()?;
    }

    let mut head = repo.head()?;
    let oid = index.write_tree()?;
    let tree = repo.find_tree(oid)?;

    let config = Config::open_default()?;
    let name = utils::config_opt(config.get_string("user.name"))?.unwrap_or_default();
    let email = config.get_string("user.email")?;
    let author = Signature::now(&name, &email)?;

    let oid = if utils::config_opt(config.get_bool("commit.gpgsign"))?.unwrap_or(false) {
        let content = repo.commit_create_buffer(
            &author,
            &author,
            &opts.message,
            &tree,
            &[&head.peel_to_commit()?],
        )?;
        let signature = sign_commit(&config, &content)?;
        let id = repo.commit_signed(content.as_str().unwrap_or_default(), &signature, None)?;

        head.set_target(id, &opts.message)?;
        id
    } else {
        repo.commit(
            Some("HEAD"),
            &author,
            &author,
            &opts.message,
            &tree,
            &[&head.peel_to_commit()?],
        )?
    };

    println!("Created {}", utils::short(&oid).yellow());

    Ok(())
}
