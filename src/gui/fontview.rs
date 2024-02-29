use std::{collections::HashSet, process::Command};

use egui::{style::WidgetVisuals, Color32, Stroke, Style};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::{interface::Interface, parsing::glyph_entries::GlyphEntry, ufo_cache::UFOCache, viewer::UFOViewer};

pub fn fontview(ctx: &egui::Context, viewer: &mut UFOViewer, ufo_cache: &mut UFOCache) {
    ufo_cache.create_default_texture(ctx);

    if let Some(ufo) = viewer.get_active_master() {
        ufo_cache.rebuild_images(ctx, &ufo.metadata, &viewer.interpolation_check);
    }

    ufo_cache.clear_rebuild();
    viewer.handle_filesystem_events(ufo_cache);

    filter_side_panel(ctx, viewer);

    let original_style = ctx.style().clone();

    let frame = WidgetVisuals {
        bg_fill: Color32::from_white_alpha(0),
        weak_bg_fill: Color32::from_white_alpha(0),
        bg_stroke: Stroke::new(8., Color32::from_white_alpha(0)),
        rounding: egui::Rounding::default(),
        fg_stroke: Stroke::new(8., Color32::from_white_alpha(0)),
        expansion: 0.,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut filter_string = viewer.filter_string.clone();
        let mut master_idx = viewer.active_master_idx;

        if let Some(ufo) = viewer.get_active_master() {
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut filter_string);

                egui::ComboBox::from_label("Master")
                    .selected_text(format!("{:1}", ufo.metadata.name))
                    .show_ui(ui, |ui| {
                        for (idx, master) in viewer.masters.iter().enumerate() {
                            ui.selectable_value(&mut master_idx, Some(idx), master.metadata.name.clone());
                        }
                    });

                if let Some(check) = &viewer.interpolation_check {
                    if !check.succeeded {
                        ui.label("Interpolation errors found!");
                    }
                }
            });

            egui::ScrollArea::vertical()
                .stick_to_right(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
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
                            filter_glyphs(&ufo.glyph_entries, &viewer.filter_string.to_lowercase());

                        let filtered_set: HashSet<_> = filtered_vec.into_iter().cloned().collect();

                        let visible_set: HashSet<_> = if let Some(block_name) = &viewer.filter_block
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
                                let glyph_image = ufo_cache.get_image_handle(&entry, &ufo.metadata);

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
        } else {
            ui.vertical_centered(|ui| {
                ui.label(
                    "No font has been loaded! Please go to File > Open and open a valid UFO font.",
                );
            });
        }

        viewer.filter_string = filter_string;
        if let Some(idx) = master_idx {
            viewer.set_active_master(idx);
        }
    });

    ctx.set_style(original_style);
}

fn filter_side_panel(ctx: &egui::Context, viewer: &mut UFOViewer) {
    let mut filter_block = viewer.filter_block.to_owned();

    if let Some(ufo) = viewer.get_active_master() {
        egui::SidePanel::left("my_left_panel").show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_right(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if ui
                        .selectable_label(viewer.filter_block.is_none(), "All")
                        .clicked()
                    {
                        filter_block = None;
                    }

                    for block in &ufo.unicode_blocks {
                        let checked = Some(block.name) == filter_block.as_deref();
                        if ui.selectable_label(checked, block.name).clicked() {
                            filter_block = Some(block.name.to_owned());
                        }
                    }
                });
        });
    }

    viewer.filter_block = filter_block;
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
