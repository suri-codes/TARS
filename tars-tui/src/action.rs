use common::{Diff, types::Task};
use id_tree::NodeId;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::app::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Signal {
    Tick,
    Render,
    Resize(u16, u16),
    Resume,
    ClearScreen,
    Error(String),
    Select(NodeId),
    ScopeUpdate(NodeId),
    Update,
    // NOTE: whenever something is changed / added, we send this refresh action. All components
    // will then refresh their state from the daemon, otherwise they dont communicate with it.
    // this will also clear raw text
    Refresh,
    RawText,
    EditDescriptionForTask(Task),
    Diff(Diff),
    // actions that the user inputs
    Action(Action),
}

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    SwitchTo(Mode),
    Suspend,
    Help,
    Quit,
    ToggleShowFinished,
    Delete,
    NewTask,
    NewGroup,
    NewSubGroup,
    MoveDown,
    MovePageDown,
    MoveUp,
    MovePageUp,
    MoveInto,
    MoveOutOf,
    EditName,
    EditColor,
    EditDue,
    EditDescription,
    RandomColor,
    EditPriority,
    ToggleFinishTask,
}
