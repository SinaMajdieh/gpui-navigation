#![deny(unsafe_code, missing_docs)]

//! # GPUI Navigation
//!
//! A lightweight, typed navigation layer for applications built with
//! [`gpui_kit`].
//!
//! The crate provides an application-wide [`Navigator`] façade over
//! [`NavStackState`] instances. Navigation scopes are identified by Rust types
//! rather than strings, allowing applications to organize independent
//! navigation stacks into a hierarchical [`ScopePath`].
//!
//! ## Installation
//!
//! Install the global navigation registry during application initialization:
//!
//! ```ignore
//! Navigator::install(cx, NavigatorConfig::default());
//! ```
//!
//! Use [`Navigator::try_install`] when initialization errors need to be
//! handled explicitly.
//!
//! ## Basic navigation
//!
//! A navigator can target the root stack directly:
//!
//! ```ignore
//! Navigator::new().push(view, cx);
//! ```
//!
//! Typed child scopes can be selected with marker types:
//!
//! ```ignore
//! #[derive(Clone, Copy, Debug)]
//! struct Workspace;
//!
//! #[derive(Clone, Copy, Debug)]
//! struct Settings;
//!
//! Navigator::new()
//!     .scope(Workspace)
//!     .scope(Settings)
//!     .push(view, cx);
//! ```
//!
//! Scope values are identified by their concrete Rust type, so different
//! values of the same scope type refer to the same navigation scope.
//!
//! ## Current-scope navigation
//!
//! [`Navigator::current_scope`] creates a navigator whose target is resolved
//! lazily against the application's currently active scope:
//!
//! ```ignore
//! Navigator::current_scope()
//!     .scope(Settings)
//!     .push(view, cx);
//! ```
//!
//! The current scope is application-global. It is not inferred from the GPUI
//! entity performing the operation and is not window-local.
//!
//! ## Error handling
//!
//! Every fallible navigation operation has two forms. The ordinary methods such
//! as [`Navigator::push`] report failures through [`tracing::error!`] and
//! otherwise return a convenient fallback value. Their `try_*` counterparts,
//! such as [`Navigator::try_push`], return [`NavigationResult`] so callers can
//! handle errors explicitly.
//!
//! ## Navigation model
//!
//! Each [`ScopePath`] identifies an independent [`NavStackState`]. Navigators
//! can therefore be scoped without passing stack entities through the view
//! hierarchy. Scopes may be created lazily by navigation operations and can be
//! removed together with their descendants.
//!
//! [`gpui_kit`]: https://docs.rs/gpui-kit
//! [`NavStackState`]: gpui_kit::base::NavStackState

mod config;
mod error;
mod navigator;
mod registry;
mod scope;

pub use config::NavigatorConfig;
pub use error::{NavigationResult, NavigatorError};
pub use navigator::Navigator;
pub use scope::{Scope, ScopeId, ScopePath};

pub use gpui_kit::base::{
    NavMotion, NavOperation, NavPage, NavStack, NavStackEvent, NavStackState,
};
