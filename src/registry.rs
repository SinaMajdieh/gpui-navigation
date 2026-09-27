use std::collections::HashMap;

use gpui_kit::{
    App, AppContext, Entity, Global,
    base::{NavMotion, NavStackState},
};

use crate::{NavigationResult, NavigatorConfig, NavigatorError, ScopePath};

/// Application-wide navigation state.
///
/// The registry owns the [`NavStackState`] associated with each navigation
/// [`ScopePath`] and tracks which scope is currently active.
///
/// A registry always contains the root scope. Scope entries are created
/// lazily by [`Self::ensure_state`] and may be removed together with their
/// descendants through [`Self::remove_scope`].
#[derive(Debug)]
pub(crate) struct NavigationRegistry {
    config: NavigatorConfig,
    stacks: HashMap<ScopePath, Entity<NavStackState>>,
    current_scope: ScopePath,
}

impl Default for NavigationRegistry {
    fn default() -> Self {
        Self {
            config: NavigatorConfig::default(),
            stacks: HashMap::new(),
            current_scope: ScopePath::root(),
        }
    }
}

impl Global for NavigationRegistry {}

impl NavigationRegistry {
    /// Installs the application-wide navigation registry.
    ///
    /// A [`NavStackState`] is created for the root scope during installation.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::AlreadyInstalled`] if a registry is already
    /// installed in `cx`.
    pub fn install(cx: &mut App) -> NavigationResult<()> {
        if cx.try_global::<Self>().is_some() {
            return Err(NavigatorError::AlreadyInstalled);
        }

        let root = cx.new(|_| NavStackState::new());
        let mut registry = Self::default();
        registry.stacks.insert(ScopePath::root(), root);

        cx.set_global(registry);
        Ok(())
    }

    /// Replaces the registry configuration.
    ///
    /// Returns a shared reference to the registry to allow configuration calls
    /// to be chained while constructing or updating the registry.
    pub fn with_config(&mut self, config: NavigatorConfig) -> &Self {
        self.config = config;
        self
    }

    /// Returns the transition configured for navigation operations that do not
    /// specify their own motion.
    pub fn default_motion(&self) -> NavMotion {
        self.config.default_motion
    }

    /// Changes the default navigation transition of the global registry.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global registry has not
    /// been installed.
    pub fn set_default_motion(motion: NavMotion, cx: &mut App) -> NavigationResult<()> {
        cx.try_global::<Self>()
            .ok_or(NavigatorError::NotInstalled)?;
        cx.global_mut::<Self>().config.default_motion = motion;
        Ok(())
    }

    /// Returns the path of the currently active navigation scope.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global registry has not
    /// been installed.
    pub fn current_scope(cx: &App) -> NavigationResult<ScopePath> {
        cx.try_global::<Self>()
            .map(|registry| registry.current_scope.clone())
            .ok_or(NavigatorError::NotInstalled)
    }

    /// Makes an existing scope the currently active navigation scope.
    ///
    /// Unlike [`Self::ensure_state`], this method does not create a missing
    /// scope.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global registry has not
    /// been installed, or [`NavigatorError::ScopeNotFound`] if `path` has no
    /// associated navigation stack.
    pub fn activate(path: &ScopePath, cx: &mut App) -> NavigationResult<()> {
        let registry = cx
            .try_global::<Self>()
            .ok_or(NavigatorError::NotInstalled)?;

        if !registry.stacks.contains_key(path) {
            return Err(NavigatorError::ScopeNotFound(path.clone()));
        }

        cx.global_mut::<Self>().current_scope = path.clone();
        Ok(())
    }

    /// Returns the navigation stack for `path`, creating it if necessary.
    ///
    /// A newly created scope is also made the current navigation scope.
    ///
    /// Existing scopes are reused rather than replaced, preserving their
    /// [`NavStackState`].
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global registry has not
    /// been installed.
    pub fn ensure_state(path: &ScopePath, cx: &mut App) -> NavigationResult<Entity<NavStackState>> {
        if cx.try_global::<Self>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }

        let state = if let Some(state) = cx
            .try_global::<Self>()
            .and_then(|registry| registry.stacks.get(path))
            .cloned()
        {
            state
        } else {
            let state = cx.new(|_| NavStackState::new());
            cx.global_mut::<Self>()
                .stacks
                .insert(path.clone(), state.clone());
            state
        };

        cx.global_mut::<Self>().current_scope = path.clone();
        Ok(state)
    }

    /// Returns the navigation stack associated with an existing scope.
    ///
    /// Unlike [`Self::ensure_state`], this method never creates a missing scope.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::ScopeNotFound`] if no stack is registered for
    /// `path`.
    pub fn resolve_state(path: &ScopePath, cx: &App) -> NavigationResult<Entity<NavStackState>> {
        cx.try_global::<Self>()
            .and_then(|registry| registry.stacks.get(path))
            .cloned()
            .ok_or_else(|| NavigatorError::ScopeNotFound(path.clone()))
    }

    /// Removes a scope and all of its descendant scopes.
    ///
    /// If the removed scope contains the current scope, the current scope is
    /// moved to the removed scope's parent when that parent still exists.
    /// Otherwise, the root scope becomes current.
    ///
    /// Removing a path that does not exist is a no-op and returns `false`.
    ///
    /// # Errors
    ///
    /// Returns [`NavigatorError::NotInstalled`] if the global registry has not
    /// been installed, or [`NavigatorError::CannotRemoveRoot`] if `path` is
    /// the root scope.
    ///
    /// # Returns
    ///
    /// Returns `true` if at least one scope was removed and `false` if `path`
    /// did not identify an existing scope.
    pub fn remove_scope(path: &ScopePath, cx: &mut App) -> NavigationResult<bool> {
        let registry = cx
            .try_global::<Self>()
            .ok_or(NavigatorError::NotInstalled)?;

        if path.is_root() {
            return Err(NavigatorError::CannotRemoveRoot);
        }

        if !registry.stacks.contains_key(path) {
            return Ok(false);
        }

        let registry = cx.global_mut::<Self>();
        let before = registry.stacks.len();

        registry
            .stacks
            .retain(|candidate, _| !path.is_prefix_of(candidate));

        if path.is_prefix_of(&registry.current_scope) {
            registry.current_scope = path
                .parent()
                .filter(|parent| registry.stacks.contains_key(parent))
                .unwrap_or_else(ScopePath::root);
        }

        Ok(before != registry.stacks.len())
    }
}
