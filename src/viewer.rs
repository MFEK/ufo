use core::panic;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{self, Path, PathBuf},
    process::{Command, ExitStatus},
    str,
    sync::mpsc::{Receiver, Sender, TryRecvError},
};

use crate::{
    ipc,
    parsing::{
        glyph_entries::{self, parse_tsv, GlyphEntry},
        metadata::{parse_metadata, Metadata},
    },
    ufo_cache::UFOCache,
};
use glifparser::{FlattenedGlif, Glif, MFEKGlif};
use libmfekufo::{
    blocks::{self, Block},
    glyphs,
};
use mfek_ipc::module::{available, binaries};

pub struct UFO {
    pub metadata: Metadata,
    pub glyph_entries: Vec<GlyphEntry>,
    pub unicode_blocks: Vec<Block>,
}

//#[derive(Default)]
pub struct UFOViewer {
    pub ufo: Option<UFO>,
    pub ufo_cache: UFOCache,
    pub filter_string: String,
    pub filter_block: Option<String>,
    pub sort_by_blocks: bool,
    pub glyph_name_map: HashMap<String, usize>,
    should_exit: bool,

    // filesystem watching
    pub(crate) filesystem_watch_tx: Sender<path::PathBuf>,
    pub(crate) filesystem_watch_rx: Receiver<path::PathBuf>,
}

impl Default for UFOViewer {
    fn default() -> Self {
        let (fstx, fsrx) = std::sync::mpsc::channel();

        UFOViewer {
            filesystem_watch_tx: fstx,
            filesystem_watch_rx: fsrx,
            ufo: Default::default(),
            ufo_cache: Default::default(),
            filter_string: Default::default(),
            filter_block: Default::default(),
            sort_by_blocks: Default::default(),
            glyph_name_map: Default::default(),
            should_exit: Default::default(),
        }
    }
}

impl UFOViewer {
    pub fn set_font(&mut self, path: PathBuf) {
        if let Ok((v, pbuf)) = available("metadata", "0.0.4") {
            match v {
                mfek_ipc::module::Version::OutOfDate(_) => {
                    log::warn!("Version mismatch found with mfekmetadata!")
                }
                _ => {}
            }

            let glyph_entries = self.fetch_glyph_entries(&pbuf, &path);
            let metadata = self.fetch_metadata(&pbuf, &path);
            let unicode_blocks = Self::get_unicode_blocks(path.clone());

            let ufo = UFO {
                metadata,
                glyph_entries,
                unicode_blocks,
            };

            self.populate_glyph_name_map(&ufo);

            self.ufo = Some(ufo);
            self.ufo_cache = UFOCache::default();
            ipc::launch_fs_watcher(self, path);
        } else {
            panic!("Failed to locate mfekmetadata! Is it installed on your system?")
        }
    }

    fn populate_glyph_name_map(&mut self, ufo: &UFO) {
        self.glyph_name_map.clear();

        for (idx, entry) in ufo.glyph_entries.iter().enumerate() {
            self.glyph_name_map.insert(entry.glifname.clone(), idx);
        }
    }

    fn get_unicode_blocks<P: AsRef<Path>>(path: P) -> Vec<Block> {
        let gvec = glyphs::for_ufo(path.as_ref().to_str().unwrap().to_owned());
        let unique_encodings = glyphs::to_unique_codepoints(&gvec);
        let blocks = blocks::for_unicode_data(&unique_encodings);
        blocks::grouped_by(&gvec, &blocks)
    }

    fn fetch_glyph_entries<P: AsRef<Path>>(
        &mut self,
        metadata_path: P,
        font_path: P,
    ) -> Vec<GlyphEntry> {
        let output = Command::new(metadata_path.as_ref())
            .args([font_path.as_ref().to_str().unwrap(), "glyphs"])
            .output()
            .expect("Call to list glyphs from mfekmetadata failed!");

        if !output.status.success() {
            panic!(
                "mfekmetadata returned a non-zero exit code: {0}",
                output.status.code().unwrap()
            )
        }

        // Convert the stdout Vec<u8> to a &str
        let stdout_str =
            str::from_utf8(&output.stdout).expect("The command's output was not valid UTF-8");

        // Pass the &str to the parse_tsv function
        match parse_tsv(stdout_str) {
            Ok(data) => {
                return data;
            }
            Err(err) => panic!("Error parsing TSV data: {}", err),
        }
    }

    fn fetch_metadata<P: AsRef<Path>>(&mut self, metadata_path: P, font_path: P) -> Metadata {
        let metadata_path_str = font_path
            .as_ref()
            .to_str()
            .expect("Failed to convert font path to str.");
        // let's get the familyName of the ufo and store that in the viewer
        let output = Command::new(metadata_path.as_ref())
            .args([
                metadata_path_str,
                "arbitrary",
                "-k",
                "postscriptFullName",
                "-k",
                "ascender",
                "-k",
                "descender",
                "-k",
                "copyright",
            ])
            .output()
            .expect("Failed to run mfekmetadata command to find familyName!");

        if !output.status.success() {
            panic!(
                "mfekmetadata exited with a non-zero exit code: {:?} \n {:?}",
                output.status.code().unwrap(),
                output
            )
        }

        let stdout_str =
            str::from_utf8(&output.stdout).expect("The command's output was not valid UTF-8");

        match parse_metadata(stdout_str) {
            Ok(data) => {
                return data;
            }
            Err(err) => {
                panic!("Error parsing metadata: {}", err);
            }
        }
    }

    pub fn handle_filesystem_events(&mut self) {
        loop {
            let event = self.filesystem_watch_rx.try_recv();
            match event {
                Ok(p) => {
                    if p.extension() == Some(OsStr::new("glif"))
                        || p.extension() == Some(OsStr::new("glifjson"))
                    {
                        // load the glif
                        let mut glif: Glif<()> =
                            glifparser::read_from_filename(&p).expect("Failed to load glyph!");
                        if glif.components.vec.len() > 0 {
                            glif = glif.flattened(&mut None).unwrap_or(glif);
                        }

                        let ufo = self.ufo.as_ref().unwrap();

                        for potential_match in &ufo.glyph_entries {
                            if glif.name == potential_match.glifname {
                                self.ufo_cache.force_rebuild(potential_match);
                            }
                        }
                    } else {
                        log::debug!("Ignored write of file {:?}", p)
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(_) => panic!("Filesystem watcher disconnected!"),
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
