use std::error::Error;

use colored::Color;

use crate::git::Ref;

use super::fmt::FmtString;

pub fn remote(name: &str) -> FmtString {
    FmtString::new(format!("⬡ {name}")).with_color(Color::Cyan)
}

pub fn branch(name: &str) -> FmtString {
    FmtString::new(format!(" {name}")).with_color(Color::Magenta)
}

pub fn commit(oid: git2::Oid) -> FmtString {
    FmtString::new(format!("{oid}").chars().take(7).collect::<String>()).with_color(Color::Yellow)
}

pub fn reference(reference: &Ref<'_>) -> Result<FmtString, Box<dyn Error>> {
    if reference.is_branch() || reference.is_tag() {
        Ok(branch(reference.shorthand()?))
    } else {
        match reference.target() {
            Some(oid) => Ok(commit(oid)),
            None => Ok(FmtString::new("<unknown>".to_string())),
        }
    }
}
