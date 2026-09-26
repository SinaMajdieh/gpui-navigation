use gpui_kit::base::NavMotion;

/// Configuration for the application-wide navigation registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NavigatorConfig {
    /// Transition used by operations that do not specify a per-call motion.
    pub default_motion: NavMotion,
}

impl NavigatorConfig {
    /// Creates the default navigator configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            default_motion: NavMotion::Animated,
        }
    }

    /// Returns a copy using `motion` as the default transition.
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
