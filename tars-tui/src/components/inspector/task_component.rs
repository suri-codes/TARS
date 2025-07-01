use async_trait::async_trait;
use color_eyre::Result;
use common::types::Task;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Modifier, Style},
    widgets::Widget,
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::TextArea;

use crate::{action::Action, components::Component};

#[derive(Debug)]
pub struct TaskComponent<'a> {
    name: TarsText<'a>,
}

#[derive(Debug, Default)]
pub enum EditMode {
    Editing,
    #[default]
    Inactive,
}

#[derive(Debug)]
struct TarsText<'a> {
    textarea: TextArea<'a>,
    mode: EditMode,
}

impl<'a> TarsText<'a> {
    pub fn deactivate(&mut self) {
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_cursor_style(Style::default());
    }

    pub fn activate(&mut self) {
        self.textarea
            .set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        self.textarea
            .set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }
}

impl<'a> TaskComponent<'a> {
    pub fn new(task: &Task) -> Self {
        let mut name = TextArea::from(vec![task.name.as_str()]);
        name.set_cursor_line_style(Style::default());
        // name.set_placeholder_text(task.name.as_str());

        let mut component = Self {
            name: ,
            mode: EditMode::default(),
        };

        component.name.deactivate();
        component
    }
}

#[async_trait]
impl Component for TaskComponent<'_> {
    fn init(
        &mut self,

        _area: ratatui::prelude::Size,
        _default_mode: crate::app::Mode,
    ) -> color_eyre::eyre::Result<()> {
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::SwitchTo(_) = action {
            self.deactivate()
        }

        Ok(None)
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::eyre::Result<()> {
        frame.render_widget(&self.name, area);
        Ok(())
    }
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Char('n') => {
                self.activate();
            }

            KeyCode::Esc => {
                self.deactivate();
            }

            _ => {}
        }

        Ok(None)
    }
}
