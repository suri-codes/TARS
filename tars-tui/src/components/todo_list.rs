use async_trait::async_trait;
use common::types::Task;
use ratatui::widgets::Paragraph;
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Action, app::Mode, config::Config};

use super::{Component, frame_block};

#[derive(Default)]
pub struct TodoList {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    active: bool,
}

impl TodoList {
    pub fn new() -> Self {
        Self::default()
    }

    fn mode(&self) -> Mode {
        Mode::TodoList
    }
}

#[async_trait]
impl Component for TodoList {
    fn init(
        &mut self,
        _area: ratatui::prelude::Size,
        default_mode: Mode,
    ) -> color_eyre::eyre::Result<()> {
        if default_mode == self.mode() {
            self.active = true
        }

        Ok(())
    }
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

    async fn update(&mut self, action: Action) -> color_eyre::eyre::Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            Action::SwitchTo(Mode::TodoList) => self.active = true,
            Action::SwitchTo(_) => self.active = false,
            _ => {}
        }
        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        frame.render_widget(
            Paragraph::new("penis").block(frame_block(self.active, self.mode())),
            area,
        );
        Ok(())
    }
}
