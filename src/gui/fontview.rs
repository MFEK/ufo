use egui::{Pos2, Rect};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use libmfekufo::glyphs::GlyphRef;

use crate::{interface::Interface, parsing::glyph_entries::GlyphEntry, viewer::{UFOViewer, UFO}};

pub fn fontview(ctx: &egui::Context, viewer: &mut UFOViewer, interface: &mut Interface) {
    let interface_size = interface.get_size();
    let window_rect = Rect::from_two_pos(
        Pos2::new(0., 24.),
        Pos2::new(interface_size.0, interface_size.1),
    );

    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(ufo) = &viewer.ufo {
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut viewer.filter_string);
                ui.checkbox(&mut viewer.sort_by_blocks, "Group by blocks?")
            });
            
            egui::ScrollArea::vertical()
                .stick_to_right(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(window_rect.width());
                    ui.set_max_width(window_rect.width());
                    if !viewer.sort_by_blocks {
                        ui.horizontal_wrapped(|ui| {
                            ui.set_max_width(window_rect.width());
                            for entry in
                                filter_glyphs(&ufo.glyph_entries, &viewer.filter_string.to_lowercase())
                            {
                                let glyph_image =
                                    viewer
                                        .ufo_cache
                                        .get_image_handle(ui.ctx(), entry, &ufo.metadata);
    
                                ui.add(egui::ImageButton::new(glyph_image, [128., 128.]));
                            }
                        });
                    } else {
                        for block in &ufo.unicode_blocks {
                            ui.collapsing(block.name, |ui| {
                                ui.horizontal_wrapped(|ui| {
                                    for gref in &block.glyphs {
                                        println!("{:?}", gref);
                                        let entry_idx = viewer.glyph_name_map.get(&gref.name).expect("Something went wrong.");
                                        let entry = &ufo.glyph_entries[*entry_idx];

                                        let glyph_image =
                                        viewer
                                            .ufo_cache
                                            .get_image_handle(ui.ctx(), entry, &ufo.metadata);
        
                                        ui.add(egui::ImageButton::new(glyph_image, [128., 128.]));
                                    }
                                });
                            });
                        }
                    }
                });
        } else {
            ui.vertical_centered(|ui| {
                ui.label(
                    "No font has been loaded! Please go to File > Open and open a valid UFO font.",
                );
            });
        }
    });
}

fn filter_glyphs<'a>(glyph_entries: &'a [GlyphEntry], query: &str) -> Vec<&'a GlyphEntry> {
    let matcher = SkimMatcherV2::default();

    let mut scored_entries: Vec<(&'a GlyphEntry, i64)> = glyph_entries
        .iter()
        .filter_map(|entry| {
            // Calculate string similarity using the Skim algorithm
            let score = matcher.fuzzy_match(&entry.glifname.to_lowercase(), &query);

            match score {
                Some(score) => Some((entry, score)),
                None => None,
            }
        })
        .collect();

    // Sort scored_entries by score in descending order
    scored_entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));

    // Create a new Vec containing only the references to GlyphEntry
    scored_entries.into_iter().map(|(entry, _)| entry).collect()
}
