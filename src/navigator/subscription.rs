//! Navigation-event subscriptions.
//!
//! This module provides access to [`NavStackEvent`] notifications emitted by
//! an existing navigation stack.

use gpui_kit::{Context, Subscription, base::NavStackEvent};

use crate::{NavigationResult, registry::NavigationRegistry};

use super::Navigator;

impl Navigator {
    /// Subscribes the current entity to navigation events emitted by the
    /// targeted scope.
    ///
    /// The returned [`Subscription`] should normally be retained by the
    /// subscribing GPUI entity so that the subscription remains active for the
    /// desired lifetime.
    ///
    /// This operation requires the target scope to already have a registered
    /// navigation stack; it does not create one.
    ///
    /// Errors are reported through [`tracing::error!`] and `None` is returned.
    /// Use [`Self::try_subscribe`] when explicit error handling is required.
    pub fn subscribe<T: 'static>(
        &self,
        cx: &mut Context<T>,
        handler: impl FnMut(&mut T, &NavStackEvent, &mut Context<T>) + 'static,
    ) -> Option<Subscription> {
        self.report("subscribe", self.try_subscribe(cx, handler))
    }

    /// Subscribes the current entity to navigation events emitted by the
    /// targeted scope.
    ///
    /// The target scope must already have a registered navigation stack.
    ///
    /// # Errors
    ///
    /// Returns [`crate::NavigatorError::ScopeNotFound`] if the target scope does
    /// not have a registered stack.
    pub fn try_subscribe<T: 'static>(
        &self,
        cx: &mut Context<T>,
        mut handler: impl FnMut(&mut T, &NavStackEvent, &mut Context<T>) + 'static,
    ) -> NavigationResult<Subscription> {
        let path = self.try_resolved_path(cx)?;
        let state = NavigationRegistry::resolve_state(&path, cx)?;

        Ok(cx.subscribe(&state, move |this, _, event, cx| {
            handler(this, event, cx);
        }))
    }
}
