use common::{
    Diff,
    types::{Group, Task},
};
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    SwitchTo(Mode),
    //TODO: make select only work off id instaed of passing the whole thing
    Select(Selection),
    ScopeUpdate(Option<Group>),
    Update,
    // NOTE: whenever something is changed / added, we send this refresh action. All components
    // will then refresh their state from the daemon, otherwise they dont communicate with it.
    // this will also clear raw text
    Refresh,
    RawText,
    EditDescription(Task),
    Diff(Diff),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Selection {
    Task(Task),
    Group(Group),
}
