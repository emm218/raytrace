use std::cmp::{max, min};
use std::ptr;

use font_kit::canvas::{Canvas, Format};
use gl::types::*;
use pathfinder_geometry::{
    rect::RectI,
    transform2d::Transform2F,
    vector::{Vector2F, Vector2I},
};

pub const ATLAS_SIZE: i32 = 1024;

pub enum AtlasInsertError {
    // the texture atlas is full
    Full,

    // the glyph cannot fit within a single texture
    GlyphTooLarge,
}

#[derive(Debug)]
pub struct Atlas {
    pub tex_id: GLuint,
    pub canvas: Canvas,

    // leftmost free x coordinate
    insert_x: i32,
    // topmost free y coordinate
    insert_y: i32,

    // portion of atlas not yet uploaded to GPU
    dirty_y: i32,
    dirty_height: i32,

    // height of tallest glyph in current row to use for advancement
    row_tallest: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct GlyphTexInfo {
    pub tex_id: GLuint,
    pub uv_left: f32,
    pub uv_bot: f32,
    pub uv_width: f32,
    pub uv_height: f32,
}

impl Atlas {
    pub fn new() -> Self {
        let mut tex_id: GLuint = 0;
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::GenTextures(1, &mut tex_id);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                ATLAS_SIZE,
                ATLAS_SIZE,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as i32,
            );

            // unbind texture
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        let canvas = Canvas::new(Vector2I::splat(ATLAS_SIZE), Format::Rgba32);

        Self {
            tex_id,
            canvas,
            insert_x: 0,
            insert_y: 0,
            dirty_y: 0,
            dirty_height: 0,
            row_tallest: 0,
        }
    }

    pub fn insert(
        &mut self,
        glyph_bounds: &RectI,
    ) -> Result<(Transform2F, GlyphTexInfo), AtlasInsertError> {
        let glyph_width = glyph_bounds.width();
        let glyph_height = glyph_bounds.height();

        // this should never happen
        if glyph_width > ATLAS_SIZE || glyph_height > ATLAS_SIZE {
            return Err(AtlasInsertError::GlyphTooLarge);
        }

        if self.insert_x + glyph_width > ATLAS_SIZE {
            self.advance_row()?;
        }

        if self.insert_y + glyph_height > ATLAS_SIZE {
            return Err(AtlasInsertError::Full);
        }

        let lower_left = glyph_bounds.lower_left();
        let transform = Transform2F::from_translation(Vector2F::new(
            (self.insert_x - lower_left.x()) as f32,
            (self.insert_y - lower_left.y()) as f32,
        ));

        let uv_left = self.to_uv(self.insert_x);
        let uv_bot = self.to_uv(self.insert_y);
        let uv_width = self.to_uv(glyph_width);
        let uv_height = self.to_uv(glyph_height);

        self.insert_x += glyph_width;
        self.row_tallest = max(glyph_height, self.row_tallest);
        self.dirty_y = min(self.dirty_y, self.insert_y);
        if self.dirty_y + self.dirty_height < self.insert_y + glyph_height {
            self.dirty_height = self.insert_y + glyph_height - self.dirty_y;
        }

        Ok((
            transform,
            GlyphTexInfo {
                tex_id: self.tex_id,
                uv_left,
                uv_bot,
                uv_width,
                uv_height,
            },
        ))
    }

    fn advance_row(&mut self) -> Result<(), AtlasInsertError> {
        let new_y = self.insert_y + self.row_tallest;
        if new_y > ATLAS_SIZE {
            return Err(AtlasInsertError::Full);
        }

        self.insert_y = new_y;
        self.insert_x = 0;
        self.row_tallest = 0;

        Ok(())
    }

    // helper to convert pixel values to UV values
    fn to_uv(&self, num: i32) -> f32 {
        num as f32 / ATLAS_SIZE as f32
    }

    pub unsafe fn update_texture(&mut self) {
        if self.dirty_height == 0 {
            return;
        }
        let pixels = &self.canvas.pixels;
        let stride = self.canvas.stride;
        let start = stride * self.dirty_y as usize;
        let end = start + stride * self.dirty_height as usize;
        let pixels_slice = &pixels[start..end];

        gl::BindTexture(gl::TEXTURE_2D, self.tex_id);

        gl::TexSubImage2D(
            gl::TEXTURE_2D,
            0,
            0,
            self.dirty_y,
            ATLAS_SIZE,
            self.dirty_height,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels_slice.as_ptr() as *const _,
        );

        // unbind texture
        gl::BindTexture(gl::TEXTURE_2D, 0);

        self.dirty_y = ATLAS_SIZE + 1;
        self.dirty_height = 0;
    }
}

impl Drop for Atlas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.tex_id);
        }
    }
}
