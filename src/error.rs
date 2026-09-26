use thiserror::Error;

use crate::ScopePath;

pub type NavigationResult<T> = Result<T, NavigatorError>;

/// Errors returned by the navigation façade.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NavigatorError {
    /// The global navigation registry has not been installed.
    #[error(
        "global navigation has not been installed; call Navigator::install during application initialization"
    )]
    NotInstalled,

    /// The global navigation registry is already installed.
    #[error("global navigation is already installed")]
    AlreadyInstalled,

    /// A requested scope does not exist.
    #[error("navigation scope `{0}` does not exist")]
    ScopeNotFound(ScopePath),

    /// The root scope is protected from removal.
    #[error("the root navigation scope cannot be removed")]
    CannotRemoveRoot,
}
