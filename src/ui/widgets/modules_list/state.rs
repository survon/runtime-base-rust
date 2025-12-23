// src/widgets/modules_list/state.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulesListState {
    pub selected_index: usize,
    pub use_template: bool,
}

impl Default for ModulesListState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            use_template: true,
        }
    }
}

// ----------------------------------------------------------------------------
// INTENT - Messages that express what user wants to do
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ModulesListIntent {
    SelectNext,
    SelectPrevious,
    SelectModule { index: usize },
    ToggleTemplateMode,
}

// ----------------------------------------------------------------------------
// EVENTS - Things that happened (past tense)
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ModulesListEvent {
    StateChanged(ModulesListState),
    ModuleSelected { index: usize },
    TemplateModeChanged { enabled: bool },
}

// ----------------------------------------------------------------------------
// STATE MACHINE - Pure function: (State, Intent) -> (State, Vec<Event>)
// ----------------------------------------------------------------------------

pub struct ModulesListStateMachine;

impl ModulesListStateMachine {
    pub fn transition(
        state: ModulesListState,
        intent: ModulesListIntent,
        module_count: usize,
    ) -> (ModulesListState, Vec<ModulesListEvent>) {
        use ModulesListIntent::*;

        match intent {
            SelectNext => Self::handle_select_next(state, module_count),
            SelectPrevious => Self::handle_select_previous(state, module_count),
            SelectModule { index } => Self::handle_select_module(state, index, module_count),
            ToggleTemplateMode => Self::handle_toggle_template(state),
        }
    }

    fn handle_select_next(mut state: ModulesListState, module_count: usize) -> (ModulesListState, Vec<ModulesListEvent>) {
        if module_count > 0 {
            state.selected_index = (state.selected_index + 1) % module_count;
            (state.clone(), vec![
                ModulesListEvent::ModuleSelected { index: state.selected_index },
                ModulesListEvent::StateChanged(state),
            ])
        } else {
            (state, vec![])
        }
    }

    fn handle_select_previous(mut state: ModulesListState, module_count: usize) -> (ModulesListState, Vec<ModulesListEvent>) {
        if module_count > 0 {
            state.selected_index = if state.selected_index == 0 {
                module_count - 1
            } else {
                state.selected_index - 1
            };
            (state.clone(), vec![
                ModulesListEvent::ModuleSelected { index: state.selected_index },
                ModulesListEvent::StateChanged(state),
            ])
        } else {
            (state, vec![])
        }
    }

    fn handle_select_module(mut state: ModulesListState, index: usize, module_count: usize) -> (ModulesListState, Vec<ModulesListEvent>) {
        if index < module_count {
            state.selected_index = index;
            (state.clone(), vec![
                ModulesListEvent::ModuleSelected { index },
                ModulesListEvent::StateChanged(state),
            ])
        } else {
            (state, vec![])
        }
    }

    fn handle_toggle_template(mut state: ModulesListState) -> (ModulesListState, Vec<ModulesListEvent>) {
        state.use_template = !state.use_template;
        (state.clone(), vec![
            ModulesListEvent::TemplateModeChanged { enabled: state.use_template },
            ModulesListEvent::StateChanged(state),
        ])
    }
}
