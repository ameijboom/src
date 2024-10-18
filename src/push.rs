use std::{env, error::Error, str::FromStr, time::Duration};

use clap::Parser;
use colored::Colorize;
use git2::{Branch, BranchType, Cred, ErrorCode, PushOptions, RemoteCallbacks, Repository};
use http::Uri;
use indicatif::ProgressBar;
use ssh2_config::{ParseRule, SshConfig};

use crate::utils;

#[derive(Debug, thiserror::Error)]
pub enum PushError {
    #[error("missing target")]
    MissingTarget,
    #[error("failed to parse remote in upstream")]
    RemoteParse,
}

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

fn set_tracking_branch(
    repo: &Repository,
    remote: &str,
    branch: &mut Branch<'_>,
) -> Result<(), Box<dyn Error>> {
    let name = branch.name()?.unwrap_or_default();
    let reference = repo.reference(
        &format!("refs/remotes/{remote}/{name}"),
        branch
            .get()
            .target()
            .ok_or_else(|| PushError::MissingTarget)?,
        true,
        "",
    )?;

    branch.set_upstream(reference.shorthand())?;

    Ok(())
}

pub fn run(repo: Repository, _opts: Opts) -> Result<(), Box<dyn Error>> {
    let head = repo.head()?;
    let head_name = head.shorthand().unwrap_or_default();
    let mut branch = repo.find_branch(head_name, BranchType::Local)?;
    let branch_name = branch.name()?.unwrap_or_default().to_string();

    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(e) if e.code() == ErrorCode::NotFound => {
            let config = repo.config()?;
            let setup_remote = config.get_bool("push.autoSetupRemote")?;

            if !setup_remote {
                println!("{}", "No remote branch found".red());
                return Ok(());
            }

            set_tracking_branch(&repo, "origin", &mut branch)?;
            branch.upstream()?
        }
        Err(e) => return Err(e.into()),
    };

    let mut out = vec![];
    let bar = ProgressBar::new_spinner();
    let mut callbacks = RemoteCallbacks::new();

    callbacks.credentials(|url, username, _| get_credentials(url, username));
    callbacks.sideband_progress(|data| {
        out.extend_from_slice(data);
        true
    });
    callbacks.pack_progress(|_stage, current, total| {
        bar.set_message("Packing");
        bar.set_length(total as u64);
        bar.set_position(current as u64);
    });
    callbacks.push_transfer_progress(|current, total, _bytes| {
        if !(total == 0 && current == 0) {
            bar.set_message("Pushing");
            bar.set_length(total as u64);
            bar.set_position(current as u64);
        }
    });

    let remote_name = utils::parse_remote(upstream.name()?.unwrap_or_default())
        .ok_or_else(|| PushError::RemoteParse)?;
    let mut remote = repo.find_remote(&remote_name)?;

    println!(
        "Pushing to: {} / {}",
        format!("⬡ {remote_name}").cyan(),
        format!(" {branch_name}").purple(),
    );

    bar.enable_steady_tick(Duration::from_millis(100));

    remote.push(
        &[head.name().unwrap_or_default()],
        Some(PushOptions::new().remote_callbacks(callbacks)),
    )?;

    bar.finish_and_clear();
    println!("✓ done");

    if let Ok(msg) = std::str::from_utf8(&out) {
        println!("\nMessage:");
        println!("{}", msg.trim().black());
    }

    Ok(())
}
