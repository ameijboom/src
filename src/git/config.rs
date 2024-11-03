use std::collections::HashMap;

use super::Optional;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("invalid utf8: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("invalid gpg format: {0}")]
    InvalidGpgFormat(String),
}

pub enum GpgFormat {
    Ssh,
}

pub struct Commit {
    pub gpg_sign: bool,
}

#[derive(Default)]
pub struct GpgConfig {
    pub program: Option<String>,
}

pub struct Gpg {
    pub format: Option<GpgFormat>,
    pub config: HashMap<String, GpgConfig>,
}

pub struct User {
    pub name: Option<String>,
    pub signing_key: Option<String>,
    pub email: String,
}

impl User {
    pub fn signature(&self) -> Result<git2::Signature<'_>, git2::Error> {
        git2::Signature::now(self.name.as_deref().unwrap_or_default(), &self.email)
    }
}

pub struct Push {
    pub auto_setup_remote: bool,
}

pub struct Config {
    pub commit: Commit,
    pub gpg: Gpg,
    pub user: User,
    pub push: Push,
}

impl Config {
    pub fn open_default() -> Result<Self, Error> {
        git2::Config::open_default()?.try_into()
    }
}

fn bool_or_default(config: &git2::Config, name: &str) -> Result<bool, git2::Error> {
    Ok(config.get_bool(name).optional()?.unwrap_or(false))
}

fn string(config: &git2::Config, name: &str) -> Result<Option<String>, git2::Error> {
    config.get_string(name).optional()
}

fn parse_gpg_config(config: &git2::Config) -> Result<HashMap<String, GpgConfig>, Error> {
    let mut gpg = HashMap::new();
    let mut entries = config.entries(Some("gpg.*"))?;

    while let Some(entry) = entries.next() {
        let entry = entry?;
        let name = std::str::from_utf8(entry.name_bytes())?;
        let components = name.split('.').collect::<Vec<_>>();

        if components.len() != 3 {
            continue;
        }

        let value: &mut GpgConfig = gpg.entry(components[1].to_string()).or_default();

        match components[2] {
            "program" => value.program = string(config, name)?,
            _ => (),
        }
    }

    Ok(gpg)
}

impl TryFrom<git2::Config> for Config {
    type Error = Error;

    fn try_from(config: git2::Config) -> Result<Self, Self::Error> {
        Ok(Self {
            gpg: Gpg {
                format: string(&config, "gpg.format")?
                    .map(|format| match format.as_str() {
                        "ssh" => Ok(GpgFormat::Ssh),
                        format => Err(Error::InvalidGpgFormat(format.to_string())),
                    })
                    .transpose()?,
                config: parse_gpg_config(&config)?,
            },
            commit: Commit {
                gpg_sign: bool_or_default(&config, "commit.gpgsign")?,
            },
            user: User {
                name: string(&config, "user.name")?,
                email: config.get_string("user.email")?,
                signing_key: string(&config, "user.signingkey")?,
            },
            push: Push {
                auto_setup_remote: bool_or_default(&config, "push.autoSetupRemote")?,
            },
        })
    }
}
