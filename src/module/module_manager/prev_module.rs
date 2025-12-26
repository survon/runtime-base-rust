use crate::module::ModuleManager;

impl ModuleManager {
    pub fn prev_module(&mut self) {
        let displayable_indices = self.get_displayable_indices();
        if displayable_indices.is_empty() {
            return;
        }

        if let Some(current_pos) = displayable_indices.iter().position(|&idx| idx == self.selected_module) {
            let new_pos = if current_pos == 0 {
                displayable_indices.len() - 1
            } else {
                current_pos - 1
            };
            self.selected_module = displayable_indices[new_pos];
        } else {
            self.selected_module = displayable_indices[0];
        }
    }
}
