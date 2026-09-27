//! Navigation-stack mutation.
//!
//! This module contains operations that modify a targeted [`NavStackState`]:
//! pushing, replacing, popping, restoring forward history, clearing, resetting,
//! and the conventional `back`, `home`, and `go_forward` aliases.

use gpui_kit::{
    AnyView, App,
    base::{NavMotion, NavStack},
};

use crate::{NavigationResult, registry::NavigationRegistry};

use super::Navigator;

impl Navigator {
    /// Returns the navigation stack for this navigator's scope.
    ///
    /// If the scope does not yet have a stack, one is created. Accessing the
    /// stack also makes its scope the application's current scope.
    ///
    /// Errors are reported through [`tracing::error!`] and `None` is returned.
    /// Use [`Self::try_stack`] when the exact error is needed.
    pub fn stack(&self, cx: &mut App) -> Option<NavStack> {
        self.report("stack", self.try_stack(cx))
    }

    /// Returns the navigation stack for this navigator's scope.
    ///
    /// The scope is created when it does not already exist and becomes the
    /// application's current scope.
    ///
    /// # Errors
    ///
    /// Returns [`crate::NavigatorError::NotInstalled`] if the global registry
    /// has not been installed.
    pub fn try_stack(&self, cx: &mut App) -> NavigationResult<NavStack> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;
        Ok(NavStack::new(&state))
    }

    /// Pushes `view` onto the targeted navigation stack.
    ///
    /// If the target scope does not yet exist, it is created.
    ///
    /// This is the non-panicking convenience form of [`Self::try_push`].
    /// Navigation errors are reported through [`tracing::error!`].
    pub fn push<V>(&self, view: V, cx: &mut App)
    where
        V: Into<AnyView>,
    {
        self.report_unit("push", self.try_push(view, cx));
    }

    /// Pushes `view` onto the targeted navigation stack.
    ///
    /// If the target scope does not yet exist, it is created and activated.
    /// The navigator's configured transition is used.
    pub fn try_push<V>(&self, view: V, cx: &mut App) -> NavigationResult<()>
    where
        V: Into<AnyView>,
    {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;
        let motion = self.try_resolve_motion(cx)?;

        state.update(cx, |state, cx| {
            state.push(view, motion, cx);
        });

        Ok(())
    }

    /// Pushes `view` onto the targeted navigation stack using `motion`.
    ///
    /// The explicitly supplied transition takes precedence over the navigator's
    /// configured transition and the application's default transition.
    pub fn push_with_motion<V>(&self, view: V, motion: NavMotion, cx: &mut App)
    where
        V: Into<AnyView>,
    {
        self.report_unit(
            "push_with_motion",
            self.try_push_with_motion(view, motion, cx),
        );
    }

    /// Pushes `view` onto the targeted navigation stack using `motion`.
    ///
    /// The target scope is created when it does not already exist.
    pub fn try_push_with_motion<V>(
        &self,
        view: V,
        motion: NavMotion,
        cx: &mut App,
    ) -> NavigationResult<()>
    where
        V: Into<AnyView>,
    {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;

        state.update(cx, |state, cx| {
            state.push(view, motion, cx);
        });

        Ok(())
    }

    /// Replaces the current view on the targeted stack using this navigator's
    /// configured transition.
    ///
    /// The replaced view is discarded. Use [`Self::try_replace`] when the
    /// previous view needs to be retained.
    pub fn replace<V>(&self, view: V, cx: &mut App)
    where
        V: Into<AnyView>,
    {
        self.report_unit("replace", self.try_replace(view, cx).map(drop));
    }

    /// Replaces the current view on the targeted stack.
    ///
    /// Returns the previous view when one existed. Forward history is preserved
    /// by the underlying navigation stack.
    pub fn try_replace<V>(&self, view: V, cx: &mut App) -> NavigationResult<Option<AnyView>>
    where
        V: Into<AnyView>,
    {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;
        let motion = self.try_resolve_motion(cx)?;

        Ok(state.update(cx, |state, cx| state.replace(view, motion, cx)))
    }

    /// Replaces the current view using `motion`.
    ///
    /// The previous view is discarded.
    pub fn replace_with_motion<V>(&self, view: V, motion: NavMotion, cx: &mut App)
    where
        V: Into<AnyView>,
    {
        self.report_unit(
            "replace_with_motion",
            self.try_replace_with_motion(view, motion, cx).map(drop),
        );
    }

    /// Replaces the current view using `motion` and returns the previous view
    /// when one existed.
    pub fn try_replace_with_motion<V>(
        &self,
        view: V,
        motion: NavMotion,
        cx: &mut App,
    ) -> NavigationResult<Option<AnyView>>
    where
        V: Into<AnyView>,
    {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;

        Ok(state.update(cx, |state, cx| state.replace(view, motion, cx)))
    }

    /// Pops the current view from the targeted stack.
    ///
    /// The popped view is retained in forward history and discarded by this
    /// convenience method. Use [`Self::try_pop`] when the view itself is
    /// needed.
    pub fn pop(&self, cx: &mut App) {
        self.report_unit("pop", self.try_pop(cx).map(drop));
    }

    /// Pops the current view from the targeted stack.
    ///
    /// The popped view is retained in forward history and returned to the
    /// caller.
    pub fn try_pop(&self, cx: &mut App) -> NavigationResult<Option<AnyView>> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;
        let motion = self.try_resolve_motion(cx)?;

        Ok(state.update(cx, |state, cx| state.pop(motion, cx)))
    }

    /// Pops the current view using `motion`.
    ///
    /// The popped view is retained in forward history.
    pub fn pop_with_motion(&self, motion: NavMotion, cx: &mut App) {
        self.report_unit(
            "pop_with_motion",
            self.try_pop_with_motion(motion, cx).map(drop),
        );
    }

    /// Pops the current view using `motion` and returns the popped view when
    /// one existed.
    pub fn try_pop_with_motion(
        &self,
        motion: NavMotion,
        cx: &mut App,
    ) -> NavigationResult<Option<AnyView>> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;

        Ok(state.update(cx, |state, cx| state.pop(motion, cx)))
    }

    /// Pops every entry above the root entry.
    ///
    /// The removed entries are retained in forward history and discarded by
    /// this convenience method.
    pub fn pop_to_root(&self, cx: &mut App) {
        self.report_unit("pop_to_root", self.try_pop_to_root(cx).map(drop));
    }

    /// Pops every entry above the root entry and returns the removed views.
    ///
    /// The removed entries remain available through [`Self::forward`] until
    /// forward history is otherwise changed.
    pub fn try_pop_to_root(&self, cx: &mut App) -> NavigationResult<Vec<AnyView>> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;
        let motion = self.try_resolve_motion(cx)?;

        Ok(state.update(cx, |state, cx| state.pop_to_root(motion, cx)))
    }

    /// Pops every entry above the root entry using `motion`.
    pub fn pop_to_root_with_motion(&self, motion: NavMotion, cx: &mut App) {
        self.report_unit(
            "pop_to_root_with_motion",
            self.try_pop_to_root_with_motion(motion, cx).map(drop),
        );
    }

    /// Pops every entry above the root entry using `motion` and returns the
    /// removed views.
    pub fn try_pop_to_root_with_motion(
        &self,
        motion: NavMotion,
        cx: &mut App,
    ) -> NavigationResult<Vec<AnyView>> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;

        Ok(state.update(cx, |state, cx| state.pop_to_root(motion, cx)))
    }

    /// Restores the most recently popped view from forward history.
    ///
    /// The restored view is discarded by this convenience method. Use
    /// [`Self::try_forward`] when the restored view is needed.
    pub fn forward(&self, cx: &mut App) {
        self.report_unit("forward", self.try_forward(cx).map(drop));
    }

    /// Restores the most recently popped view from forward history.
    ///
    /// Returns the restored view when one exists.
    pub fn try_forward(&self, cx: &mut App) -> NavigationResult<Option<AnyView>> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;
        let motion = self.try_resolve_motion(cx)?;

        Ok(state.update(cx, |state, cx| state.forward(motion, cx)))
    }

    /// Restores the most recently popped view using `motion`.
    pub fn forward_with_motion(&self, motion: NavMotion, cx: &mut App) {
        self.report_unit(
            "forward_with_motion",
            self.try_forward_with_motion(motion, cx).map(drop),
        );
    }

    /// Restores the most recently popped view using `motion` and returns it
    /// when one exists.
    pub fn try_forward_with_motion(
        &self,
        motion: NavMotion,
        cx: &mut App,
    ) -> NavigationResult<Option<AnyView>> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;

        Ok(state.update(cx, |state, cx| state.forward(motion, cx)))
    }

    /// Clears the targeted navigation stack and discards its forward history.
    ///
    /// Unlike stack-creating operations such as [`Self::push`], this operation
    /// requires the target scope to already exist.
    pub fn clear(&self, cx: &mut App) {
        self.report_unit("clear", self.try_clear(cx));
    }

    /// Clears the targeted navigation stack and discards its forward history.
    ///
    /// The target scope must already have a registered navigation stack.
    ///
    /// # Errors
    ///
    /// Returns [`crate::NavigatorError::ScopeNotFound`] if the target scope has
    /// no registered stack.
    pub fn try_clear(&self, cx: &mut App) -> NavigationResult<()> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::resolve_state(&path, cx)?;

        NavigationRegistry::activate(&path, cx)?;

        state.update(cx, |state, cx| {
            state.clear(cx);
        });

        Ok(())
    }

    /// Clears the targeted stack and establishes `view` as its new root entry.
    ///
    /// The new root is inserted using [`NavMotion::Immediate`], regardless of
    /// this navigator's configured transition.
    pub fn reset<V>(&self, view: V, cx: &mut App)
    where
        V: Into<AnyView>,
    {
        self.report_unit("reset", self.try_reset(view, cx));
    }

    /// Clears the targeted stack and establishes `view` as its new root entry.
    ///
    /// The target scope is created if necessary. The new root is inserted with
    /// [`NavMotion::Immediate`].
    pub fn try_reset<V>(&self, view: V, cx: &mut App) -> NavigationResult<()>
    where
        V: Into<AnyView>,
    {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::ensure_state(&path, cx)?;

        state.update(cx, |state, cx| {
            state.clear(cx);
            state.push(view, NavMotion::Immediate, cx);
        });

        Ok(())
    }

    /// Alias for [`Self::pop`].
    pub fn back(&self, cx: &mut App) {
        self.pop(cx);
    }

    /// Fallible alias for [`Self::try_pop`].
    pub fn try_back(&self, cx: &mut App) -> NavigationResult<Option<AnyView>> {
        self.try_pop(cx)
    }

    /// Alias for [`Self::pop_to_root`].
    pub fn home(&self, cx: &mut App) {
        self.pop_to_root(cx);
    }

    /// Fallible alias for [`Self::try_pop_to_root`].
    pub fn try_home(&self, cx: &mut App) -> NavigationResult<Vec<AnyView>> {
        self.try_pop_to_root(cx)
    }

    /// Alias for [`Self::forward`].
    pub fn go_forward(&self, cx: &mut App) {
        self.forward(cx);
    }

    /// Fallible alias for [`Self::try_forward`].
    pub fn try_go_forward(&self, cx: &mut App) -> NavigationResult<Option<AnyView>> {
        self.try_forward(cx)
    }
}
