use std::{collections::HashSet, process::Command};

use egui::{style::WidgetVisuals, Color32, Stroke, Style};
use egui_dock::TabViewer;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{parsing::glyph_entries::GlyphEntry, ufo_cache::UFOCache, viewer::UFO};

pub struct MasterTabViewer <'a> {
    pub masters: &'a mut Vec<UFO>,
    pub ufo_cache: &'a mut UFOCache,
    pub filter_string: String,
    pub filter_block: Option<String>,
}

impl<'a> TabViewer for MasterTabViewer<'a> {
    type Tab = usize;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.masters[*tab].metadata.name.clone().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        println!("UID {tab}");
        let filter_string = &self.filter_string;
        let filter_block = &self.filter_block;
        let ufo = &self.masters[*tab];
        egui::ScrollArea::vertical()
            .stick_to_right(true)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let frame = WidgetVisuals {
                    bg_fill: Color32::from_white_alpha(0),
                    weak_bg_fill: Color32::from_white_alpha(0),
                    bg_stroke: Stroke::new(8., Color32::from_white_alpha(0)),
                    rounding: egui::Rounding::default(),
                    fg_stroke: Stroke::new(8., Color32::from_white_alpha(0)),
                    expansion: 0.,
                };
                
                ui.set_style(Style {
                    visuals: egui::Visuals {
                        widgets: egui::style::Widgets {
                            active: frame, // Set the custom frame style for ImageButtons
                            inactive: frame,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                });

                ui.set_width(ui.available_width());
                ui.set_max_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    let filtered_vec: Vec<&GlyphEntry> =
                        filter_glyphs(&ufo.glyph_entries, &filter_string.to_lowercase());

                    let filtered_set: HashSet<_> = filtered_vec.into_iter().cloned().collect();

                    let visible_set: HashSet<_> = if let Some(block_name) = &filter_block
                    {
                        ufo.unicode_blocks
                            .iter()
                            .find(|block| block.name == *block_name)
                            .map(|block| {
                                let block_set: HashSet<String> =
                                    block.glyphs.iter().map(|x| x.name.clone()).collect();
                                filtered_set
                                    .iter()
                                    .filter(|x| block_set.contains(&x.glifname))
                                    .cloned()
                                    .collect()
                            })
                            .unwrap_or_else(HashSet::new)
                    } else {
                        filtered_set
                    };

                    ufo.glyph_entries
                        .iter()
                        .filter(|entry| visible_set.contains(entry))
                        .for_each(|entry| {
                            let glyph_image = self.ufo_cache.get_image_handle(&entry);

                            let response =
                                ui.add(egui::ImageButton::new(glyph_image, [128., 128.]));

                            if response.clicked() {
                                let glif_filename = entry.filename.clone();

                                Command::new("MFEKglif")
                                    .arg(glif_filename)
                                    .spawn()
                                    .expect("Couldn't open MFEKglif! Is it installed?");
                            }
                        });
                });
            });
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }
}


fn filter_glyphs<'a>(glyph_entries: &'a [GlyphEntry], query: &str) -> Vec<&'a GlyphEntry> {
    let matcher = SkimMatcherV2::default();

    let scored_entries: Vec<_> = glyph_entries
        .iter()
        .filter_map(|entry| {
            // Calculate string similarity using the Skim algorithm
            let score = matcher.fuzzy_match(&entry.glifname.to_lowercase(), &query);

            score.map(|s| (entry, s))
        })
        .collect();

    // Sort scored_entries by score in descending order
    let mut sorted_entries: Vec<_> = scored_entries.into_iter().map(|(entry, _)| entry).collect();
    sorted_entries.sort_unstable_by_key(|entry| {
        let score = matcher.fuzzy_match(&entry.glifname.to_lowercase(), &query);
        -(score.unwrap_or(0))
    });

    sorted_entries
}
