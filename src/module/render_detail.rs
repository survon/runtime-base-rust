use ratatui::{
    buffer::Buffer,
    layout::Rect,
};

use crate::module::Module;

impl Module {
    pub fn render_detail(&mut self, area: Rect, buf: &mut Buffer) -> std::result::Result<(), String> {
        self.get_template()?;

        let mut template = self.cached_template.take()
            .ok_or_else(|| "Template not loaded".to_string())?;

        template.render_detail(area, buf, self);

        self.cached_template = Some(template);

        Ok(())
    }
}
