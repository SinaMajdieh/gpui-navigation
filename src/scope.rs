use std::{
    any::{TypeId, type_name},
    fmt,
    sync::Arc,
};

/// A type used as the identity of a navigation scope.
///
/// Scope identity is determined by the concrete type, not by the value passed
/// to [`Navigator::scope`](crate::Navigator::scope). This makes lightweight
/// marker types suitable for defining application navigation scopes.
///
/// ```ignore
/// #[derive(Clone, Copy, Debug)]
/// pub struct Workspace;
///
/// #[derive(Clone, Copy, Debug)]
/// pub struct Settings;
/// ```
///
/// A marker value is only a convenient way to name its scope:
/// `Navigator::new().scope(Workspace)`.
///
/// Different values of the same type therefore identify the same scope.
///
/// The trait is implemented automatically for every type that satisfies its
/// bounds; applications normally do not need to implement it manually.
pub trait Scope: Copy + Send + Sync + 'static {}

impl<T> Scope for T where T: Copy + Send + Sync + 'static {}

/// An opaque identifier for a navigation scope type.
///
/// A [`ScopeId`] is derived from the concrete [`Scope`] type and is independent
/// of any value of that type. Two calls to [`ScopeId::of`] with the same type
/// produce equal identifiers.
///
/// The identifier retains both the [`TypeId`] used for identity comparisons and
/// the Rust type name used for display and diagnostics.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ScopeId {
    type_id: TypeId,
    name: &'static str,
}

impl ScopeId {
    /// Creates the identifier associated with scope type `S`.
    #[must_use]
    pub fn of<S: Scope>() -> Self {
        Self {
            type_id: TypeId::of::<S>(),
            name: type_name::<S>(),
        }
    }

    /// Returns the [`TypeId`] backing this scope identifier.
    ///
    /// The returned [`TypeId`] can be used when interoperability with APIs
    /// based on runtime type identity is required.
    #[must_use]
    pub const fn type_id(self) -> TypeId {
        self.type_id
    }

    /// Returns the Rust type name associated with this scope identifier.
    ///
    /// This is the name returned by [`std::any::type_name`] for the scope type
    /// and is also the representation used by [`Display`](fmt::Display).
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl fmt::Debug for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ScopeId").field(&self.name).finish()
    }
}

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// A hierarchical path of typed navigation scopes.
///
/// A path consists of zero or more [`ScopeId`] values ordered from the root
/// scope to the current scope. The root path contains no segments and is
/// displayed as `/`.
///
/// `ScopePath` is immutable: operations such as [`Self::child`] and
/// [`Self::parent`] return new paths rather than modifying the existing path.
///
/// Internally, the segments are stored in an [`Arc`] so cloning a path is
/// inexpensive until a new path needs to be constructed.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct ScopePath(Arc<[ScopeId]>);

impl ScopePath {
    /// Returns the root navigation path.
    ///
    /// The root path contains no scope segments.
    #[must_use]
    pub fn root() -> Self {
        Self(Arc::from(Vec::<ScopeId>::new()))
    }

    /// Returns a child path containing the scope identified by `S`.
    ///
    /// The value itself is not used for identity; only its concrete type
    /// determines the resulting scope.
    #[must_use]
    pub fn child<S: Scope>(&self, _scope: S) -> Self {
        self.child_type::<S>()
    }

    /// Returns a child path identified by scope type `S`.
    ///
    /// This is equivalent to [`Self::child`] without requiring a value of the
    /// scope type.
    #[must_use]
    pub fn child_type<S: Scope>(&self) -> Self {
        self.child_id(ScopeId::of::<S>())
    }

    /// Returns a child path containing `id` as its final segment.
    ///
    /// This is the internal counterpart to [`Self::child`] and
    /// [`Self::child_type`], allowing the registry to extend paths from an
    /// already constructed [`ScopeId`].
    pub(crate) fn child_id(&self, id: ScopeId) -> Self {
        let mut segments = self.0.to_vec();
        segments.push(id);
        Self(Arc::from(segments))
    }

    /// Returns the parent path.
    ///
    /// Returns `None` when called on the root path. Otherwise, the returned path
    /// contains every segment except the final one.
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            return None;
        }

        Some(Self(Arc::from(self.0[..self.0.len() - 1].to_vec())))
    }

    /// Returns `true` if this is the root path.
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of scope segments in this path.
    ///
    /// The root path has a depth of `0`.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the path's scope segments from root to leaf.
    ///
    /// The iterator's length is exactly [`Self::depth`].
    pub fn segments(&self) -> impl ExactSizeIterator<Item = &ScopeId> {
        self.0.iter()
    }

    /// Returns whether this path is an ancestor of, or equal to, `other`.
    ///
    /// In other words, `self` is a prefix of `other` when every segment in
    /// `self` occurs at the beginning of `other` in the same order.
    ///
    /// A path is therefore considered a prefix of itself, and the root path is
    /// a prefix of every path.
    #[must_use]
    pub fn is_prefix_of(&self, other: &Self) -> bool {
        self.0.len() <= other.0.len()
            && self
                .0
                .iter()
                .zip(other.0.iter())
                .all(|(left, right)| left == right)
    }
}

impl fmt::Display for ScopePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_root() {
            return f.write_str("/");
        }

        for segment in &*self.0 {
            f.write_str("/")?;
            f.write_str(segment.name())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct Workspace;

    #[derive(Clone, Copy, Debug)]
    struct Settings;

    #[test]
    fn builds_typed_hierarchical_paths() {
        let root = ScopePath::root();
        let workspace = root.child(Workspace);
        let settings = workspace.child(Settings);

        assert!(root.is_root());
        assert_eq!(workspace.depth(), 1);
        assert_eq!(settings.depth(), 2);
        assert!(workspace.is_prefix_of(&settings));
        assert_eq!(settings.parent(), Some(workspace));
        assert_eq!(root.parent(), None);
    }

    #[test]
    fn scope_identity_is_type_based() {
        #[derive(Clone, Copy)]
        #[allow(unused)]
        struct A(u8);

        let a = ScopeId::of::<A>();
        assert_eq!(a, ScopeId::of::<A>());
        assert_eq!(a.type_id(), TypeId::of::<A>());
    }
}
