use gpui_kit::base::NavMotion;

/// Configuration for the application-wide navigation registry.
///
/// A [`NavigatorConfig`] controls behavior shared by navigation operations
/// that do not provide their own per-operation configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavigatorConfig {
    /// Transition used when a navigation operation does not specify a motion.
    pub default_motion: NavMotion,
}

impl NavigatorConfig {
    /// Creates a configuration with [`NavMotion::Animated`] as the default
    /// transition.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            default_motion: NavMotion::Animated,
        }
    }

    /// Returns a configuration with `motion` as the default transition.
    ///
    /// This method does not modify the original configuration.
    #[must_use]
    pub const fn with_default_motion(self, motion: NavMotion) -> Self {
        Self {
            default_motion: motion,
        }
    }
}

impl Default for NavigatorConfig {
    fn default() -> Self {
        Self::new()
    }
}
