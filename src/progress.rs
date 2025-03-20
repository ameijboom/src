use std::sync::Arc;

use prodash::{
    render::line::JoinHandle,
    tree::{root::Options, Root},
};

pub fn tree() -> Arc<Root> {
    Arc::new(
        Options {
            message_buffer_capacity: 200,
            ..Default::default()
        }
        .into(),
    )
}

pub fn setup_line_renderer(progress: &Arc<Root>) -> JoinHandle {
    prodash::render::line(
        std::io::stderr(),
        std::sync::Arc::downgrade(progress),
        prodash::render::line::Options {
            level_filter: Some(2..=2),
            frames_per_second: 6.0,
            initial_delay: None,
            timestamp: false,
            throughput: true,
            hide_cursor: false,
            ..prodash::render::line::Options::default()
        }
        .auto_configure(prodash::render::line::StreamKind::Stderr),
    )
}
