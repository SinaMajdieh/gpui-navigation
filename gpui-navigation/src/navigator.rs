use std::rc::Rc;

use gpui_kit::{App, BorrowAppContext, Entity, Global, Window};
use tracing::{debug, error, info, trace};

use crate::{NavPath, NavigationError, RouteSegment, Router, router::navigate};

/// Application-wide navigation controller.
///
/// The navigator owns navigation state and history. Router traversal is
/// handled by the router dispatch system.
pub struct Navigator {
    root: Rc<dyn Fn(&mut App, &mut Window, &NavPath) -> Result<(), NavigationError>>,

    current: Option<NavPath>,
    back_stack: Vec<NavPath>,
}

impl Global for Navigator {}

impl Navigator {
    /// Installs the application's root router.
    pub fn install<R>(root: &Entity<R>, cx: &mut App) -> Result<(), NavigationError>
    where
        R: Router,
    {
        if cx.try_global::<Self>().is_some() {
            return Err(NavigationError::AlreadyInstalled);
        }

        let root = root.clone();

        let dispatch = Rc::new(
            move |cx: &mut App,
                  window: &mut Window,
                  path: &NavPath|
                  -> Result<(), NavigationError> { navigate(&root, path, window, cx) },
        );

        cx.set_global(Self {
            root: dispatch,
            current: None,
            back_stack: Vec::new(),
        });

        info!("navigator installed");

        Ok(())
    }

    /// Navigates to an absolute path.
    pub fn go<P>(path: P, window: &mut Window, cx: &mut App)
    where
        P: Into<NavPath>,
    {
        let result = Self::try_go(path, window, cx);

        if let Err(error) = result {
            error!(%error, "navigation failed");
        }
    }

    /// Fallible form of [`Navigator::go`].
    pub fn try_go<P>(path: P, window: &mut Window, cx: &mut App) -> Result<(), NavigationError>
    where
        P: Into<NavPath>,
    {
        let path = path.into();

        info!(%path, "navigating to");

        if path.is_empty() {
            return Err(NavigationError::EmptyPath);
        }

        let (dispatch, already_current) = {
            let navigator = cx
                .try_global::<Self>()
                .ok_or(NavigationError::NotInstalled)?;

            trace!(%path, "checking current navigation state");

            (
                Rc::clone(&navigator.root),
                navigator.current.as_ref() == Some(&path),
            )
        };

        if already_current {
            debug!(%path, "path is already current");
            return Ok(());
        }

        debug!(%path, "dispatching navigation");

        dispatch(cx, window, &path)?;

        cx.update_global::<Self, _>(|navigator, _| {
            if let Some(previous) = navigator.current.replace(path.clone()) {
                trace!(%previous, "adding previous path to history");
                navigator.back_stack.push(previous);
            }
        });

        info!(%path, "navigation completed");

        Ok(())
    }

    /// Appends one route segment to the current location.
    pub fn push<R>(route: R, window: &mut Window, cx: &mut App)
    where
        R: RouteSegment,
    {
        let result = Self::try_push(route, window, cx);

        if let Err(error) = result {
            error!(%error, "navigation push failed");
        }
    }

    /// Fallible form of [`Navigator::push`].
    pub fn try_push<R>(route: R, window: &mut Window, cx: &mut App) -> Result<(), NavigationError>
    where
        R: RouteSegment,
    {
        debug!("pushing route");

        let current = Self::current(cx).ok_or(NavigationError::NoCurrentLocation)?;

        Self::try_go(current.push(route), window, cx)
    }

    /// Appends a complete relative path.
    pub fn push_path(path: NavPath, window: &mut Window, cx: &mut App) {
        let result = Self::try_push_path(path, window, cx);

        if let Err(error) = result {
            error!(%error, "navigation push failed");
        }
    }

    /// Fallible form of [`Navigator::push_path`].
    pub fn try_push_path(
        path: NavPath,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<(), NavigationError> {
        debug!(%path, "pushing path");

        if path.is_empty() {
            return Err(NavigationError::EmptyPath);
        }

        let current = Self::current(cx).ok_or(NavigationError::NoCurrentLocation)?;

        Self::try_go(current.join(path), window, cx)
    }

    /// Replaces the current final route segment.
    pub fn replace<R>(route: R, window: &mut Window, cx: &mut App)
    where
        R: RouteSegment,
    {
        let result = Self::try_replace(route, window, cx);

        if let Err(error) = result {
            error!(%error, "navigation replace failed");
        }
    }

    /// Fallible form of [`Navigator::replace`].
    pub fn try_replace<R>(
        route: R,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<(), NavigationError>
    where
        R: RouteSegment,
    {
        debug!("replacing current route");

        let current = Self::current(cx).ok_or(NavigationError::NoCurrentLocation)?;

        let parent = current.parent().unwrap_or_default();

        Self::try_replace_path(parent.push(route), window, cx)
    }

    /// Replaces the complete current path without modifying history.
    pub fn replace_path(path: NavPath, window: &mut Window, cx: &mut App) {
        let result = Self::try_replace_path(path, window, cx);

        if let Err(error) = result {
            error!(%error, "navigation replace failed");
        }
    }

    /// Fallible form of [`Navigator::replace_path`].
    pub fn try_replace_path(
        path: NavPath,
        window: &mut Window,
        cx: &mut App,
    ) -> Result<(), NavigationError> {
        info!(%path, "replacing current path");

        if path.is_empty() {
            return Err(NavigationError::EmptyPath);
        }

        let dispatch = {
            let navigator = cx
                .try_global::<Self>()
                .ok_or(NavigationError::NotInstalled)?;

            trace!(%path, "retrieving navigation dispatcher");

            Rc::clone(&navigator.root)
        };

        debug!(%path, "dispatching replacement");

        dispatch(cx, window, &path)?;

        cx.update_global::<Self, _>(|navigator, _| {
            navigator.current = Some(path.clone());
        });

        info!(%path, "path replaced");

        Ok(())
    }

    /// Navigates to the previous location.
    pub fn back(window: &mut Window, cx: &mut App) {
        let result = Self::try_back(window, cx);

        if let Err(error) = result {
            error!(%error, "navigation back failed");
        }
    }

    /// Fallible form of [`Navigator::back`].
    pub fn try_back(window: &mut Window, cx: &mut App) -> Result<(), NavigationError> {
        let (dispatch, previous) = {
            let navigator = cx
                .try_global::<Self>()
                .ok_or(NavigationError::NotInstalled)?;

            let previous = navigator
                .back_stack
                .last()
                .cloned()
                .ok_or(NavigationError::NoHistory)?;

            (Rc::clone(&navigator.root), previous)
        };

        info!(%previous, "navigating back");

        debug!(%previous, "dispatching back navigation");

        dispatch(cx, window, &previous)?;

        cx.update_global::<Self, _>(|navigator, _| {
            navigator.back_stack.pop();
            navigator.current = Some(previous.clone());
        });

        info!(%previous, "back navigation completed");

        Ok(())
    }

    /// Moves one structural level toward the root.
    pub fn up(window: &mut Window, cx: &mut App) {
        let result = Self::try_up(window, cx);

        if let Err(error) = result {
            error!(%error, "navigation up failed");
        }
    }

    /// Fallible form of [`Navigator::up`].
    pub fn try_up(window: &mut Window, cx: &mut App) -> Result<(), NavigationError> {
        info!("navigating up");

        let current = Self::current(cx).ok_or(NavigationError::NoCurrentLocation)?;

        let parent = current.parent().ok_or(NavigationError::AlreadyAtRoot)?;

        trace!(%parent, "resolved parent path");

        Self::try_go(parent, window, cx)
    }

    /// Navigates to the root segment.
    pub fn home(window: &mut Window, cx: &mut App) {
        let result = Self::try_home(window, cx);

        if let Err(error) = result {
            error!(%error, "navigation home failed");
        }
    }

    /// Fallible form of [`Navigator::home`].
    pub fn try_home(window: &mut Window, cx: &mut App) -> Result<(), NavigationError> {
        info!("navigating home");

        let current = Self::current(cx).ok_or(NavigationError::NoCurrentLocation)?;

        let root = current.root_path().ok_or(NavigationError::EmptyPath)?;

        trace!(%root, "resolved root path");

        Self::try_go(root, window, cx)
    }

    /// Returns the current location.
    pub fn current(cx: &App) -> Option<NavPath> {
        cx.try_global::<Self>()
            .and_then(|navigator| navigator.current.clone())
    }

    /// Returns whether a previous history entry exists.
    pub fn can_go_back(cx: &App) -> bool {
        cx.try_global::<Self>()
            .is_some_and(|navigator| !navigator.back_stack.is_empty())
    }

    /// Returns whether the current path has a parent.
    pub fn can_go_up(cx: &App) -> bool {
        Self::current(cx).is_some_and(|path| path.parent().is_some())
    }

    /// Returns whether `route` is the current final segment.
    pub fn is_current<R>(route: &R, cx: &App) -> bool
    where
        R: RouteSegment,
    {
        Self::current(cx).is_some_and(|path| path.last_is(route))
    }

    /// Returns whether `route` occurs anywhere in the current path.
    pub fn contains<R>(route: &R, cx: &App) -> bool
    where
        R: RouteSegment,
    {
        Self::current(cx).is_some_and(|path| path.contains(route))
    }

    /// Clears navigation history.
    pub fn clear_history(cx: &mut App) -> Result<(), NavigationError> {
        if cx.try_global::<Self>().is_none() {
            return Err(NavigationError::NotInstalled);
        }

        cx.update_global::<Self, _>(|navigator, _| {
            navigator.back_stack.clear();
        });

        info!("navigation history cleared");

        Ok(())
    }
}
