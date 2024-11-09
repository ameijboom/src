use std::error::Error;

use clap::Parser;
use inquire::{
    ui::{Color, RenderConfig},
    Confirm,
};

use crate::{cmd::add::add_callback, git::Repo, term::render};

#[derive(Parser)]
#[clap(about = "Amend recorded changes to the repository")]
pub struct Opts {
    #[clap(short, long, help = "Add all changes")]
    add_all: bool,

    #[clap(short, long, help = "Amend without prompting")]
    yes: bool,

    #[clap(help = "Commit message")]
    message: Option<String>,
}

pub fn run(repo: Repo, opts: Opts) -> Result<(), Box<dyn Error>> {
    let mut index = repo.index()?;

    if opts.add_all {
        index.add(["."], add_callback)?;
        index.write()?;
    }

    let oid = index.write_tree()?;
    let mut head = repo.head()?;
    let tree = repo.find_tree(oid)?;
    let commit = head.find_commit()?;
    let message = commit.message().unwrap_or_default().to_string();

    if !opts.yes {
        println!(
            "{}\n\n{}\n",
            commit
                .headers_formatted()
                .with_color(colored::Color::BrightBlack),
            commit.message_formatted()
        );

        let mut config = RenderConfig::default_colored();
        config.prompt.fg = Some(Color::LightCyan);

        if !Confirm::new("Amend this commit?")
            .with_default(false)
            .with_render_config(config)
            .prompt()?
        {
            return Ok(());
        }
    }

    let parent = commit.parent()?.ok_or("unable to amend empty commit")?;
    let message = opts.message.as_deref().unwrap_or(&message);
    let oid = repo.create_commit(&tree, message, Some(&parent))?;

    drop(commit);
    drop(parent);

    head.set_target(oid, &format!("commit amended: {message}"))?;

    println!("Created {}", render::commit(oid));

    Ok(())
}
