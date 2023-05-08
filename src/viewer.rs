use core::panic;
use std::{path::{PathBuf, Path}, process::{Command, ExitStatus}, str};

use mfek_ipc::module::{available, binaries};
use crate::{ufo_cache::UFOCache, parsing::{glyph_entries::{GlyphEntry, parse_tsv, self}, metadata::{parse_metadata, Metadata}}};

pub struct UFO {
    pub metadata: Metadata,
    pub glyph_entries: Vec<GlyphEntry>
}

#[derive(Default)]
pub struct UFOViewer {
    pub ufo: Option<UFO>,
    pub ufo_cache: UFOCache,
    should_exit: bool,
}

impl UFOViewer {
    pub fn set_font(&mut self, path: PathBuf) {
        if let Ok((v, pbuf)) = available("metadata", "0.0.4") {
            match v {
                mfek_ipc::module::Version::OutOfDate(_) => log::warn!("Version mismatch found with mfekmetadata!"),
                _ => {}
            }

            let glyph_entries = self.fetch_glyph_entries(&pbuf, &path);
            let metadata = self.fetch_metadata(&pbuf,&path);

            let ufo = UFO {
                metadata,
                glyph_entries
            };

            self.ufo = Some(ufo);
            self.ufo_cache = UFOCache::default();
        } else {
            panic!("Failed to locate mfekmetadata! Is it installed on your system?")
        }
    }

    fn fetch_glyph_entries<P: AsRef<Path>>(&mut self, metadata_path: P, font_path: P) -> Vec<GlyphEntry> {
        let output = Command::new(metadata_path.as_ref())
            .args([font_path.as_ref().to_str().unwrap(), "glyphs"])
            .output()
            .expect("Call to list glyphs from mfekmetadata failed!");

        if !output.status.success() {
            panic!("mfekmetadata returned a non-zero exit code: {0}", output.status.code().unwrap())
        }

            // Convert the stdout Vec<u8> to a &str
        let stdout_str = str::from_utf8(&output.stdout)
            .expect("The command's output was not valid UTF-8");

        // Pass the &str to the parse_tsv function
        match parse_tsv(stdout_str) {
            Ok(data) => {
                return data;
            }
            Err(err) => panic!("Error parsing TSV data: {}", err),
        }
    }

    fn fetch_metadata<P: AsRef<Path>>(&mut self, metadata_path: P, font_path: P) -> Metadata {
        let metadata_path_str = font_path.as_ref().to_str().expect("Failed to convert font path to str.");
        // let's get the familyName of the ufo and store that in the viewer
        let output = Command::new(metadata_path.as_ref())
            .args([metadata_path_str, "arbitrary", "-k", "postscriptFullName", "-k", "ascender", "-k", "descender", "-k", "copyright"])
            .output()
            .expect("Failed to run mfekmetadata command to find familyName!");

        if !output.status.success() {
            panic!("mfekmetadata exited with a non-zero exit code: {:?} \n {:?}", output.status.code().unwrap(), output)
        }

        let stdout_str = str::from_utf8(&output.stdout)
            .expect("The command's output was not valid UTF-8");

        match parse_metadata(stdout_str) {
            Ok(data) => {
                return data;
            }
            Err(err) => {
                panic!("Error parsing metadata: {}", err);
            }
        }
    }

    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    pub fn is_requesting_exit(&self) -> bool {
        self.should_exit
    }
}
