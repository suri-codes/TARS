use common::types::{Group, Task};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Exit,
    Enter,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    SwitchTo(Mode),
    Select(Selection),
    ScopeUpdate(Option<Group>),
    //TODO: make it so that whenever something is changed / added, we send this refresh action. All components
    // will then refresh their state from the daemon, otherwise they dont communicate with it.
    // this will also clear raw text
    Refresh,
    //Note: raw_text
    RawText,
    // the string is the file name to launch with
    EditDescription(Task),
}

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct EditDescriptionArgs {
//     pub tmp_file_path: PathBuf,
//     pub task: Task,
// }

// impl EditDescriptionArgs {
//     pub fn new(tmp_file_path: PathBuf, task: Task) -> Self {
//         Self {
//             tmp_file_path,
//             task,
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
// struct EditArgs {
//     callback: Box<dyn Fn() -> Result<Option<Action>>>,
// }

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Selection {
    Task(Task),
    Group(Group),
}
