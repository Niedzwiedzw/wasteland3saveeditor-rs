use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SaveEditorError {
    #[error("Failed to parse input file")]
    BadFormat,
}
