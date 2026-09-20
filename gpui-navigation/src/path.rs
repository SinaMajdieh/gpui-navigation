use std::{fmt, rc::Rc};

use thiserror::Error;

use crate::RouteSegment;

/// One erased route segment stored inside [`crate::NavPath`].
///
/// [`Rc`] is used because paths are cloned when locations enter navigation
/// history. Cloning a segment therefore clones only the reference-counted
/// handle instead of allocating and cloning the underlying route value.
#[derive(Clone)]
struct Segment(Rc<dyn RouteSegment>);

impl fmt::Display for Segment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_display(formatter)
    }
}

impl Segment {
    fn new<R>(route: R) -> Self
    where
        R: RouteSegment,
    {
        Self(Rc::new(route))
    }

    fn as_route<R>(&self) -> Option<&R>
    where
        R: RouteSegment,
    {
        self.0.as_any().downcast_ref::<R>()
    }

    fn type_name(&self) -> &'static str {
        self.0.type_name()
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        self.0.equals(other.0.as_ref())
    }
}

impl Eq for Segment {}

impl fmt::Debug for Segment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_debug(formatter)
    }
}

/// Errors produced while inspecting or consuming a navigation path.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The requested route type does not match the segment at `index`.
    #[error("navigation path segment {index} has type `{actual}`, but `{expected}` was expected")]
    TypeMismatch {
        /// Zero-based path index.
        index: usize,

        /// Route type requested by the caller.
        expected: &'static str,

        /// Route type actually stored at the index.
        actual: &'static str,
    },
}

/// An ordered sequence of independent route segments.
///
/// A path such as:
///
/// ```text
/// Workspace::Projects / Projects::Open(42) / Project::Firmware
/// ```
///
/// is represented by three independent route values. No route enum contains
/// another route enum, so adding a new level never changes an existing route.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct NavPath {
    segments: Vec<Segment>,
}

impl fmt::Debug for NavPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(&self.segments).finish()
    }
}

impl fmt::Display for NavPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, segment) in self.segments.iter().enumerate() {
            if index > 0 {
                formatter.write_str("/")?;
            }

            segment.fmt(formatter)?;
        }

        Ok(())
    }
}

impl NavPath {
    /// Creates an empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a path containing one route segment.
    pub fn root<R>(route: R) -> Self
    where
        R: RouteSegment,
    {
        Self::new().then(route)
    }

    /// Appends one route segment to the path.
    pub fn then<R>(mut self, route: R) -> Self
    where
        R: RouteSegment,
    {
        self.segments.push(Segment::new(route));
        self
    }

    /// Appends every segment from `other` to this path.
    pub fn join(mut self, other: Self) -> Self {
        self.segments.extend(other.segments);
        self
    }

    /// Returns the number of route segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }

    /// Returns `true` when the path contains no segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Returns the first segment as `R` and the remaining path.
    ///
    /// This is the primary operation used by [`crate::Router`] implementations.
    /// An empty path returns `Ok(None)`.
    pub fn head<R>(&self) -> Result<Option<(&R, Self)>, PathError>
    where
        R: RouteSegment,
    {
        let Some(segment) = self.segments.first() else {
            return Ok(None);
        };

        let Some(route) = segment.as_route::<R>() else {
            return Err(PathError::TypeMismatch {
                index: 0,
                expected: std::any::type_name::<R>(),
                actual: segment.type_name(),
            });
        };

        Ok(Some((route, self.tail())))
    }

    /// Returns the first route segment as `R`.
    ///
    /// An empty path returns `Ok(None)`.
    pub fn first<R>(&self) -> Result<Option<&R>, PathError>
    where
        R: RouteSegment,
    {
        self.at(0)
    }

    /// Returns the route segment at `index` as `R`.
    ///
    /// An index outside the path returns `Ok(None)`.
    pub fn at<R>(&self, index: usize) -> Result<Option<&R>, PathError>
    where
        R: RouteSegment,
    {
        let Some(segment) = self.segments.get(index) else {
            return Ok(None);
        };

        let Some(route) = segment.as_route::<R>() else {
            return Err(PathError::TypeMismatch {
                index,
                expected: std::any::type_name::<R>(),
                actual: segment.type_name(),
            });
        };

        Ok(Some(route))
    }

    /// Returns the final route segment as `R`.
    ///
    /// An empty path returns `Ok(None)`.
    pub fn last<R>(&self) -> Result<Option<&R>, PathError>
    where
        R: RouteSegment,
    {
        match self.segments.len() {
            0 => Ok(None),
            len => self.at(len - 1),
        }
    }

    /// Returns a path containing all segments after the first.
    pub fn tail(&self) -> Self {
        if self.segments.len() <= 1 {
            return Self::new();
        }

        Self {
            segments: self.segments[1..].to_vec(),
        }
    }

    /// Returns the structural parent of this path.
    ///
    /// For example, `A / B / C` becomes `A / B`. `None` is returned when the
    /// path is already at its root.
    pub fn parent(&self) -> Option<Self> {
        if self.segments.len() <= 1 {
            return None;
        }

        Some(Self {
            segments: self.segments[..self.segments.len() - 1].to_vec(),
        })
    }

    /// Returns a path containing only the first segment.
    pub fn root_path(&self) -> Option<Self> {
        self.segments.first().cloned().map(|segment| Self {
            segments: vec![segment],
        })
    }

    /// Returns whether `route` equals the final segment.
    pub fn last_is<R>(&self, route: &R) -> bool
    where
        R: RouteSegment,
    {
        self.segments
            .last()
            .is_some_and(|segment| segment.0.equals(route))
    }

    /// Returns whether `route` occurs anywhere in this path.
    pub fn contains<R>(&self, route: &R) -> bool
    where
        R: RouteSegment,
    {
        self.segments.iter().any(|segment| segment.0.equals(route))
    }
}

impl<R> From<R> for NavPath
where
    R: RouteSegment,
{
    fn from(route: R) -> Self {
        Self::root(route)
    }
}

/// Constructs a `NavPath` from a sequence of route segments.
///
/// This macro provides a concise way to define navigation paths, replacing
/// the builder pattern `NavPath::root(A).then(B).then(C)`.
///
/// # Example
///```rust
/// use gpui_navigation::nav_path;
///
/// // Create a simple path
/// let path = nav_path![WorkspaceRoute::Projects, ProjectsRoute::Home];
///
/// // Create an empty path
/// let empty = nav_path![];
/// ```
#[macro_export]
macro_rules! nav_path {
    () => {
        $crate::NavPath::empty()
    };
    ( $head:expr $(, $tail:expr )* $(,)? ) => {{
        let mut path = $crate::NavPath::root(($head).clone());
        $(
            path = path.then(($tail).clone());
        )*
        path
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, crate::RouteSegment)]
    enum Section {
        Projects,
    }

    #[derive(crate::RouteSegment, Debug, PartialEq, Eq)]
    enum Project {
        Open(u64),
        Overview,
    }

    #[test]
    fn builds_and_joins_paths() {
        let path = NavPath::root(Section::Projects).then(Project::Open(42));

        assert_eq!(path.len(), 2);
        assert_eq!(path.first::<Section>().ok(), Some(Some(&Section::Projects)));
        assert_eq!(path.last::<Project>().ok(), Some(Some(&Project::Open(42))));

        let suffix = NavPath::new().then(Project::Open(7));
        let joined = path.join(suffix);

        assert_eq!(joined.len(), 3);
        assert_eq!(joined.at::<Project>(2).ok(), Some(Some(&Project::Open(7))));
    }

    #[test]
    fn parent_and_root_are_structural_operations() {
        let path = NavPath::root(Section::Projects).then(Project::Open(42));

        assert_eq!(path.parent().as_ref().map(NavPath::len), Some(1));
        assert_eq!(path.root_path().as_ref().map(NavPath::len), Some(1));
        assert!(
            path.parent()
                .is_some_and(|parent| { parent.last_is(&Section::Projects) })
        );
    }

    #[test]
    fn type_mismatch_is_reported() {
        let path = NavPath::root(Section::Projects);

        let result = path.first::<Project>();

        assert!(matches!(
            result,
            Err(PathError::TypeMismatch { index: 0, .. })
        ));
    }

    #[test]
    fn contains_and_last_match_values() {
        let path = NavPath::root(Section::Projects).then(Project::Open(42));

        assert!(path.contains(&Section::Projects));
        assert!(path.last_is(&Project::Open(42)));
        assert!(!path.last_is(&Project::Overview));
    }
}
