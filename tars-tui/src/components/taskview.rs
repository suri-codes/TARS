use common::types::Task;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{action::Action, app::Mode, config::Config};

use super::{Component, frame_block};

#[derive(Default)]
pub struct TaskView {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    task: Option<Task>,
}

impl TaskView {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Component for TaskView {
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> color_eyre::eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> color_eyre::eyre::Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action) -> color_eyre::eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            _ => {}
        }
        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        mode: Mode,
    ) -> color_eyre::eyre::Result<()> {
        frame.render_widget(
            Paragraph::new("penis").block(frame_block(mode, Mode::TaskView)),
            area,
        );
        Ok(())
    }
}
