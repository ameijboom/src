use std::{error::Error, io::Cursor};

use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};

pub fn single(input: &[String]) -> Result<Option<String>, Box<dyn Error>> {
    let options = SkimOptionsBuilder::default().multi(false).build()?;
    let reader = SkimItemReader::default();
    let items = reader.of_bufread(Cursor::new(input.join("\n")));

    Ok(Skim::run_with(&options, Some(items)).and_then(|out| {
        out.selected_items
            .first()
            .map(|item| item.output().to_string())
    }))
}

pub fn multi(input: &[String]) -> Result<Vec<String>, Box<dyn Error>> {
    let options = SkimOptionsBuilder::default().multi(true).build()?;
    let reader = SkimItemReader::default();
    let items = reader.of_bufread(Cursor::new(input.join("\n")));

    Ok(Skim::run_with(&options, Some(items))
        .map(|out| {
            out.selected_items
                .into_iter()
                .map(|item| item.output().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default())
}
