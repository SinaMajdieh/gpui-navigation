#![deny(warnings)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Typed, hierarchical navigation for GPUI desktop applications.
//!
//! The crate models application locations as sequences of independent route
//! segments. A feature owns its own route enum and does not need to know which
//! workspace, screen, or parent router contains it.
//!
//! The central types are:
//!
//! - [`RouteSegment`] — one typed destination owned by a feature.
//! - [`NavPath`] — an ordered sequence of route segments.
//! - [`Router`] — consumes the portion of a path owned by one view.
//! - [`Navigator`] — performs navigation and maintains history.
//!
//! ## Route declaration
//!
//! The [`RouteSegment`] derive macro is re-exported from this crate, so a
//! consumer only needs one dependency:
//!
//! ```ignore
//! use gpui_navigation::RouteSegment;
//!
//! #[derive(RouteSegment, Clone, Copy, Debug, PartialEq, Eq)]
//! pub enum ProjectsRoute {
//!     List,
//!     Open(u64),
//! }
//! ```
//!
//! ## Relative navigation
//!
//! A feature normally navigates using only its own route type:
//!
//! ```ignore
//! Navigator::push(ProjectRoute::Firmware, cx);
//! ```
//!
//! No parent route needs to be mentioned at the call site.
//!
//! ## Absolute paths
//!
//! A complete location can be constructed from independent segments:
//!
//! ```ignore
//! let path = NavPath::root(WorkspaceRoute::Projects)
//!     .push(ProjectsRoute::Open(42))
//!     .push(ProjectRoute::Firmware);
//!
//! Navigator::go(path, cx);
//! ```
//!
//! ## Desktop navigation semantics
//!
//! [`Navigator::back`] follows navigation history, while [`Navigator::up`]
//! follows the structural parent of the current path. This keeps browser-like
//! history separate from desktop hierarchy.

use std::{any::Any, fmt};

pub use gpui_navigation_derive::RouteSegment;

mod navigator;
mod path;
mod router;

pub use navigator::Navigator;
pub use path::{NavPath, PathError};
pub use router::{NavigationError, Router, RouterHandle};

/// A single strongly-typed route segment.
///
/// Route values are erased only when stored together in a heterogeneous
/// [`NavPath`]. Feature modules normally implement this trait through the
/// [`RouteSegment`] derive macro.
pub trait RouteSegment: Any {
    /// Returns the concrete route as [`Any`] for checked downcasting inside a
    /// navigation path.
    fn as_any(&self) -> &dyn Any;

    /// Compares this route with another type-erased route segment.
    fn equals(&self, other: &dyn RouteSegment) -> bool;

    /// Returns the concrete Rust type name of this route.
    fn type_name(&self) -> &'static str;

    /// Formats this route using its [`Debug`] implementation.
    fn fmt_debug(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Formats this route using its [`Display`] implementation.
    fn fmt_display(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;
}
