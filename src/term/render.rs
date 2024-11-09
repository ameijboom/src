use std::error::Error;

use colored::{ColoredString, Colorize};

use crate::git::Ref;

pub fn remote(name: &str) -> ColoredString {
    format!("⬡ {name}").cyan()
}

pub fn branch(name: &str) -> ColoredString {
    format!(" {name}").purple()
}

pub fn commit(oid: git2::Oid) -> ColoredString {
    format!("{oid}")
        .chars()
        .take(7)
        .collect::<String>()
        .yellow()
}

pub fn reference(reference: &Ref<'_>) -> Result<ColoredString, Box<dyn Error>> {
    if reference.is_branch() || reference.is_tag() {
        Ok(branch(reference.shorthand()?))
    } else {
        match reference.target() {
            Some(oid) => Ok(commit(oid)),
            None => Ok("<unknown>".bright_black()),
        }
    }
}
