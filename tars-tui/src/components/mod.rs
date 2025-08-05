use async_trait::async_trait;
use color_eyre::Result;

use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    Frame,
    layout::{Rect, Size},
    style::{Color, Style},
    widgets::{Block, Borders},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{action::Signal, app::Mode, config::Config, tui::Event};

// pub mod fps;
// pub mod home;
pub mod explorer;
pub mod inspector;
pub mod todo_list;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
#[async_trait]
pub trait Component: Send + Sync {
    /// Register an action handler that can send actions for processing if necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn register_action_handler(&mut self, tx: UnboundedSender<Signal>) -> Result<()> {
        let _ = tx; // to appease clippy
        Ok(())
    }
    /// Register a configuration handler that provides configuration settings if necessary.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        let _ = config; // to appease clippy
        Ok(())
    }
    /// Initialize the component with a specified area and default `Mode` (REQUIRED).
    ///
    /// # Arguments
    ///
    /// * `area` - Rectangular area to initialize the component within.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    async fn init(&mut self, area: Size, default_mode: Mode) -> Result<()>;
    /// Handle incoming events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `event` - An optional event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    async fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Signal>> {
        let action = match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event).await?,
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event)?,
            _ => None,
        };
        Ok(action)
    }
    /// Handle key events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `key` - A key event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<Option<Signal>> {
        let _ = key; // to appease clippy
        Ok(None)
    }
    /// Handle mouse events and produce actions if necessary.
    ///
    /// # Arguments
    ///
    /// * `mouse` - A mouse event to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<Option<Signal>> {
        let _ = mouse; // to appease clippy
        Ok(None)
    }
    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    async fn update(&mut self, action: Signal) -> Result<Option<Signal>> {
        let _ = action; // to appease clippy
        Ok(None)
    }
    /// Render the component on the screen. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `f` - A frame used for rendering.
    /// * `area` - The area in which the component should be drawn.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}

/// returns a frame block given the active mode, and the caller mode. If both
/// modes are the same, the frame will be colored green, otherwise gray.
pub fn frame_block(active: bool, caller_mode: Mode) -> Block<'static> {
    let block = Block::new().borders(Borders::all());

    let style = if active {
        Style::new().fg(Color::Green)
    } else {
        Style::new().fg(Color::Gray)
    };

    let block = block.title(format!("[{}]", Into::<u8>::into(caller_mode)));
    let block = block.title(format!("{caller_mode:?}"));

    block.border_style(style)
}
