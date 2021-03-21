#[derive(thiserror::Error, Debug)]
pub enum BgpError {
    #[error("IO error {0}")]
    IoError(#[from] std::io::Error),
}