use std::{env, error::Error, str::FromStr};

use clap::Parser;
use colored::Colorize;
use git2::{Cred, PushOptions, RemoteCallbacks, Repository};
use http::Uri;
use ssh2_config::{ParseRule, SshConfig};

use crate::utils;

#[derive(Parser)]
pub struct Opts {}

fn get_credentials(url: &str, username: Option<&str>) -> Result<Cred, git2::Error> {
    let mut username = username.unwrap_or_default().to_string();

    if let Ok(config) = SshConfig::parse_default_file(ParseRule::ALLOW_UNKNOWN_FIELDS) {
        if let Ok(uri) = Uri::from_str(&format!("git://{url}")) {
            let params = uri.host().map(|h| config.query(h)).unwrap_or_default();

            if let Some(user) = params.user {
                username = user;
            }

            if let Some(files) = params.identity_file {
                return Cred::ssh_key(&username, None, &files[0], None);
            }

            if let Some(agent) = params.identity_agent.as_ref().and_then(|p| p.to_str()) {
                env::set_var("SSH_AUTH_SOCK", agent);
            }
        }
    }

    if env::var("SSH_AUTH_SOCK").is_ok() {
        return Cred::ssh_key_from_agent(&username);
    }

    Cred::default()
}

pub fn run(repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let branch = head.shorthand().unwrap_or_default();
    let remote_ref = utils::find_remote_ref(&repo, head.name().unwrap_or_default())?;

    let Some(remote_name) = remote_ref
        .name()
        .unwrap_or_default()
        .strip_prefix("refs/remotes/")
        .and_then(|n| n.split('/').next())
    else {
        println!("{}", "No remote branch found".red());
        return Ok(());
    };

    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|url, username, _| get_credentials(url, username));
    callbacks.push_transfer_progress(|current, total, _bytes| {
        if !(total == 0 && current == 0) {
            println!("{}", format!("Progress: {}/{}", current, total).black());
        }
    });

    println!(
        "Pushing to: {} / {}",
        format!("⬡ {remote_name}").cyan(),
        format!(" {branch}").purple(),
    );

    let mut remote = repo.find_remote(remote_name)?;
    remote.push(
        &[head.name().unwrap_or_default()],
        Some(PushOptions::new().remote_callbacks(callbacks)),
    )?;

    Ok(())
}
