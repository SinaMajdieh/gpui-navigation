use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopeId(Arc<str>);

impl ScopeId {
    #[must_use]
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ScopeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<Arc<str>> for ScopeId {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl From<String> for ScopeId {
    fn from(value: String) -> Self {
        Self(Arc::<str>::from(value))
    }
}

impl From<&str> for ScopeId {
    fn from(value: &str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct ScopePath(Arc<[ScopeId]>);

impl ScopePath {
    #[must_use]
    pub fn root() -> Self {
        Self(Arc::from(Vec::<ScopeId>::new()))
    }

    #[must_use]
    pub fn child(&self, id: impl Into<ScopeId>) -> Self {
        let mut segments = self.0.to_vec();
        segments.push(id.into());
        Self(Arc::from(segments))
    }

    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        if self.0.is_empty() {
            return None;
        }

        Some(Self(Arc::from(self.0[..self.0.len() - 1].to_vec())))
    }

    #[must_use]
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn segments(&self) -> impl ExactSizeIterator<Item = &ScopeId> {
        self.0.iter()
    }

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
            f.write_str(segment.as_str())?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_and_navigates_hierarchical_paths() {
        let root = ScopePath::root();
        let workspace = root.child("workspace");
        let settings = workspace.child("settings");

        assert!(root.is_root());
        assert_eq!(workspace.to_string(), "/workspace");
        assert_eq!(settings.to_string(), "/workspace/settings");
        assert!(workspace.is_prefix_of(&settings));
        assert_eq!(settings.parent(), Some(workspace));
        assert_eq!(root.parent(), None);
    }
}
