use std::error::Error;

use git2::Buf;

pub mod ssh;

pub trait Signer {
    fn sign(&self, content: &Buf) -> Result<String, Box<dyn Error>>;
}
