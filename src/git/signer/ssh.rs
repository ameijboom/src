use std::{
    io::Write,
    process::{Command, Stdio},
};

use git2::Config;
use tempfile::NamedTempFile;

use crate::git::Optional;

use super::Signer;

pub struct SshSigner {
    signing_key: String,
    program: Option<String>,
}

impl SshSigner {
    pub fn new(signing_key: String, program: Option<String>) -> Self {
        Self {
            signing_key,
            program,
        }
    }

    pub fn from_config(config: &Config) -> Result<Self, git2::Error> {
        let program = config.get_string("gpg.ssh.program").optional()?;
        let signing_key = config.get_string("user.signingkey")?;

        Ok(Self::new(signing_key, program))
    }
}

impl Signer for SshSigner {
    fn sign(&self, content: &git2::Buf) -> Result<String, Box<dyn std::error::Error>> {
        // Aparently, we have to write this to a file
        let mut tmp = NamedTempFile::new()?;
        tmp.write_all(self.signing_key.as_bytes())?;
        tmp.flush()?;

        let program = self.program.as_deref().unwrap_or("ssh");

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

        if !output.status.success() {
            return Err(format!("failed to sign: {}", String::from_utf8(output.stderr)?).into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }
}
