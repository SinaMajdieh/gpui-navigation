//! Navigation transition configuration.
//!
//! This module controls the transition selected by a [`Navigator`] and the
//! application's registry-wide default transition.

use gpui_kit::{App, base::NavMotion};

use crate::{NavigationResult, NavigatorError, registry::NavigationRegistry};

use super::Navigator;

impl Navigator {
    /// Returns a copy of this navigator using `motion` for navigation
    /// operations that do not explicitly receive a transition.
    ///
    /// The original navigator is unchanged.
    #[must_use]
    pub fn with_motion(&self, motion: NavMotion) -> Self {
        Self {
            target: self.target.clone(),
            motion: Some(motion),
        }
    }

    /// Returns a copy of this navigator configured to use immediate
    /// transitions.
    ///
    /// The original navigator is unchanged.
    #[must_use]
    pub fn immediate(&self) -> Self {
        self.with_motion(NavMotion::Immediate)
    }

    /// Returns a copy of this navigator configured to use animated
    /// transitions.
    ///
    /// The original navigator is unchanged.
    #[must_use]
    pub fn animated(&self) -> Self {
        self.with_motion(NavMotion::Animated)
    }

    /// Sets the application's default navigation transition.
    ///
    /// This changes the registry-level default used by navigators that do not
    /// have an operation-specific transition configured with
    /// [`Self::with_motion`] or one of its convenience methods.
    ///
    /// Errors are reported through [`tracing::error!`].
    pub fn set_default_motion(cx: &mut App, motion: NavMotion) {
        if let Err(error) = Self::try_set_default_motion(cx, motion) {
            super::report_global_error("set_default_motion", &error);
        }
    }

    /// Sets the application's default navigation transition.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global navigation
    /// registry has not been installed.
    pub fn try_set_default_motion(cx: &mut App, motion: NavMotion) -> NavigationResult<()> {
        NavigationRegistry::set_default_motion(motion, cx)
    }

    /// Resolves the transition used by an operation.
    ///
    /// An operation-specific transition takes precedence over the registry's
    /// configured default transition.
    pub(super) fn try_resolve_motion(&self, cx: &App) -> NavigationResult<NavMotion> {
        let registry = cx
            .try_global::<NavigationRegistry>()
            .ok_or(NavigatorError::NotInstalled)?;

        Ok(self.motion.unwrap_or(registry.default_motion()))
    }
}
