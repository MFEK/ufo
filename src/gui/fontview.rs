use egui_dock::{DockArea, Style};

use crate::{ufo_cache::UFOCache, viewer::UFOViewer, gui::mastertab::MasterTabViewer};

pub fn fontview(ctx: &egui::Context, viewer: &mut UFOViewer, ufo_cache: &mut UFOCache) {
    ufo_cache.create_default_texture(ctx);

    if let Some(ufo) = viewer.get_active_master() {
        ufo_cache.rebuild_images(ctx, &ufo.metadata, &viewer.interpolation_check);
    }

    ufo_cache.clear_rebuild();
    viewer.handle_filesystem_events();

    filter_side_panel(ctx, viewer);

    let original_style = ctx.style().clone();

        if viewer.get_active_master().is_some() {
            DockArea::new(&mut viewer.dockstate)
                .show_close_buttons(false)
                .draggable_tabs(true)
                .allowed_splits(egui_dock::AllowedSplits::TopBottomOnly)
                .style({
                    let mut style = Style::from_egui(ctx.style().as_ref());
                    style.tab_bar.fill_tab_bar = true;
                    style
                })
                .show(
                    ctx,
                    &mut MasterTabViewer {
                        masters: &mut viewer.masters,
                        ufo_cache,
                        filter_string: viewer.filter_string.clone(),
                        filter_block: viewer.filter_block.clone(),
                    }
                );
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        "No font has been loaded! Please go to File > Open and open a valid UFO font.",
                    );
                });
            });
        }
        
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
