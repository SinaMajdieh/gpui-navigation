//! Navigation target construction and scope resolution.
//!
//! This module contains operations that construct, inspect, and resolve
//! [`Navigator`] targets. Targeting is separate from stack mutation so a
//! navigator can be configured independently of the operation eventually
//! performed on it.

use gpui_kit::App;

use crate::{NavigationResult, Scope, ScopePath, registry::NavigationRegistry};

use super::Navigator;

impl Navigator {
    /// Returns a navigator that resolves to the application's currently active
    /// scope.
    ///
    /// Resolution is lazy: calling this method does not read application state
    /// and does not capture the current scope. The current scope is looked up
    /// when the resulting navigator is resolved or used for navigation.
    ///
    /// This makes it possible to retain a navigator whose target follows the
    /// application's current scope over time.
    #[must_use]
    pub fn current_scope(&self) -> Self {
        Self {
            target: super::NavigationTarget::Current {
                suffix: ScopePath::root(),
            },
            motion: self.motion,
        }
    }

    /// Returns a navigator targeting a child scope identified by `S`.
    ///
    /// The value passed to this method is used only to supply the scope's
    /// concrete type. Its runtime value has no effect on the resulting path.
    ///
    /// Calling `scope` repeatedly builds a hierarchical scope path:
    ///
    /// ```ignore
    /// #[derive(Clone, Copy)]
    /// struct Workspace;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Settings;
    ///
    /// let navigator = Navigator::new()
    ///     .scope(Workspace)
    ///     .scope(Settings);
    /// ```
    ///
    /// Different values of the same scope type identify the same scope.
    ///
    /// If this navigator already targets a concrete path, `S` is appended to
    /// that path. If it targets [`Self::current_scope`], `S` is appended to the
    /// relative suffix and remains relative to the scope that is current when
    /// the resulting navigator is used.
    #[must_use]
    pub fn scope<S: Scope>(&self, scope: S) -> Self {
        match &self.target {
            super::NavigationTarget::Explicit(path) => Self {
                target: super::NavigationTarget::Explicit(path.child(scope)),
                motion: self.motion,
            },
            super::NavigationTarget::Current { suffix } => Self {
                target: super::NavigationTarget::Current {
                    suffix: suffix.child(scope),
                },
                motion: self.motion,
            },
        }
    }

    /// Returns a navigator targeting the root navigation scope.
    ///
    /// Any previously selected scope target is discarded. The navigator's
    /// transition configuration is preserved.
    #[must_use]
    pub fn root(&self) -> Self {
        Self {
            target: super::NavigationTarget::Explicit(ScopePath::root()),
            motion: self.motion,
        }
    }

    /// Returns a navigator targeting the parent of this navigator's scope.
    ///
    /// For a relative target created with [`Self::current_scope`], the parent
    /// is computed from the relative suffix rather than from the application's
    /// current scope immediately. The resulting navigator therefore remains
    /// relative.
    ///
    /// Returns `None` when the target is already the root scope.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        match &self.target {
            super::NavigationTarget::Explicit(path) => Some(Self {
                target: super::NavigationTarget::Explicit(path.parent()?),
                motion: self.motion,
            }),
            super::NavigationTarget::Current { suffix } => Some(Self {
                target: super::NavigationTarget::Current {
                    suffix: suffix.parent()?,
                },
                motion: self.motion,
            }),
        }
    }

    /// Returns this navigator's explicit scope path.
    ///
    /// Returns `None` when the navigator targets the current scope, because
    /// that target is resolved lazily and therefore has no concrete path until
    /// it is evaluated against application state.
    #[must_use]
    pub fn path(&self) -> Option<&ScopePath> {
        match &self.target {
            super::NavigationTarget::Explicit(path) => Some(path),
            super::NavigationTarget::Current { .. } => None,
        }
    }

    /// Resolves this navigator's effective scope path against application
    /// state.
    ///
    /// This is the non-panicking convenience form of
    /// [`Self::try_resolved_path`]. If resolution fails, the error is reported
    /// through [`tracing::error!`] and `None` is returned.
    pub fn resolved_path(&self, cx: &App) -> Option<ScopePath> {
        self.report("resolve_path", self.try_resolved_path(cx))
    }

    /// Resolves this navigator's effective scope path against application
    /// state.
    ///
    /// Explicit targets are returned unchanged. A target created with
    /// [`Self::current_scope`] is resolved by taking the registry's current
    /// scope and appending the navigator's relative suffix.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] when the navigator requires
    /// the current scope but the global navigation registry is not installed.
    pub fn try_resolved_path(&self, cx: &App) -> NavigationResult<ScopePath> {
        match &self.target {
            super::NavigationTarget::Explicit(path) => Ok(path.clone()),
            super::NavigationTarget::Current { suffix } => {
                let mut path = NavigationRegistry::current_scope(cx)?;

                for segment in suffix.segments() {
                    path = path.child_id(*segment);
                }

                Ok(path)
            }
        }
    }

    /// Creates a navigator targeting `path`.
    ///
    /// Unlike [`Self::current_scope`], the returned navigator always refers to
    /// this concrete path and does not depend on which scope is currently
    /// active.
    ///
    /// The navigator starts without an operation-specific transition, so
    /// navigation operations use the registry's configured default transition.
    #[must_use]
    pub fn for_path(path: ScopePath) -> Self {
        Self {
            target: super::NavigationTarget::Explicit(path),
            motion: None,
        }
    }

    /// Marks this navigator's resolved scope as the application's current
    /// scope.
    ///
    /// This changes only the registry's active scope; it does not modify the
    /// targeted navigation stack.
    ///
    /// Errors are reported through [`tracing::error!`].
    pub fn activate(&self, cx: &mut App) {
        self.report_unit("activate", self.try_activate(cx));
    }

    /// Marks this navigator's resolved scope as the application's current
    /// scope.
    ///
    /// This operation does not push, pop, replace, or otherwise modify the
    /// targeted stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the target cannot be resolved or does not correspond
    /// to an existing navigation scope.
    pub fn try_activate(&self, cx: &mut App) -> NavigationResult<()> {
        let path = self.try_resolved_path(cx)?;
        NavigationRegistry::activate(&path, cx)
    }

    /// Removes this navigator's scope and all of its descendant scopes.
    ///
    /// Returns `false` when the scope cannot be removed. Any error is reported
    /// through [`tracing::error!`].
    pub fn remove_scope(&self, cx: &mut App) -> bool {
        self.report("remove_scope", self.try_remove_scope(cx))
            .unwrap_or(false)
    }

    /// Removes this navigator's scope and all of its descendant scopes.
    ///
    /// The root scope cannot be removed.
    ///
    /// # Errors
    ///
    /// Returns an error if the target cannot be resolved, the registry is not
    /// installed, or the target is the root scope.
    pub fn try_remove_scope(&self, cx: &mut App) -> NavigationResult<bool> {
        let path = self.try_resolved_path(cx)?;
        NavigationRegistry::remove_scope(&path, cx)
    }
}
