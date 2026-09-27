use thiserror::Error;

use crate::ScopePath;

/// Result type returned by navigation operations.
pub type NavigationResult<T> = Result<T, NavigatorError>;

/// Errors returned by the navigation façade.
///
/// These errors describe failures involving the lifecycle of the global
/// navigation registry or resolution of navigation scopes.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NavigatorError {
    /// The global navigation registry has not been installed.
    ///
    /// The registry must be installed during application initialization before
    /// navigation operations can be performed.
    #[error(
        "global navigation has not been installed; call Navigator::install during application initialization"
    )]
    NotInstalled,

    /// The global navigation registry is already installed.
    ///
    /// Only one application-wide navigation registry can be installed.
    #[error("global navigation is already installed")]
    AlreadyInstalled,

    /// A requested navigation scope does not exist.
    ///
    /// The contained [`ScopePath`] identifies the scope that could not be
    /// resolved.
    #[error("navigation scope `{0}` does not exist")]
    ScopeNotFound(ScopePath),

    /// The root navigation scope cannot be removed.
    ///
    /// The root scope is created when the navigation registry is installed and
    /// remains available for the lifetime of the registry.
    #[error("the root navigation scope cannot be removed")]
    CannotRemoveRoot,

    /// A relative target could not be resolved from the current navigation scope.
    ///
    /// This error indicates that an operation requiring the current scope could
    /// not determine a valid scope path from it.
    #[error("the current navigation scope cannot resolve the requested relative path")]
    InvalidCurrentScope,
}
