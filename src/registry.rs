use std::collections::HashMap;

use gpui_kit::{
    App, AppContext, Entity, Global,
    base::{NavMotion, NavStackState},
};

use crate::{NavigationResult, NavigatorConfig, NavigatorError, ScopePath};

#[derive(Debug, Default)]
pub(crate) struct NavigationRegistry {
    config: NavigatorConfig,
    stacks: HashMap<ScopePath, Entity<NavStackState>>,
}

impl Global for NavigationRegistry {}

impl NavigationRegistry {
    pub fn install(cx: &mut App) -> NavigationResult<()> {
        if cx.try_global::<Self>().is_some() {
            return Err(NavigatorError::AlreadyInstalled);
        }

        let root = cx.new(|_| NavStackState::new());
        let mut stacks = HashMap::with_capacity(1);
        stacks.insert(ScopePath::root(), root);

        let config = NavigatorConfig::default();

        cx.set_global(Self { config, stacks });
        Ok(())
    }

    pub fn with_config(&mut self, config: NavigatorConfig) -> &Self {
        self.config = config;
        self
    }

    pub fn default_motion(&self) -> NavMotion {
        self.config.default_motion
    }

    pub fn set_default_motion(motion: NavMotion, cx: &mut App) -> NavigationResult<()> {
        cx.try_global::<Self>()
            .ok_or(NavigatorError::NotInstalled)?;
        cx.global_mut::<Self>().config.default_motion = motion;
        Ok(())
    }

    pub fn ensure_state(path: &ScopePath, cx: &mut App) -> NavigationResult<Entity<NavStackState>> {
        if let Some(state) = cx
            .try_global::<Self>()
            .and_then(|registry| registry.stacks.get(path))
        {
            return Ok(state.clone());
        }

        if cx.try_global::<Self>().is_none() {
            return Err(NavigatorError::NotInstalled);
        }

        let state = cx.new(|_| NavStackState::new());
        cx.global_mut::<Self>()
            .stacks
            .insert(path.clone(), state.clone());

        Ok(state)
    }

    pub fn resolve_state(path: &ScopePath, cx: &App) -> NavigationResult<Entity<NavStackState>> {
        cx.try_global::<Self>()
            .and_then(|registry| registry.stacks.get(path))
            .cloned()
            .ok_or_else(|| NavigatorError::ScopeNotFound(path.clone()))
    }

    pub fn remove_scope(path: &ScopePath, cx: &mut App) -> bool {
        let registry = cx.global_mut::<Self>();
        let before = registry.stacks.len();
        registry
            .stacks
            .retain(|candidate, _| !path.is_prefix_of(candidate));
        before != registry.stacks.len()
    }
}
