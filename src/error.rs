/// Errors
#[derive(Debug)]
pub enum Error {
    /// Compilation failed
    CompilationFailed,

    /// Json decoding error
    Json(serde_json::Error),

    /// std::io::Error
    Io(std::io::Error),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}
