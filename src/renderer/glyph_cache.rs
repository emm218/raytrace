use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::BuildHasherDefault;
use std::vec::Vec;

use fnv::FnvHasher;
use font_kit::{
    canvas::RasterizationOptions,
    error::{FontLoadingError, GlyphLoadingError, SelectionError},
    family_name::FamilyName,
    font::Font,
    hinting::HintingOptions,
    properties::Properties,
    source::SystemSource,
};
use pathfinder_geometry::{rect::RectI, transform2d::Transform2F};

use super::atlas::{Atlas, AtlasInsertError, GlyphTexInfo};

// #[derive(Debug)]
// pub enum GlyphCacheError {
//     InitError,
// FontError,
// }

// impl Error for GlyphCacheError {}

// impl fmt::Display for GlyphCacheError {
// fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     match &self {
//         InitError => write!(f, "Failed to initialize fontconfig")
//         FontError => write!(f, "Failed to load a font")
//     }
// }
// }

#[derive(Debug)]
pub enum GlyphCacheError {
    SelectionError(SelectionError),
    FontLoadingError(FontLoadingError),
}

impl Error for GlyphCacheError {}

impl fmt::Display for GlyphCacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            GlyphCacheError::SelectionError(e) => e.fmt(f),
            GlyphCacheError::FontLoadingError(e) => e.fmt(f),
        }
    }
}

impl From<SelectionError> for GlyphCacheError {
    fn from(err: SelectionError) -> Self {
        GlyphCacheError::SelectionError(err)
    }
}

impl From<FontLoadingError> for GlyphCacheError {
    fn from(err: FontLoadingError) -> Self {
        GlyphCacheError::FontLoadingError(err)
    }
}

pub struct GlyphCache {
    cache: HashMap<u32, GlyphTexInfo, BuildHasherDefault<FnvHasher>>,
    pub atlases: Vec<Atlas>,
    font: Font,
    font_size: f32,
}

impl GlyphCache {
    pub fn new(font_size: f32) -> Result<Self, GlyphCacheError> {
        let font_handle = SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())?;
        let font = font_handle.load()?;

        let atlases = vec![Atlas::new()];

        Ok(Self {
            cache: HashMap::default(),
            atlases,
            font,
            font_size,
        })
    }

    pub fn get(
        &mut self,
        glyph_id: u32,
    ) -> Result<GlyphTexInfo, GlyphLoadingError> {
        if let Some(glyph) = self.cache.get(&glyph_id) {
            return Ok(*glyph);
        }

        let bounds = self.font.raster_bounds(
            glyph_id,
            self.font_size,
            Default::default(),
            HintingOptions::None,
            RasterizationOptions::SubpixelAa,
        )?;

        let (transform, glyph) = self.load_glyph(&bounds);
        let font_size = self.font_size;
        let atlas = &mut self.atlases.last_mut().unwrap();
        let font = &mut self.font;

        // println!(
        //     "{:?} {:?} {:?}\n{:?}\n{:?}\n",
        //     bounds.upper_right(),
        //     bounds.lower_left(),
        //     bounds.lower_right(),
        //     transform,
        //     glyph
        // );

        font.rasterize_glyph(
            &mut atlas.canvas,
            glyph_id,
            font_size,
            transform,
            HintingOptions::None,
            RasterizationOptions::SubpixelAa,
        )?;

        Ok(*self.cache.entry(glyph_id).or_insert(glyph))
    }

    pub fn cache_common(&mut self) {
        for i in 32u8..=126u8 {
            println!("{}", i as char);
            let glyph_id = self.font.glyph_for_char(i as char).unwrap();
            self.get(glyph_id);
        }
    }

    fn load_glyph(
        &mut self,
        glyph_bounds: &RectI,
    ) -> (Transform2F, GlyphTexInfo) {
        let cur_atlas = self.current_atlas();
        match cur_atlas.insert(glyph_bounds) {
            Ok((transform, glyph)) => (transform, glyph),
            Err(AtlasInsertError::Full) => {
                // we're done with this atlas so upload anything pending to gpu
                unsafe {
                    cur_atlas.update_texture();
                }
                self.atlases.push(Atlas::new());
                self.load_glyph(glyph_bounds)
            }
            Err(AtlasInsertError::GlyphTooLarge) => panic!("glyph too large"),
        }
    }

    fn current_atlas(&mut self) -> &mut Atlas {
        // should never be able to not have an atlas
        self.atlases.last_mut().unwrap()
    }
}
