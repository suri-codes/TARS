use async_trait::async_trait;
use color_eyre::eyre::Result;
use common::{TarsClient, types::Group};
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

use crate::{action::Action, components::Component};

use super::{TarsText, task_component::TaskComponent};

#[derive(Debug)]
pub struct GroupComponent<'a> {
    group: Group,
    name: TarsText<'a>,
    color: TarsText<'a>,
    edit_mode: EditMode,
    client: TarsClient,
    command_tx: Option<UnboundedSender<Action>>,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum EditMode {
    #[default]
    Inactive,
    Name,
    Color,
}

impl<'a> GroupComponent<'a> {
    pub fn new(group: &Group, client: TarsClient) -> Result<Self> {
        let comp = Self {
            name: TarsText::new(
                &group.name,
                Block::new()
                    .title_top("[N]ame")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            color: TarsText::new(
                group.color.as_str(),
                Block::new()
                    .title_top("[C]olor")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            group: group.clone(),
            edit_mode: EditMode::Inactive,
            client,
            command_tx: None,
        };
        Ok(comp)
    }

    pub async fn sync(&mut self) -> Result<()> {
        let new_name = self.name.textarea.lines()[0].clone();

        if !new_name.is_empty() {
            self.group.name = new_name.into();
        };

        self.group.sync(&self.client).await?;

        Ok(())
        // todo!()
    }
}

#[async_trait]
impl Component for GroupComponent<'_> {
    fn init(
        &mut self,

        _area: ratatui::prelude::Size,
        _default_mode: crate::app::Mode,
    ) -> color_eyre::eyre::Result<()> {
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::SwitchTo(_) = action {
            self.name.deactivate()
        }
        Ok(None)
    }

    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx.clone());
        info!("received action handler");
        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        // todo!()
        Ok(None)
    }

    fn draw(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) -> Result<()> {
        frame.render_widget(Paragraph::new("lol"), area);

        let group_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Percentage(15), // name
                Constraint::Percentage(15), // color
                Constraint::Percentage(15), // parent
            ],
        )
        .split(area);

        // Group name:
        frame.render_widget(
            Paragraph::new(self.group.name.as_str()).block(
                Block::new()
                    .title_top("Name")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded),
            ),
            group_layout[0],
        );
        // Group color:
        frame.render_widget(
            Paragraph::new(self.group.color.as_str()).block(
                Block::new()
                    .title_top("Color")
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded)
                    .style(Style::new().fg(self.group.color.clone().into())),
            ),
            group_layout[1],
        );
        Ok(())
    }
}
