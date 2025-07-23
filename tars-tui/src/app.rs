use std::{
    fs::{self, File, create_dir_all},
    io::Read,
    path::PathBuf,
    process::Command,
    rc::Rc,
    sync::Arc,
    thread::{self, spawn},
    time::Duration,
};

use color_eyre::Result;
use common::{Diff, TarsClient};
use crossterm::event::KeyEvent;
use futures::{StreamExt, future::Join};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
};
use reqwest_eventsource::{Event as EsEvent, EventSource};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, UnboundedSender},
        oneshot,
    },
    task::JoinHandle,
};
use tracing::{debug, error, info};

use crate::{
    action::Action,
    components::{Component, explorer::Explorer, inspector::Inspector, todo_list::TodoList},
    config::Config,
    tree::{TarsTree, TarsTreeHandle},
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    mode: Mode,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    client: TarsClient,

    // state to keep track if we need to send keystrokes un-modified
    raw_text: bool,

    tree: TarsTreeHandle,

    diff_handle: JoinHandle<()>,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Explorer,
    TodoList,
    Inspector,
}

impl From<Mode> for u8 {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Explorer => 1,
            Mode::TodoList => 2,
            Mode::Inspector => 3,
        }
    }
}

impl App {
    pub async fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let client = TarsClient::default().await.unwrap();

        let tree = Arc::new(RwLock::new(TarsTree::generate(&client).await?));

        let app = Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(Explorer::new(&client, tree.clone()).await?),
                Box::new(TodoList::new(&client).await?),
                Box::new(Inspector::new(&client).await?),
            ],
            tree,
            should_quit: false,
            should_suspend: false,
            config: Config::new()?,
            mode: Mode::Explorer,
            last_tick_key_events: Vec::new(),
            diff_handle: Self::spawn_diff_handler(&client, action_tx.clone()),
            action_tx,
            action_rx,
            raw_text: false,
            client,
        };

        Ok(app)
    }

    pub fn spawn_diff_handler(
        client: &TarsClient,
        action_tx: UnboundedSender<Action>,
    ) -> JoinHandle<()> {
        let url = client.base_path.clone();
        let url = url.join("/subscribe").unwrap();

        tokio::spawn(async move {
            let mut es = EventSource::get(url);

            while let Some(event) = es.next().await {
                match event {
                    Ok(EsEvent::Open) => info!("diff connection opened!"),
                    Ok(EsEvent::Message(message)) => {
                        let data: Diff = serde_json::from_str(message.data.as_str())
                            .expect("message should be parseable as a Diff");
                        debug!("message received: {data:?}");

                        action_tx
                            .send(Action::Diff(data))
                            .expect("sending action should not fail");
                    }
                    Err(e) => error!("error!: {e:#?}"),
                }
            }
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?, self.mode).await?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui).await?;
            if self.should_suspend {
                tui.suspend()?;

                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit => action_tx.send(Action::Quit)?,
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,

            _ => {}
        }
        for component in self.components.iter_mut() {
            if let Some(action) = component.handle_events(Some(event.clone())).await? {
                action_tx.send(action)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.mode) else {
            return Ok(());
        };

        match keymap.get(&vec![key]) {
            Some(action) => {
                if !self.raw_text {
                    action_tx.send(action.clone())?;
                }
            }
            _ => {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    if !self.raw_text {
                        action_tx.send(action.clone())?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }

                Action::Quit => self.should_quit = true,

                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::SwitchTo(mode) => self.mode = mode,
                Action::RawText => self.raw_text = true,
                Action::Refresh => {
                    self.raw_text = false;
                }

                Action::Diff(ref diff) => {
                    self.tree.write().await.apply_diff(diff.clone()).await?;
                    self.action_tx.send(Action::Refresh)?
                }
                Action::EditDescription(ref task) => {
                    tui.exit()?;

                    let mut task = task.clone();
                    let tmp_file_path = PathBuf::from(format!("/tmp/tars/{}.md", *task.name));

                    if let Some(parent) = tmp_file_path.parent() {
                        create_dir_all(parent)?;
                    }

                    fs::write(&tmp_file_path, task.description)?;

                    let tmp_file_path_hx = tmp_file_path.clone();
                    let tmp_file_path_glow = tmp_file_path.clone();

                    let hx = spawn(move || -> Result<()> {
                        Command::new("hx")
                            .arg(tmp_file_path_hx.to_str().unwrap())
                            .stdin(std::process::Stdio::inherit())
                            .stdout(std::process::Stdio::inherit())
                            .stderr(std::process::Stdio::inherit())
                            .status()?;

                        Ok(())
                    });

                    let (tx, rx) = oneshot::channel::<Option<()>>();

                    let glow = spawn(move || -> Result<()> {
                        Command::new("zellij")
                            .args([
                                "run",
                                "--direction",
                                "right",
                                "--",
                                "/bin/zsh",
                                "-l",
                                "-c",
                                &format!(
                                    "source ~/.zshrc && glow -t {}",
                                    tmp_file_path_glow.to_string_lossy()
                                ),
                            ])
                            .spawn()?;
                        thread::sleep(Duration::from_millis(100));

                        Command::new("zellij")
                            .args(["action", "move-focus", "left"])
                            .spawn()?;

                        // now we just wait to kill
                        if rx.blocking_recv()?.is_some() {
                            Command::new("zellij")
                                .args(["action", "focus-next-pane"])
                                .spawn()?;

                            thread::sleep(Duration::from_millis(20));

                            Command::new("zellij")
                                .args(["action", "close-pane"])
                                .spawn()?;
                        }
                        Ok(())
                    });

                    // Join on hx first
                    hx.join().unwrap()?;
                    // tell glow to kill itself
                    tx.send(Some(())).unwrap();
                    drop(glow);

                    let mut f = File::open(&tmp_file_path)?;

                    let mut updated_desc = String::new();
                    f.read_to_string(&mut updated_desc)?;

                    fs::remove_file(tmp_file_path)?;

                    task.description = updated_desc;

                    task.sync(&self.client).await?;

                    self.should_suspend = false;
                    tui.terminal.clear()?;
                    tui.enter()?;

                    self.action_tx.send(Action::Refresh)?;
                }
                _ => {}
            }
            for component in self.components.iter_mut() {
                if let Some(action) = component.update(action.clone()).await? {
                    self.action_tx.send(action)?
                };
            }
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        let virt_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        let two_right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)]);

        tui.draw(|frame| {
            let virt_split = virt_split.split(frame.area());

            let two_right = two_right.split(virt_split[1]);

            let layout = [Rc::new([virt_split[0]]), two_right].concat();

            for (component, rect) in self.components.iter_mut().zip(layout.iter()) {
                if let Err(err) = component.draw(frame, *rect) {
                    let _ = self
                        .action_tx
                        .send(Action::Error(format!("Failed to draw: {err:?}")));
                }
            }
        })?;
        Ok(())
    }
}
