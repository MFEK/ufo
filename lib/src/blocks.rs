use std::collections::{BTreeMap, HashSet};
use std::fmt;

use itertools::Itertools;

use crate::glyphs::{GlyphRef, UnicodeData};
use crate::util::Log;

// (char @ beginning block, char @ end of block)
pub type CharRange = (char, char);
// &str is the name of a block
pub type UnicodeBlocksMap = BTreeMap<CharRange, &'static str>;
// usize is an index into UnicodeBlocksMap, and also assures correct order not based on block name
pub struct Block {
    pub range: Option<CharRange>, // allows for an "Unassigned" block to hold all the glyphs with no assigned codepoint
    pub name: &'static str,
    pub glyphs: Vec<GlyphRef>,
}
pub type UnicodeBlocksGroupedMap = BTreeMap<usize, Block>;

use unic_ucd_block::Block as UcdBlock;
pub fn for_unicode_data(unique_encodings: &HashSet<UnicodeData>) -> UnicodeBlocksMap {
    let mut blocks = UnicodeBlocksMap::new();
    for encoding in unique_encodings {
        UcdBlock::of(encoding.encoding).map(|b| blocks.insert((b.range.low, b.range.high), b.name));
    }
    blocks.log();
    blocks
}

impl Log for UnicodeBlocksMap {
    fn log(&self) {
        log::info!(
            "{} blocks. {}.",
            self.len(),
            self
                .iter()
                .map(|(k, v)| format!("{} ({:04X}–{:04X})", v, k.0 as u32, k.1 as u32))
                .join(", ")
        );
    }
}

pub fn grouped_by(gvec: &[GlyphRef], blocks: &UnicodeBlocksMap) -> UnicodeBlocksGroupedMap {
    let mut grouped_by = UnicodeBlocksGroupedMap::new();

    blocks.iter().enumerate().for_each(|(i, key_name)| {
        let ((low, high), name) = key_name;
        let grs = gvec.iter().filter(|gr| gr.unicode.iter().any(|u| { u.encoding >= *low && u.encoding <= *high })).map(|v|v.to_owned()).collect();
        grouped_by.insert(i, Block { range: Some((*low, *high)), name, glyphs: grs });
    });

    let inserted_unencoded = grouped_by.insert(grouped_by.len(), Block { range: None, name: "Unencoded", glyphs: gvec.iter().filter(|gr|gr.unicode.len()==0).map(|v|v.to_owned()).collect() }).is_none();
    debug_assert!(inserted_unencoded);

    grouped_by.log();

    grouped_by
}

impl Log for UnicodeBlocksGroupedMap {
    fn log(&self) {
        log::debug!("Glyphs, grouped by block:");
        for (k, v) in self.iter() {
            log::debug!("§{} — {} ⇒ {}", k + 1, v, v.glyphs.iter().map(|vv|format!("{}", vv)).join(", "));
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some((low, high)) = self.range {
            let (low, high) = (low as u32, high as u32);
            write!(f, "{} ({}/{})", self.name, self.glyphs.len(), high - low)
        } else {
            write!(f, "{} ({})", self.name, self.glyphs.len())
        }
    }
}
