use crossterm::event::KeyCode;

use super::{ConfigEditor, EditorAction, EditorField, FieldValue};

impl ConfigEditor {
    pub fn handle_key(&mut self, key: KeyCode) -> EditorAction {
        // Special handling for new modules in initial setup
        if self.is_new_module && self.fields.len() == 2 {
            // We're in the initial "pick module type" phase
            match key {
                KeyCode::Up => {
                    if self.selected_field > 0 {
                        self.selected_field -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Down => {
                    if self.selected_field < 1 {
                        self.selected_field += 1;
                    }
                    EditorAction::None
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if self.selected_field == 1 {
                        // User is on Module Type field - expand with that type
                        if let Some((_, _, FieldValue::Enum { options, selected })) = self.fields.get(1) {
                            let module_type = options[*selected].clone();
                            self.expand_fields_for_type(&module_type);
                            self.selected_field = 0;
                            return EditorAction::ModuleTypeSelected;
                        }
                    } else {
                        // On Name field - allow editing
                        self.start_editing();
                    }
                    EditorAction::None
                }
                KeyCode::Left | KeyCode::Right => {
                    // For module type enum, cycle through options
                    if self.selected_field == 1 {
                        if let Some((_, _, value)) = self.fields.get_mut(1) {
                            if let FieldValue::Enum { options, selected } = value {
                                if key == KeyCode::Right {
                                    *selected = (*selected + 1) % options.len();
                                } else if *selected > 0 {
                                    *selected -= 1;
                                } else {
                                    *selected = options.len() - 1;
                                }
                                return EditorAction::ValueChanged;
                            }
                        }
                    }
                    EditorAction::None
                }
                KeyCode::Esc => EditorAction::Close,
                _ => EditorAction::None,
            }
        } else if self.is_editing {
            match key {
                KeyCode::Esc => {
                    self.is_editing = false;
                    self.edit_buffer.clear();
                    EditorAction::None
                }
                KeyCode::Enter => {
                    self.apply_edit();
                    self.is_editing = false;

                    // Update module_name if we edited the name field
                    if let Some((_, EditorField::Name, value)) = self.fields.get(0) {
                        self.module_name = value.as_display_string()
                            .to_lowercase()
                            .replace(" ", "_");
                    }

                    EditorAction::ValueChanged
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                    EditorAction::None
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.edit_buffer.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.edit_buffer.len() {
                        self.edit_buffer.remove(self.cursor_pos);
                    }
                    EditorAction::None
                }
                KeyCode::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Right => {
                    if self.cursor_pos < self.edit_buffer.len() {
                        self.cursor_pos += 1;
                    }
                    EditorAction::None
                }
                _ => EditorAction::None,
            }
        } else {
            match key {
                KeyCode::Up => {
                    if self.selected_field > 0 {
                        self.selected_field -= 1;
                    }
                    EditorAction::None
                }
                KeyCode::Down => {
                    if self.selected_field < self.fields.len().saturating_sub(1) {
                        self.selected_field += 1;
                    }
                    EditorAction::None
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.start_editing();
                    EditorAction::None
                }
                KeyCode::Left | KeyCode::Right => {
                    // For enum fields, cycle through options
                    if let Some((_, _, value)) = self.fields.get_mut(self.selected_field) {
                        if let FieldValue::Enum { options, selected } = value {
                            if key == KeyCode::Right {
                                *selected = (*selected + 1) % options.len();
                            } else if *selected > 0 {
                                *selected -= 1;
                            } else {
                                *selected = options.len() - 1;
                            }
                            return EditorAction::ValueChanged;
                        } else if let FieldValue::Bool(b) = value {
                            *b = !*b;
                            return EditorAction::ValueChanged;
                        }
                    }
                    EditorAction::None
                }
                KeyCode::Esc => EditorAction::Close,
                KeyCode::Char('s') => EditorAction::Save,
                _ => EditorAction::None,
            }
        }
    }
}
