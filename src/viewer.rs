use core::panic;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{self, Path, PathBuf},
    process::Command,
    str,
    sync::mpsc::{Receiver, Sender, TryRecvError},
};

use crate::{
    interpolation, ipc, parsing::{
        glyph_entries::{parse_tsv, GlyphEntry},
        metadata::{parse_metadata, Metadata},
    }
};
use egui_dock::DockState;
use glifparser::{FlattenedGlif, Glif};
use libmfekufo::{
    blocks::{self, Block},
    glyphs,
};
use mfek_ipc::module::available;

pub struct UFO {
    pub metadata: Metadata,
    pub path: PathBuf,
    pub glyph_entries: Vec<GlyphEntry>,
    pub unicode_blocks: Vec<Block>,
}

//#[derive(Default)]
pub struct UFOViewer {
    pub active_master_idx: Option<usize>,
    pub masters: Vec<UFO>,
    pub dockstate: DockState<usize>,
    pub filter_string: String,
    pub filter_block: Option<String>,
    pub sort_by_blocks: bool,
    pub glyph_name_map: HashMap<String, usize>,
    pub interpolation_check: Option<interpolation::InterpolationCheckResults>,
    should_exit: bool,
    pub dirty: bool,

    // filesystem watching
    pub(crate) filesystem_watch_tx: Sender<path::PathBuf>,
    pub(crate) filesystem_watch_rx: Receiver<path::PathBuf>,
}

impl Default for UFOViewer {
    fn default() -> Self {
        let (fstx, fsrx) = std::sync::mpsc::channel();

        UFOViewer {
            active_master_idx: None,
            dockstate: DockState::new(vec![]),
            filesystem_watch_tx: fstx,
            filesystem_watch_rx: fsrx,
            masters: Default::default(),
            filter_string: Default::default(),
            filter_block: Default::default(),
            sort_by_blocks: Default::default(),
            glyph_name_map: Default::default(),
            should_exit: Default::default(),
            interpolation_check: None,
            dirty: false,
        }
    }
}

impl UFOViewer {
    pub fn get_active_master(&self) -> Option<&UFO> {
        return self.masters.get(self.active_master_idx.unwrap_or(0));
    }

    pub fn set_active_master(&mut self, idx: usize) {
        self.active_master_idx = Some(idx);
    }

    pub fn set_font(&mut self, path:&PathBuf) {
        self.masters = Vec::new();
        self.set_active_master(0);

        let ufo = self.load_ufo_from_path(path);
        self.populate_glyph_name_map(&ufo);
        self.masters.push(ufo);
        self.dockstate.push_to_focused_leaf(self.masters.len() - 1);

        ipc::launch_fs_watcher(self, path);
    }

    pub fn add_master(&mut self, path: &PathBuf) {
        let ufo = self.load_ufo_from_path(path);

        for master in &self.masters {
            if master.path == ufo.path {
                return;
            }
        }

        self.masters.push(ufo);
        self.dockstate.push_to_focused_leaf(self.masters.len() - 1);
        self.dirty = true;
        self.interpolation_check = Some(interpolation::check_interpolatable(&self.masters));
    }

    pub fn load_ufo_from_path(&mut self, path: &PathBuf) -> UFO {
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

            UFO {
                metadata,
                glyph_entries,
                unicode_blocks,
                path: path.clone()
            }
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
                let mut data = data.clone();
                data.sort_by(|a, b| {
                    a.codepoints.cmp(&b.codepoints)
                });
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
                    if p.extension() == Some(OsStr::new("glif")) {
                        // load the glif
                        let mut glif: Glif<()> =
                            glifparser::read_from_filename(&p).expect("Failed to load glyph!");
                        if glif.components.vec.len() > 0 {
                            glif = glif.flattened(&mut None).unwrap_or(glif);
                        }

                        for ufo in &mut self.masters {
                            for potential_match in &mut ufo.glyph_entries {
                                if glif.filename == potential_match.glif.filename {
                                    potential_match.glif = glif.clone();
                                }
                            }
                        }

                        self.interpolation_check = Some(interpolation::check_interpolatable(&self.masters));
                        self.dirty = true;
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
