use std::{
    borrow::Cow,
    env,
    error::Error,
    str::{FromStr, Utf8Error},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use git2::{Cred, Direction, FetchOptions, Oid, PushOptions, RemoteCallbacks};
use http::Uri;
use regex::Regex;
use ssh2_config::{ParseRule, SshConfig};

use crate::term::bar::Bar;

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

#[derive(Default)]
struct Progress {
    total: AtomicUsize,
    current: AtomicUsize,
}

struct State<'a> {
    bar: &'a mut Bar,
    pack: Progress,
    push: Progress,
    count: Progress,
    compress: Progress,
    resolve: Progress,
}

impl<'a> State<'a> {
    fn new(bar: &'a mut Bar) -> Self {
        Self {
            bar,
            pack: Progress::default(),
            push: Progress::default(),
            count: Progress::default(),
            compress: Progress::default(),
            resolve: Progress::default(),
        }
    }

    fn progress(&self) -> (usize, usize) {
        let mut total = 0;
        let mut current = 0;

        for progress in [
            &self.pack,
            &self.push,
            &self.count,
            &self.compress,
            &self.resolve,
        ] {
            total += progress.total.load(Ordering::Relaxed);
            current += progress.current.load(Ordering::Relaxed);
        }

        (current, total)
    }

    fn update(&self, message: impl Into<Cow<'static, str>>) {
        let (current, total) = self.progress();

        self.bar.update(current, total);
        self.bar.set_message(message.into());
    }
}

fn parse_sideband_progress(re: &Regex, line: &[u8]) -> Option<(String, usize, usize)> {
    if let Ok(line) = std::str::from_utf8(line) {
        if let Some(captures) = re.captures(line) {
            if let (Some(kind), Some(current), Some(total)) =
                (captures.get(1), captures.get(2), captures.get(3))
            {
                return Some((
                    kind.as_str().to_string(),
                    current.as_str().parse().unwrap_or(0),
                    total.as_str().parse().unwrap_or(0),
                ));
            }
        }
    }

    None
}

#[allow(dead_code)]
pub struct Update {
    pub src: Oid,
    pub dst: Oid,
    pub refname: String,
}

pub struct RemoteOpts {
    bar: Bar,
    stdout: Vec<u8>,
    compare: Option<Oid>,
    updates: Vec<Update>,
}

impl Default for RemoteOpts {
    fn default() -> Self {
        Self {
            bar: Bar::default(),
            stdout: vec![],
            compare: None,
            updates: vec![],
        }
    }
}

impl RemoteOpts {
    pub fn with_bar(bar: Bar) -> Self {
        Self {
            bar,
            ..RemoteOpts::default()
        }
    }

    pub fn with_compare(mut self, compare: Oid) -> Self {
        self.compare = Some(compare);
        self
    }

    pub fn callbacks(&mut self) -> RemoteCallbacks<'_> {
        let stdout = &mut self.stdout;
        let mut callbacks = RemoteCallbacks::new();
        let global_state = Arc::new(State::new(&mut self.bar));

        callbacks.credentials(|url, username, _| get_credentials(url, username));

        let state = Arc::clone(&global_state);
        let re = Regex::new(
            r"(Counting|Compressing|Resolving) [A-Za-z]+:[ ]+[0-9]+% \(([0-9]+)\/([0-9]+)\)",
        )
        .expect("invalid regex");

        callbacks.sideband_progress(move |line| {
            if let Some((kind, current, total)) = parse_sideband_progress(&re, line) {
                match kind.as_str() {
                    "Counting" => {
                        state.count.current.store(current, Ordering::Relaxed);
                        state.count.total.store(total, Ordering::Relaxed);
                    }
                    "Compressing" => {
                        state.compress.current.store(current, Ordering::Relaxed);
                        state.compress.total.store(total, Ordering::Relaxed);
                    }
                    "Resolving" => {
                        state.resolve.current.store(current, Ordering::Relaxed);
                        state.resolve.total.store(total, Ordering::Relaxed);
                    }
                    _ => {}
                }

                state.update(kind);
            } else {
                stdout.extend_from_slice(line);
            }

            true
        });

        let state = Arc::clone(&global_state);

        callbacks.pack_progress(move |_stage, current, total| {
            state.pack.current.store(current, Ordering::Relaxed);
            state.pack.total.store(total, Ordering::Relaxed);
            state.update("Packing");
        });

        let state = Arc::clone(&global_state);

        callbacks.push_transfer_progress(move |current, total, _bytes| {
            state.push.current.store(current, Ordering::Relaxed);
            state.push.total.store(total, Ordering::Relaxed);
            state.update("Pushing");
        });

        callbacks.push_negotiation(|updates| {
            if let Some(oid) = self.compare {
                if !updates.iter().any(|upd| upd.src() == oid)
                    && !updates.iter().all(|upd| upd.src().is_zero())
                {
                    return Err(git2::Error::new(
                        git2::ErrorCode::User,
                        git2::ErrorClass::None,
                        "update rejected (outdated)",
                    ));
                }
            }

            Ok(())
        });

        callbacks.update_tips(|name, src, dst| {
            self.updates.push(Update {
                src,
                dst,
                refname: name.to_string(),
            });

            true
        });

        callbacks
    }

    pub fn into_reply(self) -> Reply {
        Reply {
            stdout: self.stdout,
            updates: self.updates,
        }
    }
}

pub struct Reply {
    pub stdout: Vec<u8>,
    #[allow(dead_code)]
    pub updates: Vec<Update>,
}

pub struct Remote<'a>(pub git2::Remote<'a>);

impl<'a> From<git2::Remote<'a>> for Remote<'a> {
    fn from(remote: git2::Remote<'a>) -> Self {
        Self(remote)
    }
}

impl<'a> Remote<'a> {
    pub fn name(&self) -> Result<Option<&str>, Utf8Error> {
        self.0.name_bytes().map(std::str::from_utf8).transpose()
    }

    pub fn url(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.0.url_bytes())
    }

    pub fn default_branch(&self) -> Result<String, Box<dyn Error>> {
        Ok(String::from_utf8(self.0.default_branch()?.to_vec())?)
    }

    pub fn fetch(&mut self, mut opts: RemoteOpts, refspec: &str) -> Result<Reply, git2::Error> {
        let callbacks = opts.callbacks();

        self.0.fetch(
            &[refspec],
            Some(FetchOptions::new().remote_callbacks(callbacks).depth(0)),
            None,
        )?;

        Ok(opts.into_reply())
    }

    pub fn push(&mut self, mut opts: RemoteOpts, refspec: &str) -> Result<Reply, git2::Error> {
        let callbacks = opts.callbacks();

        self.0.push(
            &[refspec],
            Some(
                PushOptions::new()
                    .remote_callbacks(callbacks)
                    .packbuilder_parallelism(0),
            ),
        )?;

        Ok(opts.into_reply())
    }

    pub fn connect(&mut self, mut opts: RemoteOpts) -> Result<Reply, git2::Error> {
        let callbacks = opts.callbacks();
        self.0
            .connect_auth(Direction::Fetch, Some(callbacks), None)?;

        Ok(opts.into_reply())
    }
}
