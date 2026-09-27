//! Global navigation-registry lifecycle.
//!
//! This module contains operations that create, inspect, or remove the
//! application-wide navigation registry, together with operations that inspect
//! or modify the registry's active scope.

use gpui_kit::App;

use crate::{
    NavigationResult, NavigatorConfig, NavigatorError, ScopePath, registry::NavigationRegistry,
};

use super::{Navigator, report_global_error};

impl Navigator {
    /// Installs the application's global navigation registry.
    ///
    /// This is the non-panicking convenience form of [`Self::try_install`].
    /// If installation fails, the error is reported through
    /// [`tracing::error!`].
    ///
    /// Call this during application initialization before performing
    /// navigation operations or rendering a navigation stack.
    ///
    /// Use [`Self::try_install`] when installation failure must be handled by
    /// the caller.
    pub fn install(cx: &mut App, config: NavigatorConfig) {
        if let Err(error) = Self::try_install(cx, config) {
            report_global_error("install", &error);
        }
    }

    /// Installs the application's global navigation registry.
    ///
    /// The registry is configured with `config` after installation. A root
    /// navigation stack is created as part of installation.
    ///
    /// This operation is intended to be performed once during application
    /// initialization.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::AlreadyInstalled`] if a navigation registry
    /// has already been installed in `cx`.
    pub fn try_install(cx: &mut App, config: NavigatorConfig) -> Result<(), NavigatorError> {
        NavigationRegistry::install(cx)?;
        cx.global_mut::<NavigationRegistry>().with_config(config);
        Ok(())
    }

    /// Returns whether the application's global navigation registry is
    /// installed.
    #[must_use]
    pub fn is_installed(cx: &App) -> bool {
        cx.try_global::<NavigationRegistry>().is_some()
    }

    /// Removes the application's global navigation registry.
    ///
    /// This is the non-panicking convenience form of [`Self::try_uninstall`].
    /// If no registry is installed, the error is reported through
    /// [`tracing::error!`].
    ///
    /// Uninstalling removes all registered navigation scopes and their stack
    /// state.
    pub fn uninstall(cx: &mut App) {
        if let Err(error) = Self::try_uninstall(cx) {
            report_global_error("uninstall", &error);
        }
    }

    /// Removes the application's global navigation registry.
    ///
    /// This operation is primarily useful for tests and controlled application
    /// teardown. A new registry can subsequently be installed with
    /// [`Self::try_install`].
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if no navigation registry is
    /// currently installed.
    pub fn try_uninstall(cx: &mut App) -> Result<(), NavigatorError> {
        if cx.try_global::<NavigationRegistry>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }

        let _ = cx.remove_global::<NavigationRegistry>();
        Ok(())
    }

    /// Returns the application's currently active scope path.
    ///
    /// Returns `None` if the navigation registry is not installed. The error is
    /// reported through [`tracing::error!`].
    #[must_use]
    pub fn current_scope_path(cx: &App) -> Option<ScopePath> {
        match Self::try_current_scope_path(cx) {
            Ok(path) => Some(path),
            Err(error) => {
                report_global_error("current_scope_path", &error);
                None
            }
        }
    }

    /// Returns the application's currently active scope path.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global navigation
    /// registry has not been installed.
    pub fn try_current_scope_path(cx: &App) -> NavigationResult<ScopePath> {
        NavigationRegistry::current_scope(cx)
    }
}
