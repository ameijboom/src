use std::error::Error;

use clap::Parser;
use git2::{build::CheckoutBuilder, Repository, StashApplyOptions};

#[derive(Parser)]
#[clap(about = "Apply the changes from the last stash")]
pub struct Opts {
    #[clap(short = 'n', long, default_value = "0")]
    index: usize,
}

pub fn run(mut repo: Repository, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut cb = CheckoutBuilder::default();
    cb.safe();

    repo.stash_pop(
        opts.index,
        Some(StashApplyOptions::default().checkout_options(cb)),
    )?;

    println!("✓ Changes applied");

    Ok(())
}
