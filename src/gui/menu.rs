use super::filedialog;
use crate::{interface::Interface, viewer::UFOViewer};

pub fn menu(ctx: &egui::Context, viewer: &mut UFOViewer, interface: &mut Interface) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    match filedialog::open_folder(None) {
                        Some(f) => {
                            viewer.set_font(&f);
                            let font = viewer.get_active_master().unwrap();
                            interface
                                .set_window_title(
                                    format!("MFEKUFO — {0}", font.metadata.name).as_str(),
                                )
                                .expect("Failed to set window title!");
                        }
                        None => {}
                    };
                }

                if viewer.get_active_master().is_some() && ui.button("Add Master").clicked() {
                    match filedialog::open_folder(None) {
                        Some(f) => {
                            viewer.add_master(&f);
                        }
                        None => {}
                    }; 
                }

                if ui.button("Exit").clicked() {
                    viewer.exit();
                }
            });
        });

        let mut filter_string = viewer.filter_string.clone();
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut filter_string);

            if let Some(check) = &viewer.interpolation_check {
                if !check.succeeded {
                    ui.label("Interpolation errors found!");
                }
            }
        });
        viewer.filter_string = filter_string;
    });
}
