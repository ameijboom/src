use std::error::Error;

use clap::Parser;

use crate::{
    git::Repo,
    term::ui::{self, Icon},
};

#[derive(Parser)]
#[clap(about = "Apply the changes from the last stash")]
pub struct Opts {
    #[clap(short = 'n', long, default_value = "0")]
    index: usize,
}

pub fn run(mut repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    repo.pop_stash(opts.index)?;
    println!("{}", ui::message_with_icon(Icon::Check, "Changes applied"));

    Ok(())
}
