use crossterm::event::KeyCode;
use crate::log_debug;
use crate::modules::{
    Module,
    module_handler::ModuleHandler,
    wasteland_manager::{
        config_editor::EditorAction,
        handler::{WastelandManagerHandler, WastelandView}
    },
};
use crate::util::io::event::AppEvent;

impl WastelandManagerHandler {
    pub(super) fn _handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        log_debug!("handle_key: {:?}", key_code);
        match self.current_view {
            WastelandView::Main => {
                log_debug!("in main view...");
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        log_debug!("pressed Up. selected index: {}", self.selected_index);
                        None
                    }
                    KeyCode::Down => {
                        let max = 5;
                        if self.selected_index < max {
                            self.selected_index += 1;
                        }
                        log_debug!("pressed Down. selected index: {}", self.selected_index);
                        None
                    }
                    KeyCode::Enter => {
                        if self.selected_index == 5 {
                            Some(AppEvent::Back)
                        } else {
                            self.handle_main_menu_select();
                            None
                        }
                    }
                    KeyCode::Char('r') => {
                        self.refresh_data_async();
                        None
                    }
                    KeyCode::Char('s') => {
                        self.handle_scan_devices();
                        None
                    }
                    _ => None,
                }
            }
            WastelandView::PendingTrust => match key_code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down => {
                    let max = self.pending_devices.len().saturating_sub(1);
                    if self.selected_index < max {
                        self.selected_index += 1;
                    }
                    None
                }
                KeyCode::Enter => {
                    self.handle_trust_device();
                    None
                }
                KeyCode::Char('i') => {
                    self.handle_ignore_device();
                    None
                }
                KeyCode::Char('v') => {
                    self.current_view = WastelandView::AllDevices;
                    self.selected_index = 0;
                    self.refresh_known_devices();
                    None
                }
                KeyCode::Esc => {
                    self.current_view = WastelandView::Main;
                    self.selected_index = 0;
                    Some(AppEvent::NoOp)
                }
                KeyCode::Char('r') => {
                    self.refresh_data_async();
                    None
                }
                KeyCode::Char('s') => {
                    self.handle_scan_devices();
                    None
                }
                _ => None,
            },
            WastelandView::AllDevices => match key_code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down => {
                    let max = self.known_devices.len().saturating_sub(1);
                    if self.selected_index < max {
                        self.selected_index += 1;
                    }
                    None
                }
                KeyCode::Char('t') => {
                    self.handle_toggle_trust();
                    None
                }
                KeyCode::Char('d') => {
                    self.handle_delete_device();
                    None
                }
                KeyCode::Char('p') => {
                    self.current_view = WastelandView::PendingTrust;
                    self.selected_index = 0;
                    None
                }
                KeyCode::Char('r') => {
                    self.refresh_known_devices();
                    None
                }
                KeyCode::Char('s') => {
                    self.handle_scan_devices();
                    None
                }
                KeyCode::Esc => {
                    self.current_view = WastelandView::Main;
                    self.selected_index = 0;
                    Some(AppEvent::NoOp)
                }
                _ => None,
            },
            WastelandView::InstallRegistry => match key_code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down => {
                    let max = self.registry_modules.len().saturating_sub(1);
                    if self.selected_index < max {
                        self.selected_index += 1;
                    }
                    None
                }
                KeyCode::Enter => {
                    self.handle_install_module();
                    None
                }
                KeyCode::Esc => {
                    self.current_view = WastelandView::Main;
                    self.selected_index = 0;
                    Some(AppEvent::NoOp)
                }
                _ => None,
            },
            WastelandView::ManageModules => match key_code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down => {
                    let max = self.installed_modules.len().saturating_sub(1);
                    if self.selected_index < max {
                        self.selected_index += 1;
                    }
                    None
                }
                KeyCode::Enter => {
                    self.handle_manage_modules_enter();
                    None
                }
                KeyCode::Char('a') => {
                    self.handle_archive_module();
                    None
                }
                KeyCode::Esc => {
                    log_debug!("Hitting escape in modules list view");
                    self.current_view = WastelandView::Main;
                    self.selected_index = 0;
                    Some(AppEvent::NoOp)
                }
                _ => None,
            },
            WastelandView::EditConfig => {
                if let Some(editor) = &mut self.config_editor {
                    match editor.handle_key(key_code) {
                        EditorAction::None => None,
                        EditorAction::ValueChanged => None,
                        EditorAction::Save => {
                            self.handle_config_editor_save();
                            Some(AppEvent::RefreshModules)
                        }
                        EditorAction::ModuleTypeSelected => None,
                        EditorAction::Close => {
                            self.config_editor = None;
                            self.current_view = WastelandView::ManageModules;
                            None
                        }
                    }
                } else {
                    self.current_view = WastelandView::ManageModules;
                    None
                }
            }
            WastelandView::ArchivedModules => match key_code {
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                    None
                }
                KeyCode::Down => {
                    let max = self.archived_modules.len().saturating_sub(1);
                    if self.selected_index < max {
                        self.selected_index += 1;
                    }
                    None
                }
                KeyCode::Enter => {
                    self.handle_restore_module();
                    None
                }
                KeyCode::Esc => {
                    self.current_view = WastelandView::Main;
                    self.selected_index = 0;
                    Some(AppEvent::NoOp)
                }
                _ => None,
            },
            WastelandView::CreateNewModule => None,
        }
    }
}
