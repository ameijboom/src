use std::{
    env,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use git2::{Cred, RemoteCallbacks};
use http::Uri;
use indicatif::ProgressBar;
use ssh2_config::{ParseRule, SshConfig};

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

#[derive(Default, Clone)]
struct Progress {
    total: Arc<AtomicUsize>,
    current: Arc<AtomicUsize>,
}

pub fn remote_callbacks<'a>(
    stdout: &'a mut Vec<u8>,
    bar: &'a mut ProgressBar,
) -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();
    let pack = Progress::default();
    let push = Progress::default();
    let update = move |bar: &ProgressBar, pack: &Progress, push: &Progress| {
        let current = pack.current.load(Ordering::Relaxed) + push.current.load(Ordering::Relaxed);
        let total = pack.total.load(Ordering::Relaxed) + push.total.load(Ordering::Relaxed);

        bar.set_length(total as u64);
        bar.set_position(current as u64);
    };

    callbacks.credentials(|url, username, _| get_credentials(url, username));
    callbacks.sideband_progress(|data| {
        stdout.extend_from_slice(data);
        true
    });

    let pack_copy = pack.clone();
    let bar_copy = bar.clone();
    let push_copy = push.clone();

    callbacks.pack_progress(move |_stage, current, total| {
        bar_copy.set_message("Packing");
        pack_copy.current.store(current, Ordering::Relaxed);
        pack_copy.total.store(total, Ordering::Relaxed);

        update(&bar_copy, &pack_copy, &push_copy);
    });

    callbacks.push_transfer_progress(move |current, total, _bytes| {
        bar.set_message("Pushing");
        push.current.store(current, Ordering::Relaxed);
        push.total.store(total, Ordering::Relaxed);

        update(bar, &pack, &push);
    });

    callbacks
}
