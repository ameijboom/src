use std::{error::Error, io::Cursor};

use skim::{
    prelude::{Event, SkimItemReader, SkimOptionsBuilder},
    Skim,
};

pub fn single(input: &[String], preview: Option<String>) -> Result<Option<String>, Box<dyn Error>> {
    let options = SkimOptionsBuilder::default()
        .exit_0(true)
        .multi(false)
        .preview(preview)
        .build()?;
    let reader = SkimItemReader::default();
    let items = reader.of_bufread(Cursor::new(input.join("\n")));

    Ok(Skim::run_with(&options, Some(items)).and_then(|out| {
        if out.final_event == Event::EvActAbort {
            return None;
        }

        out.selected_items
            .first()
            .map(|item| item.output().to_string())
    }))
}

pub fn multi(input: &[String], preview: Option<String>) -> Result<Vec<String>, Box<dyn Error>> {
    let options = SkimOptionsBuilder::default()
        .exit_0(true)
        .multi(true)
        .preview(preview)
        .build()?;
    let reader = SkimItemReader::default();
    let items = reader.of_bufread(Cursor::new(input.join("\n")));

    Ok(Skim::run_with(&options, Some(items))
        .map(|out| {
            if out.final_event == Event::EvActAbort {
                return vec![];
            }

            out.selected_items
                .into_iter()
                .map(|item| item.output().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default())
}
