use crate::module::Module;

impl Module {
    pub fn has_knowledge_dir(&self) -> bool {
        self.path.join("knowledge").exists()
    }
}
