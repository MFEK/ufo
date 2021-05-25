use std::convert::TryInto;
use std::process::Command;

use mfek_ipc as ipc;
use itertools::Itertools;

mod types;
pub use types::{GlyphRef, UnicodeData};

use std::collections::HashSet;
pub fn for_ufo(ufodir: String) -> Vec<GlyphRef> {
    let (status, cmd) = ipc::module_available("metadata");
    assert!(status.assert(), "MFEKmetadata unavailable; cannot continue");
    let mut glyphs_cmd = Command::new(cmd);

    glyphs_cmd
        .args(&[&ufodir, "glyphs"]);

    let status = glyphs_cmd.status().expect("MFEKmetadata failed to run");

    assert!(status.success(), "MFEKmetadata failed to run");

    let glyphs = glyphs_cmd
        .output()
        .expect("MFEKmetadata failed to run");
    let glyphs = String::from_utf8(glyphs.stdout).unwrap();

    let mut gvec: Vec<GlyphRef> = glyphs
        .split("\n")
        .map(|s| s.split("\t").filter(|w| w != &"").collect())
        .collect::<Vec<Vec<_>>>()
        .iter()
        .filter(|g| (g.len() > 0))
        .map(|f| {
            let (uniname, encoding, category) = if f.len() > 1 {
                (
                    Some(f[2].split(",").collect::<Vec<&str>>()),
                    Some(
                        f[1].split(",")
                            .map(|u| {
                                u32::from_str_radix(u, 16)
                                    .expect(&format!("{} not base 16 int?", u))
                                    .try_into()
                                    .expect(&format!("{} not representable as char?", u))
                            })
                            .collect::<Vec<char>>(),
                    ),
                    Some(f[3].split(",").collect::<Vec<&str>>()),
                )
            } else {
                (None, None, None)
            };

            let unicode = if let (Some(u), Some(e), Some(c)) = (uniname, encoding, category) {
                itertools::multizip((u, e, c))
                    .map(
                        |(name, encoding, category)| UnicodeData {
                            name: name.to_string(),
                            encoding,
                            category: category.to_string(),
                        },
                    )
                    .collect()
            } else {
                vec![]
            };

            GlyphRef {
                name: f[0].to_string(),
                unicode,
            }
        })
        .collect();
    // This sort order is decided by the Ord implementation for GlyphRef. Glyphs with encodings are
    // sorted by their first encoded slot, then by their name if they lack an encoding.
    gvec.sort();

    log::info!(
        "{} glyphs. {}.",
        gvec.len(),
        gvec.iter()
            .map(|gr| format!(
                "{} ({})",
                gr.name,
                if gr.unicode.len() > 0 {
                    gr.unicode
                        .iter()
                        .map(|u| {
                            format!(
                                "{}{}",
                                if u.category.contains("Nonspacing") {
                                    "\u{25CC}" // ◌ DOTTED CIRCLE
                                } else {
                                    ""
                                },
                                u.encoding
                            )
                        })
                        .join(" ,")
                } else {
                    "-1".to_owned()
                }
            ))
            .join(", ")
    );

    gvec
}

pub fn to_unique_codepoints(gvec: &[GlyphRef]) -> HashSet<UnicodeData> {
    let mut unique_encodings: HashSet<UnicodeData> = HashSet::new();
    for gr in gvec.iter() {
        gr.unicode.iter().for_each(|ud| {
            if !unique_encodings.insert(ud.clone()) {
                log::warn!("Two glyphs with identical encoding in font: U+{0:04X}! Try `grep -R {0:04X}` on glyphs dir.", ud.encoding as u32);
            }
        });
    }
    unique_encodings
}
