// #![deny(unsafe_code, missing_docs)]

mod config;
mod error;
mod registry;
mod scope;

pub use config::NavigatorConfig;
pub use error::{NavigationResult, NavigatorError};
pub use scope::{ScopeId, ScopePath};

pub use gpui_kit::base::{
    NavMotion, NavOperation, NavPage, NavStack, NavStackEvent, NavStackState,
};
use gpui_kit::{AnyView, App, Context, Entity, Subscription};

use crate::registry::NavigationRegistry;

/// A lightweight, non-owning navigation façade.
///
/// `Navigator` contains no GPUI entity handles and therefore never needs to be
/// stored in a view or threaded through the view tree. `Navigator::new()`
/// targets the application root stack. Use [`Navigator::scope`] to target a
/// named nested stack.
///
/// Every operation resolves the current navigation registry from GPUI's global
/// state. This makes the navigator effectively stateless from the application's
/// point of view while retaining an explicit, hierarchical target for nested
/// navigation.
#[must_use = "a Navigator is a lightweight handle; call a navigation method on it or retain the scoped value"]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Navigator {
    path: ScopePath,
    motion: Option<NavMotion>,
}

impl Navigator {
    /// Creates a navigator targeting the application's root stack.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Installs the application's global navigation registry.
    ///
    /// Call this once during application initialization, before the first
    /// navigation operation or navigation stack render.
    pub fn install(cx: &mut App, config: NavigatorConfig) -> Result<(), NavigatorError> {
        NavigationRegistry::install(cx)?;
        cx.global_mut::<NavigationRegistry>().with_config(config);
        Ok(())
    }

    /// Returns whether the global navigation registry has been installed.
    #[must_use]
    pub fn is_installed(cx: &App) -> bool {
        cx.try_global::<NavigationRegistry>().is_some()
    }

    /// Removes the global navigation registry.
    ///
    /// This is primarily useful for tests or controlled application teardown.
    pub fn uninstall(cx: &mut App) -> Result<(), NavigatorError> {
        if cx.try_global::<NavigationRegistry>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }

        let _ = cx.remove_global::<NavigationRegistry>();
        Ok(())
    }

    /// Targets a child navigation scope.
    ///
    /// Scopes form a hierarchy, so this is also suitable for nested routes:
    ///
    /// ```ignore
    /// Navigator::new()
    ///     .scope("workspace")
    ///     .scope("settings")
    ///     .push(settings_view, cx)?;
    /// # Ok::<(), gpui_navigation::NavigatorError>(())
    /// ```
    #[must_use]
    pub fn scope(&self, id: impl Into<ScopeId>) -> Self {
        Self {
            path: self.path.child(id),
            motion: self.motion,
        }
    }

    /// Returns a navigator targeting the root stack.
    #[must_use]
    pub fn root(&self) -> Self {
        Self {
            path: ScopePath::root(),
            motion: self.motion,
        }
    }

    /// Returns a navigator targeting the parent scope, or `None` at root.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        Some(Self {
            path: self.path.parent()?,
            motion: self.motion,
        })
    }

    /// Returns this navigator's scope path.
    #[must_use]
    pub fn path(&self) -> &ScopePath {
        &self.path
    }

    /// Creates a navigator for an existing hierarchical scope path.
    #[must_use]
    pub fn for_path(path: ScopePath) -> Self {
        Self { path, motion: None }
    }

    /// Returns a copy of this navigator using the supplied default transition.
    #[must_use]
    pub fn with_motion(&self, motion: NavMotion) -> Self {
        Self {
            path: self.path.clone(),
            motion: Some(motion),
        }
    }

    /// Returns a copy of this navigator that uses an immediate transition.
    #[must_use]
    pub fn immediate(&self) -> Self {
        self.with_motion(NavMotion::Immediate)
    }

    /// Returns a copy of this navigator that uses animated transitions.
    #[must_use]
    pub fn animated(&self) -> Self {
        self.with_motion(NavMotion::Animated)
    }

    /// Configures the application's default transition for subsequent
    /// operations that do not specify a per-call motion.
    pub fn set_default_motion(cx: &mut App, motion: NavMotion) -> Result<(), NavigatorError> {
        NavigationRegistry::set_default_motion(motion, cx)
    }

    /// Returns the stack element for this navigator's scope.
    ///
    /// The returned [`NavStack`] is intentionally only a presentation element;
    /// navigation state remains owned by the global registry.
    pub fn stack(&self, cx: &mut App) -> Result<NavStack, NavigatorError> {
        let state = self.ensure_state(cx)?;
        Ok(NavStack::new(&state))
    }

    /// Pushes a view onto the current stack.
    pub fn push<V>(&self, view: V, cx: &mut App) -> Result<(), NavigatorError>
    where
        V: Into<AnyView>,
    {
        self.update_state(cx, |state, cx, motion| {
            state.push(view, motion, cx);
        })
    }

    /// Pushes a view with an explicit transition.
    pub fn push_with_motion<V>(
        &self,
        view: V,
        motion: NavMotion,
        cx: &mut App,
    ) -> Result<(), NavigatorError>
    where
        V: Into<AnyView>,
    {
        self.update_state(cx, |state, cx, _| {
            state.push(view, motion, cx);
        })
    }

    /// Replaces the current view, preserving forward history.
    pub fn replace<V>(&self, view: V, cx: &mut App) -> Result<Option<AnyView>, NavigatorError>
    where
        V: Into<AnyView>,
    {
        let motion = self.resolve_motion(cx)?;
        self.update_state(cx, |state, cx, _| state.replace(view, motion, cx))
    }

    /// Replaces the current view with an explicit transition.
    pub fn replace_with_motion<V>(
        &self,
        view: V,
        motion: NavMotion,
        cx: &mut App,
    ) -> Result<Option<AnyView>, NavigatorError>
    where
        V: Into<AnyView>,
    {
        self.update_state(cx, |state, cx, _| state.replace(view, motion, cx))
    }

    /// Pops the current view while preserving it for `forward`.
    pub fn pop(&self, cx: &mut App) -> Result<Option<AnyView>, NavigatorError> {
        let motion = self.resolve_motion(cx)?;
        self.update_state(cx, |state, cx, _| state.pop(motion, cx))
    }

    /// Pops the current view with an explicit transition.
    pub fn pop_with_motion(
        &self,
        motion: NavMotion,
        cx: &mut App,
    ) -> Result<Option<AnyView>, NavigatorError> {
        self.update_state(cx, |state, cx, _| state.pop(motion, cx))
    }

    /// Pops all views above the root, preserving them for forward navigation.
    pub fn pop_to_root(&self, cx: &mut App) -> Result<Vec<AnyView>, NavigatorError> {
        let motion = self.resolve_motion(cx)?;
        self.update_state(cx, |state, cx, _| state.pop_to_root(motion, cx))
    }

    /// Pops all views above the root with an explicit transition.
    pub fn pop_to_root_with_motion(
        &self,
        motion: NavMotion,
        cx: &mut App,
    ) -> Result<Vec<AnyView>, NavigatorError> {
        self.update_state(cx, |state, cx, _| state.pop_to_root(motion, cx))
    }

    /// Brings the most recently popped view back onto the stack.
    pub fn forward(&self, cx: &mut App) -> Result<Option<AnyView>, NavigatorError> {
        let motion = self.resolve_motion(cx)?;
        self.update_state(cx, |state, cx, _| state.forward(motion, cx))
    }

    /// Brings the most recently popped view back with an explicit transition.
    pub fn forward_with_motion(
        &self,
        motion: NavMotion,
        cx: &mut App,
    ) -> Result<Option<AnyView>, NavigatorError> {
        self.update_state(cx, |state, cx, _| state.forward(motion, cx))
    }

    /// Empties the current stack and discards forward history.
    pub fn clear(&self, cx: &mut App) -> Result<(), NavigatorError> {
        let state = self.resolve_state(cx)?;
        state.update(cx, |state, cx| state.clear(cx));
        Ok(())
    }

    /// Clears the stack and establishes `view` as the new root.
    pub fn reset<V>(&self, view: V, cx: &mut App) -> Result<(), NavigatorError>
    where
        V: Into<AnyView>,
    {
        let state = self.ensure_state(cx)?;
        state.update(cx, |state, cx| {
            state.clear(cx);
            state.push(view, NavMotion::Immediate, cx);
        });
        Ok(())
    }

    /// Alias for [`Navigator::pop`].
    pub fn back(&self, cx: &mut App) -> Result<Option<AnyView>, NavigatorError> {
        self.pop(cx)
    }

    /// Alias for [`Navigator::pop_to_root`].
    pub fn home(&self, cx: &mut App) -> Result<Vec<AnyView>, NavigatorError> {
        self.pop_to_root(cx)
    }

    /// Alias for [`Navigator::forward`].
    pub fn go_forward(&self, cx: &mut App) -> Result<Option<AnyView>, NavigatorError> {
        self.forward(cx)
    }

    /// Returns the number of entries in the current stack.
    pub fn depth(&self, cx: &App) -> Result<usize, NavigatorError> {
        self.read_state(cx, NavStackState::depth)
    }

    /// Returns whether the current stack has no entries.
    pub fn is_empty(&self, cx: &App) -> Result<bool, NavigatorError> {
        self.read_state(cx, NavStackState::is_empty)
    }

    /// Returns whether `pop` can remove an entry from the current stack.
    pub fn can_pop(&self, cx: &App) -> Result<bool, NavigatorError> {
        self.depth(cx).map(|depth| depth > 1)
    }

    /// Returns whether `forward` has an entry to restore.
    pub fn can_forward(&self, cx: &App) -> Result<bool, NavigatorError> {
        self.read_state(cx, |state| state.forward_views().next().is_some())
    }

    /// Returns whether this scope is currently at its root entry.
    pub fn is_at_root(&self, cx: &App) -> Result<bool, NavigatorError> {
        self.depth(cx).map(|depth| depth <= 1)
    }

    /// Returns a clone of the current view, if any.
    pub fn current(&self, cx: &App) -> Result<Option<AnyView>, NavigatorError> {
        self.read_state(cx, |state| state.current().cloned())
    }

    /// Returns the current view downcast to `T`, if it is of that type.
    pub fn current_as<T: 'static>(&self, cx: &App) -> Result<Option<Entity<T>>, NavigatorError> {
        self.current(cx)
            .map(|view| view.and_then(|view| view.downcast::<T>().ok()))
    }

    /// Returns clones of all entries in the current stack, root first.
    pub fn views(&self, cx: &App) -> Result<Vec<AnyView>, NavigatorError> {
        self.read_state(cx, |state| state.views().cloned().collect())
    }

    /// Returns the forward entries, nearest first.
    pub fn forward_views(&self, cx: &App) -> Result<Vec<AnyView>, NavigatorError> {
        self.read_state(cx, |state| state.forward_views().cloned().collect())
    }

    /// Executes a read-only operation on the underlying `NavStackState`.
    ///
    /// This is the intentionally narrow advanced escape hatch for callers that
    /// need state the façade does not expose directly. Mutating the underlying
    /// stack is deliberately not exposed so the registry remains the single
    /// navigation authority.
    pub fn read<R>(
        &self,
        cx: &App,
        f: impl FnOnce(&NavStackState) -> R,
    ) -> Result<R, NavigatorError> {
        self.read_state(cx, f)
    }

    /// Subscribes the current entity to navigation events from this scope.
    ///
    /// The returned subscription should normally be stored by the subscribing
    /// entity, following normal GPUI lifecycle semantics.
    pub fn subscribe<T: 'static>(
        &self,
        cx: &mut Context<T>,
        mut handler: impl FnMut(&mut T, &NavStackEvent, &mut Context<T>) + 'static,
    ) -> Result<Subscription, NavigatorError> {
        let state = self.ensure_state(cx)?;
        Ok(cx.subscribe(&state, move |this, _, event, cx| {
            handler(this, event, cx);
        }))
    }

    /// Releases this scope and all of its descendants.
    ///
    /// The root scope cannot be removed. Releasing a scope drops the registry's
    /// owning `Entity` references, allowing its navigation state and retained
    /// pages to be collected when no other strong references exist.
    pub fn remove_scope(&self, cx: &mut App) -> Result<bool, NavigatorError> {
        if self.path.is_root() {
            return Err(NavigatorError::CannotRemoveRoot);
        }
        if cx.try_global::<NavigationRegistry>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }

        Ok(NavigationRegistry::remove_scope(&self.path, cx))
    }

    fn resolve_motion(&self, cx: &App) -> Result<NavMotion, NavigatorError> {
        let registry = cx
            .try_global::<NavigationRegistry>()
            .ok_or(NavigatorError::NotInstalled)?;
        Ok(self.motion.unwrap_or(registry.default_motion()))
    }

    fn ensure_state(&self, cx: &mut App) -> Result<Entity<NavStackState>, NavigatorError> {
        if cx.try_global::<NavigationRegistry>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }
        NavigationRegistry::ensure_state(&self.path, cx)
    }

    fn resolve_state(&self, cx: &mut App) -> Result<Entity<NavStackState>, NavigatorError> {
        if cx.try_global::<NavigationRegistry>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }
        NavigationRegistry::resolve_state(&self.path, cx)
    }

    fn read_state<R>(
        &self,
        cx: &App,
        read: impl FnOnce(&NavStackState) -> R,
    ) -> Result<R, NavigatorError> {
        if cx.try_global::<NavigationRegistry>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }
        let state = NavigationRegistry::resolve_state(&self.path, cx)?;
        Ok(read(state.read(cx)))
    }

    fn update_state<R>(
        &self,
        cx: &mut App,
        update: impl FnOnce(&mut NavStackState, &mut Context<NavStackState>, NavMotion) -> R,
    ) -> Result<R, NavigatorError> {
        let state = self.ensure_state(cx)?;
        let motion = self
            .motion
            .unwrap_or_else(|| cx.global::<NavigationRegistry>().default_motion());
        Ok(state.update(cx, |state, cx| update(state, cx, motion)))
    }
}
