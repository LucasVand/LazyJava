use std::fmt::Display;

#[derive(Eq, Clone, PartialEq, Hash, Debug)]
pub struct Package {
    path: Vec<String>,
    is_static: bool,
}

impl Package {
    pub fn empty() -> Package {
        Package {
            path: Vec::new(),
            is_static: false,
        }
    }
    pub fn from_string(s: impl Into<String>, is_static: bool) -> Package {
        let s: String = s.into();
        Package {
            path: s.split('.').into_iter().map(|s| s.to_string()).collect(),
            is_static,
        }
    }

    pub fn includes(&self, other: &Self) -> bool {
        for (i, s) in self.path.iter().enumerate() {
            if s == "*" {
                // `*` matches exactly one segment, like a Java package
                // import: `import com.example.*` only imports classes
                // directly in `com.example`, not subpackages.
                return self.path.len() == other.path.len();
            }

            match other.path.get(i) {
                Some(o) if o == s => {}
                _ => return false,
            }
        }

        self.path.len() == other.path.len()
    }
    pub fn push(&mut self, s: impl Into<String>) {
        self.path.push(s.into());
    }
    pub fn pop(&mut self) -> Option<String> {
        self.path.pop()
    }
    pub fn is_wildcard(&self) -> bool {
        return self.path.last() == Some(&"*".to_string());
    }
    pub fn join(&self, seg: impl Into<String>) -> Package {
        let mut c = self.clone();
        c.push(seg.into());
        c
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }
    pub fn remove_static(&mut self) {
        self.is_static = false;
    }
}
impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_static() {
            write!(f, "static ")?;
        }
        let join = self.path.join(".");
        write!(f, "{join}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Package;

    fn pkg(s: &str) -> Package {
        Package::from_string(s, false)
    }

    #[test]
    fn exact_match_is_included() {
        assert!(pkg("com.example").includes(&pkg("com.example")));
    }

    #[test]
    fn different_segment_is_not_included() {
        assert!(!pkg("com.other").includes(&pkg("com.example")));
    }

    #[test]
    fn shorter_other_is_not_included() {
        assert!(!pkg("com.example.lib").includes(&pkg("com.example")));
    }

    #[test]
    fn longer_other_is_not_included() {
        assert!(!pkg("com.example").includes(&pkg("com.example.lib")));
    }

    #[test]
    fn missing_trailing_segment_is_not_included() {
        assert!(!pkg("a.b.c").includes(&pkg("a.b")));
    }

    #[test]
    fn wildcard_matches_single_segment() {
        assert!(pkg("*").includes(&pkg("Main")));
        assert!(!pkg("*").includes(&pkg("com.example.lib")));
    }

    #[test]
    fn wildcard_requires_matching_prefix() {
        assert!(!pkg("org.*").includes(&pkg("com.example")));
        assert!(pkg("org.*").includes(&pkg("org.apache")));
        assert!(!pkg("org.*").includes(&pkg("org.apache.maven")));
    }

    #[test]
    fn wildcard_matches_exactly_one_segment() {
        assert!(!pkg("com.example.*").includes(&pkg("com.example")));
        assert!(pkg("com.example.*").includes(&pkg("com.example.lib")));
        assert!(!pkg("com.example.*").includes(&pkg("com.example.deep.nested.pkg")));
    }

    #[test]
    fn empty_package_matches_empty_package() {
        assert!(pkg("").includes(&pkg("")));
    }

    #[test]
    fn empty_package_does_not_match_non_empty() {
        assert!(!pkg("").includes(&pkg("com")));
        assert!(!pkg("com").includes(&pkg("")));
    }
}
