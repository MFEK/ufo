use egui::{Pos2, Rect};

use crate::{
    interface::Interface,
    viewer::{self, UFOViewer},
};

pub fn fontview(ctx: &egui::Context, viewer: &mut UFOViewer, interface: &mut Interface) {
    let interface_size = interface.get_size();
    let window_rect = Rect::from_two_pos(
        Pos2::new(0., 24.),
        Pos2::new(interface_size.0, interface_size.1),
    );

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("fontview_grid").show(ui, |ui| {
                if let Some(ufo) = &viewer.ufo {
                    let mut count = 0;
                    for entry in &ufo.glyph_entries {
                        ui.vertical(|ui: &mut egui::Ui| {
                            let glyph_image = viewer.ufo_cache.get_image_handle(ui.ctx(), entry, &ufo.metadata);
                            ui.image(glyph_image, glyph_image.size_vec2());

                            let name_string: &String = &entry.glifname;
                            ui.centered(|ui| {
                                ui.label(name_string);
                            });
                        });

                        count += 1;

                        if count >= 4 {
                            count = 0;
                            ui.end_row();
                        }
                    }
                }
            });
        });
    });
}
