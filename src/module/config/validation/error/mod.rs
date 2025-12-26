mod trait_display;
mod trait_error;

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub error: String,
}
