use std::fmt::Display;

/// A builder for separator-delimited strings (e.g. javac `-classpath` /
/// `-processorpath`), where individual entries may carry a trailing `/*` glob
/// marker.
///
/// This avoids hand-assembling strings like
/// `"{dir1}:{dir2}/*:{dir3}/*"` and keeps the trailing `/*` decision next to
/// each entry.
pub struct SeperatorList {
    sep: char,
    entries: Vec<String>,
}

impl SeperatorList {
    pub fn new(sep: char) -> Self {
        Self {
            sep,
            entries: Vec::new(),
        }
    }

    /// Add an entry with no suffix.
    pub fn add(mut self, value: impl Display) -> Self {
        self.entries.push(value.to_string());
        self
    }

    /// Add an entry followed by a `/*` glob marker.
    pub fn add_glob(mut self, value: impl Display) -> Self {
        self.entries.push(format!("{value}/*"));
        self
    }

    /// Add each value in the slice as a plain entry.
    pub fn add_slice(mut self, values: &[impl Display]) -> Self {
        for value in values {
            self.entries.push(value.to_string());
        }
        self
    }

    /// Add each value in the slice followed by a `/*` glob marker.
    pub fn add_slice_glob(mut self, values: &[impl Display]) -> Self {
        for value in values {
            self.entries.push(format!("{value}/*"));
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Join all entries with the separator and yield the final string.
    pub fn build(self) -> String {
        self.entries.join(&self.sep.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::SeperatorList;

    #[test]
    fn empty_builds_empty_string() {
        assert_eq!(SeperatorList::new(':').build(), "");
    }

    #[test]
    fn joins_with_seperator() {
        let list = SeperatorList::new(';')
            .add("a")
            .add("b")
            .add("c")
            .build();
        assert_eq!(list, "a;b;c");
    }

    #[test]
    fn supports_glob_suffix_per_entry() {
        let list = SeperatorList::new(':')
            .add("lib")
            .add_glob("jars")
            .add_glob("more")
            .build();
        assert_eq!(list, "lib:jars/*:more/*");
    }

    #[test]
    fn adds_plain_slice() {
        let list = SeperatorList::new(';').add_slice(&["a", "b", "c"]).build();
        assert_eq!(list, "a;b;c");
    }

    #[test]
    fn adds_glob_slice() {
        let list = SeperatorList::new(';')
            .add_slice_glob(&["a", "b", "c"])
            .build();
        assert_eq!(list, "a/*;b/*;c/*");
    }

    #[test]
    fn mixes_single_and_slice() {
        let list = SeperatorList::new(':')
            .add("base")
            .add_slice_glob(&["a", "b"])
            .build();
        assert_eq!(list, "base:a/*:b/*");
    }
}