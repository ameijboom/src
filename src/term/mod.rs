use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
    time::Instant,
};

use inquire::{error::InquireResult, ui::RenderConfig, Confirm};
use progress::ProgressBar;

use crate::git::{ProgressEvent, SidebandOp};

pub mod node;
pub mod progress;
pub mod render;
pub mod select;

pub fn confirm(prompt: &str) -> InquireResult<bool> {
    let mut config = RenderConfig::default_colored();
    config.prompt.fg = Some(inquire::ui::Color::LightCyan);

    Confirm::new(prompt)
        .with_default(false)
        .with_render_config(config)
        .prompt()
}

pub fn setup_progress_bar(rx: Receiver<ProgressEvent>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut now = Instant::now();
        let mut bar = ProgressBar::with_multiple(vec!["Remote", "Transfer", "Packing"]);

        bar.draw();

        for event in rx {
            match event {
                ProgressEvent::Transfer(current, total) => {
                    bar.set_message(1, format!("{current}/{total} objects"));
                    bar.set_progress(1, current, total);
                }
                ProgressEvent::PushTransfer(bytes, current, total) => {
                    bar.set_message(1, format!("{bytes} bytes"));
                    bar.set_progress(1, current, total);
                }
                ProgressEvent::Packing(current, total) => bar.set_progress(2, current, total),
                ProgressEvent::Sideband(op, current, total) => {
                    bar.set_progress(0, current, total);

                    match op {
                        SidebandOp::Counting => {
                            bar.set_message(0, format!("counting ({current}/{total} objects)"))
                        }
                        SidebandOp::Compressing => {
                            bar.set_message(0, format!("compressing ({current}/{total} objects)"))
                        }
                        SidebandOp::Resolving => {
                            bar.set_message(0, format!("resolving ({current}/{total} objects)"))
                        }
                    }
                }
            }

            if now.elapsed().as_millis() < 50 {
                continue;
            }

            now = Instant::now();
            bar.draw();
        }

        bar.clear();
    })
}
