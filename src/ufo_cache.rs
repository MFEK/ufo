use std::{collections::HashMap, default};

use egui::{Context, TextureHandle, TextureId, Color32};
use glifparser::{MFEKGlif, PointData, error::GlifParserError, Glif, FlattenedGlif};
use glifrenderer::{viewport::Viewport, toggles::{PointLabels, HandleStyle, PreviewMode}};
use image::flat::View;
use skia_safe::{Color, Color4f, Paint, Rect, Surface, Matrix};

use std::{ffi::OsString as Oss, fs, path::Path};

use mfek_ipc::module::available;

use crate::parsing::{glyph_entries::{GlyphEntry, self}, metadata::Metadata};
pub struct Texture<'a> {
    texture_handle: &'a TextureHandle,
}

#[derive(Default)]
pub struct UFOCache {
    texture_handles: HashMap<GlyphEntry, TextureHandle>,
}

impl UFOCache {
    pub fn get_image_handle(&mut self, ctx: &Context, glyph_entry: &GlyphEntry, metadata: &Metadata) -> &TextureHandle {
        self.generate_image_handle(ctx, glyph_entry, metadata);

        return self.texture_handles.get(glyph_entry).as_ref().unwrap();
    }

    fn generate_image_handle(&mut self, ctx: &Context, glyph_entry: &GlyphEntry, metadata: &Metadata) {
        if self.texture_handles.contains_key(glyph_entry) {
            return;
        }

        // load the glif
        let mut glif: Glif<()> = glifparser::read_from_filename(&glyph_entry.filename).expect("Failed to load glyph!");
        if glif.components.vec.len() > 0 {
           glif = glif.flattened(&mut None).unwrap_or(glif);
        }
        let mfekglif: MFEKGlif<()> = MFEKGlif::from(glif);

        // create the viewport
        let ascender = metadata.ascender;
        let descender = metadata.descender;
        let mut viewport = UFOCache::create_viewport_for_glyph_centered(&mfekglif, 1000., ascender, descender);
        let (size, image_data) = self.create_canvas_and_get_image_data(&mfekglif, &mut viewport);
        let egui_image = egui::ColorImage::from_rgba_unmultiplied([size, size], &image_data);

        let texture_handle =
            ctx.load_texture("my-image", egui_image, Default::default());

        self.texture_handles
            .insert(glyph_entry.clone(), texture_handle);
    }

    pub fn create_viewport_for_glyph_centered(
        glyph: &MFEKGlif<()>,
        units_per_em: f32,
        ascender: i32,
        descender: i32,
    ) -> Viewport {
        let canvas_size = 128.0;
        let factor = canvas_size / (ascender - descender) as f32 * 0.8;
        let glyph_width = units_per_em;
        let x_offset = (canvas_size / 2.0) - (494. * factor / 2.0);
        let y_offset = descender as f32 * factor;
    
    
        let mut viewport = Viewport::default();
        viewport.winsize = (canvas_size, canvas_size);
        viewport.factor = factor;
        viewport.offset = (x_offset, y_offset);
        viewport.preview_mode = PreviewMode::Paper;
        return viewport;
    }
    
    fn create_canvas_and_get_image_data(&mut self, mfekglif: &MFEKGlif<()>, viewport: &mut Viewport) -> (usize, Vec<u8>) {
        let dimension: usize = 128;
        // Create a Surface with the desired width and height
        let mut surface = Surface::new_raster_n32_premul((dimension as i32, dimension as i32)).unwrap();

        // Get the Canvas from the Surface
        let canvas = surface.canvas();

        // Clear the canvas with a white background
        canvas.clear(Color::WHITE);

        viewport.redraw(canvas);
        glifrenderer::glyph::draw(canvas, mfekglif, viewport);
        // Get the ImageInfo from the Surface
        let image_info = surface.image_info();

        // Create a buffer to store the image data
        let row_bytes = image_info.min_row_bytes();
        let size = (row_bytes * (dimension));
        let mut image_data = vec![0u8; size];

        // Read the pixels from the Surface into the buffer
        let success = surface.read_pixels(&image_info, &mut image_data, row_bytes, (0, 0));
        assert!(success, "Failed to read pixels from the Surface");

        (dimension, image_data)
    }

    pub fn load_glif_impl<F: AsRef<Path> + Clone>(&mut self, file: F) -> MFEKGlif<()> {
        // TODO: Actually handle errors now that we have them.
        return {
            let ext = file.as_ref().extension().map(|e| e.to_ascii_lowercase());
            let ext_or = ext
                .unwrap_or(Oss::from("glif"))
                .to_string_lossy()
                .into_owned();
            let mut tempglif: MFEKGlif<_> = match ext_or.as_str() {
                "glifjson" => {
                    serde_json::from_str(&fs::read_to_string(&file).expect("Could not open file"))
                        .expect("Could not deserialize JSON MFEKGlif")
                }
                "glif" => glifparser::read_from_filename(&file)
                    .expect("Invalid glif!")
                    .into(),
                _ => {
                    panic!("Failed to load glif file!");
                }
            };

            tempglif.filename = Some(file.as_ref().to_path_buf());

            tempglif
        };
    }
}
