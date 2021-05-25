use std::fmt;

#[derive(Debug, Clone)]
pub struct UnicodeData {
    pub name: String,
    pub category: String,
    pub encoding: char,
}

impl PartialEq for UnicodeData {
    fn eq(&self, other: &Self) -> bool {
        self.encoding == other.encoding
    }
}
impl Eq for UnicodeData {}

use std::cmp::Ordering;
impl PartialOrd for UnicodeData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for UnicodeData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.encoding.cmp(&other.encoding)
    }
}

use std::hash::{Hash, Hasher};
impl Hash for UnicodeData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.encoding.hash(state);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct GlyphRef {
    pub name: String,
    pub unicode: Vec<UnicodeData>,
}
impl Eq for GlyphRef {}

impl PartialOrd for GlyphRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GlyphRef {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.unicode.len() > 0 && other.unicode.len() > 0 {
            self.unicode[0].encoding.cmp(&other.unicode[0].encoding)
        } else if self.unicode.len() > 0 {
            Ordering::Less
        } else if other.unicode.len() > 0 {
            Ordering::Greater
        } else {
            self.name.cmp(&other.name)
        }
    }
}

impl fmt::Display for GlyphRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.name)
    }
}
