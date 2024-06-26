use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use egui::{Context, TextureHandle};
use glifparser::{FlattenedGlif, Glif, MFEKGlif};
use glifrenderer::{glyph::Style, toggles::PreviewMode, viewport::Viewport};
use skia_safe::{Color, Color4f, Font, Paint, Point, Surface, TextBlob, Typeface};

use crate::{interpolation, parsing::{glyph_entries::GlyphEntry, metadata::Metadata}};

#[derive(Default)]
pub struct UFOCache {
    default_texture: Option<TextureHandle>,
    texture_handles: HashMap<GlyphEntry, TextureHandle>,
    needs_rebuild: VecDeque<GlyphEntry>,
}

impl UFOCache {
    pub fn get_image_handle(
        &mut self,
        glyph_entry: &GlyphEntry,
    ) -> &TextureHandle {
        let texture_handle = self.texture_handles.get(glyph_entry);

        if let Some(txhandle) = texture_handle {
            return txhandle;
        } else {
            self.needs_rebuild.push_front(glyph_entry.clone());
            return self.default_texture.as_ref().unwrap();
        }
    }

    pub fn create_default_texture(&mut self, ctx: &Context) {
        if self.default_texture.is_some() {
            return;
        }
        let (size, image_data) = Self::create_default_image();
        let egui_image = egui::ColorImage::from_rgba_unmultiplied([size, size], &image_data);

        let texture_handle = ctx.load_texture("default", egui_image, Default::default());
        self.default_texture = Some(texture_handle);
    }

    pub fn force_rebuild_all(&mut self) {
        let entries_to_remove: Vec<_> = self.texture_handles.keys().cloned().collect();
    
        for entry in entries_to_remove {
            self.texture_handles.remove(&entry);
            self.needs_rebuild.push_front(entry);
        }
    }

    pub fn rebuild_images(&mut self, ctx: &Context, metadata: &Metadata, interp_check: &Option<interpolation::InterpolationCheckResults>) {
        let time_limit = 1. / 30.;
        let start_time = Instant::now();

        while start_time.elapsed().as_secs_f32() < time_limit {
            let to_rebuild = self.needs_rebuild.pop_back();
            if let Some(entry) = to_rebuild {
                let mut interp_success = true;

                if let Some(interp_info) = interp_check {
                    if interp_info.combined.get(&entry.uniname).is_some() {
                        interp_success = false;
                    }
                }
                self.generate_image_handle(ctx, &entry, metadata, interp_success)
            } else {
                break;
            }
        }
    }

    pub fn clear_rebuild(&mut self) {
        self.needs_rebuild = VecDeque::new();
    }

    fn generate_image_handle(
        &mut self,
        ctx: &Context,
        glyph_entry: &GlyphEntry,
        metadata: &Metadata,
        interp_success: bool,
    ) {
        if self.texture_handles.contains_key(glyph_entry) {
            return;
        }

        let mut glif: Glif<()> = glyph_entry.glif.clone();
        if glif.components.vec.len() > 0 {
            glif = glif.flattened(&mut None).unwrap_or(glif);
        }

        let glif_name = glif.name.clone();
        let mfekglif: MFEKGlif<()> = MFEKGlif::from(glif);

        // create the viewport
        let ascender = metadata.ascender;
        let descender = metadata.descender;
        let mut viewport =
            UFOCache::create_viewport_for_glyph_centered(&mfekglif, ascender, descender);

        let egui_text_color: Color = Color::new(u32::from_le_bytes(
            ctx.style().visuals.text_color().to_array().into(),
        ));

        let (size, image_data) =
            self.create_canvas_and_get_image_data(&mfekglif, &mut viewport, egui_text_color, interp_success);
        let egui_image = egui::ColorImage::from_rgba_unmultiplied([size, size], &image_data);

        let texture_handle = ctx.load_texture(glif_name, egui_image, Default::default());

        self.texture_handles
            .insert(glyph_entry.clone(), texture_handle);
    }

    pub fn create_viewport_for_glyph_centered(
        glyph: &MFEKGlif<()>,
        ascender: i32,
        descender: i32,
    ) -> Viewport {
        let canvas_size = 128.0;
        let factor = canvas_size / (ascender - descender + 12) as f32 * 0.6;
        let glyph_width = glyph.width.unwrap_or(0);
        let x_offset = glyph_width as f32 / 2.0;
        let y_offset = (ascender as f32 - descender as f32)/2.;
    
        let mut viewport = Viewport::default();
        viewport.winsize = (canvas_size, canvas_size);
        viewport.factor = factor;
        viewport.offset = (-x_offset, y_offset);
        viewport.preview_mode = PreviewMode::Paper;
        
        return viewport;
    }
    

    fn create_default_image() -> (usize, Vec<u8>) {
        let dimension = 128;

        // Create a Surface with the desired width and height
        let mut surface =
            Surface::new_raster_n32_premul((dimension as i32, dimension as i32)).unwrap();

        // Get the Canvas from the Surface
        let canvas = surface.canvas();

        // Clear the canvas with a white background
        canvas.clear(Color4f::new(0., 0., 0., 0.));

        // Get the ImageInfo from the Surface
        let image_info = surface.image_info();

        // Create a buffer to store the image data
        let row_bytes = image_info.min_row_bytes();
        let size = row_bytes * (dimension);
        let mut image_data = vec![0u8; size];

        // Read the pixels from the Surface into the buffer
        let success = surface.read_pixels(&image_info, &mut image_data, row_bytes, (0, 0));
        assert!(success, "Failed to read pixels from the Surface");

        (dimension, image_data)
    }

    fn create_canvas_and_get_image_data(
        &mut self,
        mfekglif: &MFEKGlif<()>,
        viewport: &mut Viewport,
        text_color: Color,
        interp_success: bool
    ) -> (usize, Vec<u8>) {
        let dimension: usize = 128;

        // Draw the Glyph name
        let mut paint = Paint::new(Color4f::new(1., 1., 1., 1.), None);
        paint.set_color(text_color);
        let typeface: skia_safe::RCHandle<skia_bindings::SkTypeface> = Typeface::default();
        let font = Font::new(typeface, 12.0); // Adjust the font size here
        let text_blob = TextBlob::new(&mfekglif.name, &font).unwrap();

        // Measure the text size to center it horizontally and vertically
        let text_bounds = font.measure_str(&mfekglif.name, None).1;
        let text_height = font.measure_str("|", None).1.height();
        let text_width = text_bounds.width();

        let dimension = dimension + text_height as usize;
        // Create a Surface with the desired width and height
        let mut surface =
            Surface::new_raster_n32_premul((dimension as i32, dimension as i32)).unwrap();

        // Get the Canvas from the Surface
        let canvas = surface.canvas();

        // Clear the canvas with a white background
        canvas.clear(Color4f::new(0., 0., 0., 0.));

        // Position and draw the text
        let text_position = Point::new(
            (dimension as f32 - text_width) / 2.0,
            dimension as f32 - text_height,
        );
        canvas.draw_text_blob(&text_blob, text_position, &paint);

        if !interp_success {
            let typeface: skia_safe::RCHandle<skia_bindings::SkTypeface> = Typeface::default();
            let font = Font::new(typeface, 24.0); // Adjust the font size here
            let text_blob = TextBlob::new("!", &font).unwrap();
            let text_position = Point::new(
                dimension as f32 - 24.,
                dimension as f32 - 24.
            );

            canvas.draw_text_blob(&text_blob, text_position, &paint);
        }

        // Draw the glyph
        let style = Style::new(Color::new(0xffffffff), text_color.into());
        viewport.redraw(canvas);
        glifrenderer::glyph::draw(canvas, mfekglif, viewport, Some(style));

        // Get the ImageInfo from the Surface
        let image_info = surface.image_info();

        // Create a buffer to store the image data
        let row_bytes = image_info.min_row_bytes();
        let size = row_bytes * (dimension);
        let mut image_data = vec![0u8; size];

        // Read the pixels from the Surface into the buffer
        let success = surface.read_pixels(&image_info, &mut image_data, row_bytes, (0, 0));
        assert!(success, "Failed to read pixels from the Surface");

        (dimension, image_data)
    }
}
