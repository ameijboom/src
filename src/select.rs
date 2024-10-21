use std::{error::Error, io::Cursor};

use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};

pub fn select(input: &[String]) -> Result<Option<String>, Box<dyn Error>> {
    let options = SkimOptionsBuilder::default().multi(false).build()?;
    let reader = SkimItemReader::default();
    let items = reader.of_bufread(Cursor::new(input.join("\n")));

    Ok(Skim::run_with(&options, Some(items)).and_then(|out| {
        out.selected_items
            .first()
            .map(|item| item.output().to_string())
    }))
}
