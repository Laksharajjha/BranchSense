//! Build metadata exposed by `BranchSense` entry points.

use std::fmt;

/// Immutable metadata identifying a `BranchSense` executable build.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildInfo {
    name: &'static str,
    version: &'static str,
}

impl BuildInfo {
    /// Creates build metadata from compile-time package values.
    #[must_use]
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self { name, version }
    }

    /// Returns the executable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the semantic version of this executable.
    #[must_use]
    pub const fn version(self) -> &'static str {
        self.version
    }
}

impl fmt::Display for BuildInfo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::BuildInfo;

    #[test]
    fn display_contains_name_and_version() {
        let build_info = BuildInfo::new("branchsense", "0.1.0");

        assert_eq!(build_info.to_string(), "branchsense 0.1.0");
    }
}
