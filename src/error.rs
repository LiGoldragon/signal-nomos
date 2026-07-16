#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("archive encoding failed: {0}")]
    Encoding(#[from] rkyv::rancor::Error),
}
