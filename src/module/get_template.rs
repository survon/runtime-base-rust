use crate::{
    module::Module,
    ui::template::{get_template, UiTemplate},
};

impl Module {
    pub fn get_template(&mut self) -> std::result::Result<&Box<dyn UiTemplate>, String> {
        if self.cached_template.is_none() {
            let template = get_template(&self.config.template)
                .ok_or_else(|| format!("Unknown template: {}", self.config.template))?;

            for binding in template.required_bindings() {
                if !self.config.bindings.contains_key(*binding) {
                    return Err(format!(
                        "Module '{}' missing required binding '{}' for template '{}'",
                        self.config.name, binding, self.config.template
                    ));
                }
            }

            self.cached_template = Some(template);
        }

        Ok(self.cached_template.as_ref().unwrap())
    }
}
