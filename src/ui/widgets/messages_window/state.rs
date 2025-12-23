// src/widgets/messages_window/state.rs
use serde::{Deserialize, Serialize};
use crate::util::io::bus::BusMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesState {
    pub messages: Vec<BusMessage>,
    pub scroll_offset: usize,
    pub max_messages: usize,
}

impl Default for MessagesState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            max_messages: 100,
        }
    }
}

// ----------------------------------------------------------------------------
// INTENT - Messages that express what user wants to do
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MessagesIntent {
    ScrollUp,
    ScrollDown,
    ScrollToBottom,
    AddMessage(BusMessage),
    SetVisibleLines(usize),
}

// ----------------------------------------------------------------------------
// EVENTS - Things that happened (past tense)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MessagesEvent {
    StateChanged(MessagesState),
    MessageAdded { message: BusMessage, total: usize },
    Scrolled { offset: usize },
}

// ----------------------------------------------------------------------------
// STATE MACHINE - Pure function: (State, Intent) -> (State, Vec<Event>)
// ----------------------------------------------------------------------------

pub struct MessagesStateMachine;

impl MessagesStateMachine {
    pub fn transition(
        state: MessagesState,
        intent: MessagesIntent,
    ) -> (MessagesState, Vec<MessagesEvent>) {
        use MessagesIntent::*;

        match intent {
            ScrollUp => Self::handle_scroll_up(state),
            ScrollDown => Self::handle_scroll_down(state),
            ScrollToBottom => Self::handle_scroll_to_bottom(state),
            AddMessage(msg) => Self::handle_add_message(state, msg),
            SetVisibleLines(lines) => Self::handle_set_visible_lines(state, lines),
        }
    }

    fn handle_scroll_up(mut state: MessagesState) -> (MessagesState, Vec<MessagesEvent>) {
        if state.scroll_offset > 0 {
            state.scroll_offset -= 1;
            (state.clone(), vec![
                MessagesEvent::Scrolled { offset: state.scroll_offset },
                MessagesEvent::StateChanged(state),
            ])
        } else {
            (state, vec![])
        }
    }

    fn handle_scroll_down(mut state: MessagesState) -> (MessagesState, Vec<MessagesEvent>) {
        // Note: max scroll is calculated based on visible_lines which isn't in state
        // We'll need to pass this contextually or store it
        state.scroll_offset += 1;
        (state.clone(), vec![
            MessagesEvent::Scrolled { offset: state.scroll_offset },
            MessagesEvent::StateChanged(state),
        ])
    }

    fn handle_scroll_to_bottom(mut state: MessagesState) -> (MessagesState, Vec<MessagesEvent>) {
        // This will be clamped by the widget based on visible lines
        state.scroll_offset = state.messages.len();
        (state.clone(), vec![
            MessagesEvent::Scrolled { offset: state.scroll_offset },
            MessagesEvent::StateChanged(state),
        ])
    }

    fn handle_add_message(mut state: MessagesState, message: BusMessage) -> (MessagesState, Vec<MessagesEvent>) {
        state.messages.push(message.clone());

        // Trim old messages
        if state.messages.len() > state.max_messages {
            state.messages.remove(0);
            if state.scroll_offset > 0 {
                state.scroll_offset -= 1;
            }
        }

        let total = state.messages.len();

        (state.clone(), vec![
            MessagesEvent::MessageAdded { message, total },
            MessagesEvent::StateChanged(state),
        ])
    }

    fn handle_set_visible_lines(state: MessagesState, _lines: usize) -> (MessagesState, Vec<MessagesEvent>) {
        // This doesn't change state, just used for rendering calculations
        (state, vec![])
    }
}
