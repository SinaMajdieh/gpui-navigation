//! Core navigation façade and shared implementation helpers.
//!
//! The [`Navigator`] type itself is intentionally small. Its navigation
//! behavior is split across submodules according to responsibility:
//!
//! - [`lifecycle`] handles installation and application-wide scope lifecycle;
//! - [`target`] handles scope targeting and path resolution;
//! - [`motion`] handles transition configuration;
//! - [`stack`] handles stack mutation;
//! - [`query`] handles read-only stack inspection;
//! - [`subscription`] handles navigation-event subscriptions.
//!
//! The submodules all extend [`Navigator`] through additional `impl` blocks,
//! keeping the public type and its behavior organized without changing its
//! public API.

mod lifecycle;
mod motion;
mod query;
mod stack;
mod subscription;
mod target;

use gpui_kit::{App, base::NavMotion};
use tracing::error;

use crate::{NavigationResult, NavigatorError, ScopePath, registry::NavigationRegistry};

#[derive(Clone, Debug, Eq, PartialEq)]
enum NavigationTarget {
    Explicit(ScopePath),
    Current { suffix: ScopePath },
}

/// A lightweight, non-owning handle for application navigation.
///
/// A [`Navigator`] stores only a navigation target and an optional transition;
/// it does not contain GPUI entity handles or borrow application state. It can
/// therefore be freely cloned and retained by views or other application code.
///
/// [`Navigator::new`] targets the root navigation scope.
/// [`Navigator::scope`] extends the target with a typed child scope, while
/// [`Navigator::current_scope`] creates a target that is resolved relative to
/// the application's currently active scope when a navigation operation is
/// performed.
///
/// Navigation operations are available in two forms:
///
/// - infallible convenience methods such as [`Self::push`], [`Self::pop`], and
///   [`Self::replace`] report failures through [`tracing::error!`] and
///   otherwise leave the navigation state unchanged;
/// - corresponding `try_*` methods such as [`Self::try_push`] and
///   [`Self::try_pop`] return [`NavigationResult`] so callers can handle
///   failures explicitly.
///
/// Scope activation is application-global. It is changed when a scope is used
/// for stack access or navigation, or when [`Self::activate`] is called
/// explicitly. It is not inferred from the entity performing the operation
/// and is not window-local.
#[must_use = "a Navigator is a lightweight handle; call a navigation method on it or retain the scoped value"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Navigator {
    target: NavigationTarget,
    motion: Option<NavMotion>,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            target: NavigationTarget::Explicit(ScopePath::root()),
            motion: None,
        }
    }
}

impl Navigator {
    /// Creates a navigator targeting the application's root navigation scope.
    ///
    /// The returned navigator uses the application's configured default
    /// transition unless a transition is subsequently selected with
    /// [`Self::with_motion`], [`Self::immediate`], or [`Self::animated`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Executes a read-only operation against the targeted navigation stack.
    ///
    /// This helper is shared by the query methods. It resolves the path,
    /// obtains the existing stack state, and passes a shared reference to
    /// `read`.
    pub(super) fn read_path<R>(
        &self,
        cx: &App,
        path: &ScopePath,
        read: impl FnOnce(&gpui_kit::base::NavStackState) -> R,
    ) -> NavigationResult<R> {
        let state = NavigationRegistry::resolve_state(path, cx)?;
        Ok(read(state.read(cx)))
    }

    /// Converts a navigation result into the convenience API's return value,
    /// reporting failures through [`tracing::error!`].
    pub(super) fn report<T>(
        &self,
        operation: &'static str,
        result: NavigationResult<T>,
    ) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.report_error(operation, &error);
                None
            }
        }
    }

    /// Reports a failed operation whose result is intentionally discarded.
    pub(super) fn report_unit<T>(&self, operation: &'static str, result: NavigationResult<T>) {
        if let Err(error) = result {
            self.report_error(operation, &error);
        }
    }

    /// Logs a navigation failure together with the operation and target.
    pub(super) fn report_error(&self, operation: &'static str, error: &NavigatorError) {
        error!(
            operation,
            navigator = ?self.target,
            error = %error,
            "navigation operation failed"
        );
    }
}

/// Reports an error from an operation that does not have a navigator instance,
/// such as installation or removal of the global registry.
pub(super) fn report_global_error(operation: &'static str, error: &NavigatorError) {
    error!(
        operation,
        error = %error,
        "global navigation operation failed"
    );
}
