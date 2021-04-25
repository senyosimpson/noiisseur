use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid OAuth state parameter")]
    InvalidOAuthState
}