use inquire::{error::InquireResult, ui::RenderConfig, Confirm};

pub mod bar;
pub mod node;
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
