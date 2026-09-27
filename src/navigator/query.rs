//! Read-only navigation-stack inspection.
//!
//! This module contains queries that inspect stack depth, history, current
//! views, and arbitrary read-only state without mutating the targeted stack.

use gpui_kit::{AnyView, App, Entity, base::NavStackState};

use crate::NavigationResult;

use super::Navigator;

impl Navigator {
    /// Returns the number of entries in the targeted stack.
    ///
    /// Returns `0` if the target cannot be resolved or the navigation registry
    /// is unavailable. The underlying error is reported through
    /// [`tracing::error!`].
    #[must_use]
    pub fn depth(&self, cx: &App) -> usize {
        self.report("depth", self.try_depth(cx)).unwrap_or_default()
    }

    /// Returns the number of entries in the targeted stack.
    ///
    /// # Errors
    ///
    /// Returns an error if the target cannot be resolved or its stack does not
    /// exist.
    pub fn try_depth(&self, cx: &App) -> NavigationResult<usize> {
        let path = self.try_resolved_path(cx)?;
        self.read_path(cx, &path, NavStackState::depth)
    }

    /// Returns whether the targeted stack is empty.
    ///
    /// Returns `true` if the target cannot be resolved. The underlying error is
    /// reported through [`tracing::error!`].
    #[must_use]
    pub fn is_empty(&self, cx: &App) -> bool {
        self.report("is_empty", self.try_is_empty(cx))
            .unwrap_or(true)
    }

    /// Returns whether the targeted stack is empty.
    pub fn try_is_empty(&self, cx: &App) -> NavigationResult<bool> {
        let path = self.try_resolved_path(cx)?;
        self.read_path(cx, &path, NavStackState::is_empty)
    }

    /// Returns whether the targeted stack contains an entry that can be popped.
    ///
    /// A stack at depth `1` contains only its root entry and therefore cannot
    /// be popped.
    ///
    /// Returns `false` if the target cannot be resolved. The underlying error is
    /// reported through [`tracing::error!`].
    #[must_use]
    pub fn can_pop(&self, cx: &App) -> bool {
        self.report("can_pop", self.try_can_pop(cx))
            .unwrap_or(false)
    }

    /// Returns whether the targeted stack contains an entry that can be popped.
    ///
    /// The result is `true` only when the stack depth is greater than `1`.
    pub fn try_can_pop(&self, cx: &App) -> NavigationResult<bool> {
        self.try_depth(cx).map(|depth| depth > 1)
    }

    /// Returns whether the targeted stack has an entry available in forward
    /// history.
    ///
    /// Returns `false` if the target cannot be resolved. The underlying error is
    /// reported through [`tracing::error!`].
    #[must_use]
    pub fn can_forward(&self, cx: &App) -> bool {
        self.report("can_forward", self.try_can_forward(cx))
            .unwrap_or(false)
    }

    /// Returns whether the targeted stack has an entry available in forward
    /// history.
    pub fn try_can_forward(&self, cx: &App) -> NavigationResult<bool> {
        let path = self.try_resolved_path(cx)?;

        self.read_path(cx, &path, |state| state.forward_views().next().is_some())
    }

    /// Returns whether the targeted stack is positioned at its root entry.
    ///
    /// A stack is considered to be at its root when its depth is at most `1`.
    /// If the target cannot be resolved, this convenience method returns `true`
    /// after reporting the error.
    #[must_use]
    pub fn is_at_root(&self, cx: &App) -> bool {
        self.report("is_at_root", self.try_is_at_root(cx))
            .unwrap_or(true)
    }

    /// Returns whether the targeted stack is positioned at its root entry.
    pub fn try_is_at_root(&self, cx: &App) -> NavigationResult<bool> {
        self.try_depth(cx).map(|depth| depth <= 1)
    }

    /// Returns a clone of the current view, if one exists.
    ///
    /// Returns `None` if the stack is empty or the target cannot be resolved.
    /// Errors are reported through [`tracing::error!`].
    #[must_use]
    pub fn current(&self, cx: &App) -> Option<AnyView> {
        self.report("current", self.try_current(cx)).flatten()
    }

    /// Returns a clone of the current view, if one exists.
    pub fn try_current(&self, cx: &App) -> NavigationResult<Option<AnyView>> {
        let path = self.try_resolved_path(cx)?;

        self.read_path(cx, &path, |state| state.current().cloned())
    }

    /// Returns the current view downcast to `T`, if it is an `Entity<T>`.
    ///
    /// Returns `None` when the stack is empty, the current view has a different
    /// type, or the target cannot be resolved. Errors are reported through
    /// [`tracing::error!`].
    #[must_use]
    pub fn current_as<T: 'static>(&self, cx: &App) -> Option<Entity<T>> {
        self.report("current_as", self.try_current_as::<T>(cx))
            .flatten()
    }

    /// Returns the current view downcast to `T`, if it is an `Entity<T>`.
    pub fn try_current_as<T: 'static>(&self, cx: &App) -> NavigationResult<Option<Entity<T>>> {
        self.try_current(cx)
            .map(|view| view.and_then(|view| view.downcast::<T>().ok()))
    }

    /// Returns clones of all entries in the targeted stack, ordered from the
    /// root entry to the current entry.
    ///
    /// Returns an empty vector if the target cannot be resolved. Errors are
    /// reported through [`tracing::error!`].
    #[must_use]
    pub fn views(&self, cx: &App) -> Vec<AnyView> {
        self.report("views", self.try_views(cx)).unwrap_or_default()
    }

    /// Returns clones of all entries in the targeted stack, ordered from the
    /// root entry to the current entry.
    pub fn try_views(&self, cx: &App) -> NavigationResult<Vec<AnyView>> {
        let path = self.try_resolved_path(cx)?;

        self.read_path(cx, &path, |state| state.views().cloned().collect())
    }

    /// Returns clones of entries in forward history, nearest entry first.
    ///
    /// Returns an empty vector if the target cannot be resolved. Errors are
    /// reported through [`tracing::error!`].
    #[must_use]
    pub fn forward_views(&self, cx: &App) -> Vec<AnyView> {
        self.report("forward_views", self.try_forward_views(cx))
            .unwrap_or_default()
    }

    /// Returns clones of entries in forward history, nearest entry first.
    pub fn try_forward_views(&self, cx: &App) -> NavigationResult<Vec<AnyView>> {
        let path = self.try_resolved_path(cx)?;

        self.read_path(cx, &path, |state| state.forward_views().cloned().collect())
    }

    /// Executes a read-only operation on the targeted navigation stack.
    ///
    /// Errors are reported through [`tracing::error!`], and `None` is returned
    /// when the target cannot be resolved.
    ///
    /// Use [`Self::try_read`] when the error must be handled explicitly.
    pub fn read<R>(&self, cx: &App, f: impl FnOnce(&NavStackState) -> R) -> Option<R> {
        self.report("read", self.try_read(cx, f))
    }

    /// Executes a read-only operation on the targeted navigation stack.
    ///
    /// The closure receives a shared reference to the underlying
    /// [`NavStackState`] and may return any value derived from it.
    ///
    /// # Errors
    ///
    /// Returns an error if the target cannot be resolved or its stack does not
    /// exist.
    pub fn try_read<R>(
        &self,
        cx: &App,
        f: impl FnOnce(&NavStackState) -> R,
    ) -> NavigationResult<R> {
        let path = self.try_resolved_path(cx)?;
        self.read_path(cx, &path, f)
    }
}
