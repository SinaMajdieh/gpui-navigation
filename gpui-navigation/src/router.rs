use std::rc::Rc;

use gpui_kit::{App, AppContext, Context, Entity, Window};
use thiserror::Error;

use crate::{NavPath, PathError, RouteSegment};

/// A router owns one level of a navigation hierarchy.
///
/// A router consumes exactly one route segment and returns the router that
/// should receive the remainder of the path.
///
/// Returning `None` means that this route is a leaf destination.
pub trait Router: Sized + 'static {
    /// The route type owned by this router.
    type Route: RouteSegment;

    /// Handles one route segment.
    ///
    /// The navigation system has already extracted this router's segment.
    /// The returned router, when present, receives the remaining path.
    fn route(
        &mut self,
        route: &Self::Route,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<RouterHandle>;
}

/// A type-erased handle to a concrete router.
///
/// `Router` itself cannot be turned into a useful trait object because every
/// router has its own associated `Route` type. `RouterHandle` is the erased
/// continuation used internally by the navigation system.
///
/// Users normally create one implicitly with:
///
/// ```ignore
/// Some(self.child.clone().into())
/// ```
#[derive(Clone)]
pub struct RouterHandle {
    dispatch: Rc<dyn Fn(&mut App, &mut Window, &NavPath) -> Result<(), NavigationError>>,
}

impl RouterHandle {
    /// Creates a handle for a concrete router entity.
    pub fn new<R>(router: Entity<R>) -> Self
    where
        R: Router,
    {
        Self {
            dispatch: Rc::new(move |cx, window, path| navigate(&router, path, window, cx)),
        }
    }

    /// Dispatches a path into this router.
    pub(crate) fn dispatch(
        &self,
        cx: &mut App,
        window: &mut Window,
        path: &NavPath,
    ) -> Result<(), NavigationError> {
        (self.dispatch)(cx, window, path)
    }
}

impl<R> From<Entity<R>> for RouterHandle
where
    R: Router,
{
    fn from(router: Entity<R>) -> Self {
        Self::new(router)
    }
}

/// Dispatches a path into one concrete router.
///
/// The dispatcher:
///
/// 1. consumes exactly one path segment using [`NavPath::head`];
/// 2. asks the router what should handle the remainder;
/// 3. stops when the path has been fully consumed;
/// 4. otherwise dispatches the remainder into the returned router.
pub(crate) fn navigate<R>(
    router: &Entity<R>,
    path: &NavPath,
    window: &mut Window,
    cx: &mut App,
) -> Result<(), NavigationError>
where
    R: Router,
{
    let Some((route, remaining)) = path.head::<R::Route>()? else {
        return Err(NavigationError::EmptyPath);
    };

    let next = cx.update_entity(router, |router, cx| router.route(route, window, cx));

    // The current router consumed the final segment.
    //
    // Even if the router returned Some(...), there is nothing left to route,
    // so the continuation is simply ignored.
    if remaining.is_empty() {
        return Ok(());
    }

    // There are more segments, therefore this router must provide the next
    // router in the hierarchy.
    let next = next.ok_or_else(|| NavigationError::UnexpectedTrailingSegments {
        router: std::any::type_name::<R>(),
        remaining: remaining.len(),
    })?;

    next.dispatch(cx, window, &remaining)
}

/// Errors produced by navigation operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NavigationError {
    /// No global navigator has been installed in the application.
    #[error("the global navigator has not been installed")]
    NotInstalled,

    /// An application attempted to install more than one navigator.
    #[error(
        "the global navigator is already installed; installing it more than once is not supported"
    )]
    AlreadyInstalled,

    /// An absolute navigation operation received an empty path.
    #[error("navigation requires a non-empty path")]
    EmptyPath,

    /// A router expected a different route segment type.
    #[error(transparent)]
    Path(#[from] PathError),

    /// A router consumed its route but the path still contained segments and
    /// the router did not provide a child router.
    #[error("router `{router}` could not consume {remaining} trailing route segment(s)")]
    UnexpectedTrailingSegments {
        /// Type name of the router that stopped navigation.
        router: &'static str,

        /// Number of unconsumed segments.
        remaining: usize,
    },

    /// Relative navigation was requested before an initial location existed.
    #[error("there is no current navigation location")]
    NoCurrentLocation,

    /// Back navigation was requested without history.
    #[error("there is no previous navigation location")]
    NoHistory,

    /// Up navigation was requested while already at the root.
    #[error("the current navigation location is already at the root")]
    AlreadyAtRoot,
}
