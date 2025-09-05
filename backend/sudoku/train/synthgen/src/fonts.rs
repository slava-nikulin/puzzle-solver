use std::{fs::read_dir, path::Path, sync::OnceLock};

use ab_glyph::{Font, FontArc};
use rand::{Rng, rngs::SmallRng};

static FONT_CACHE: OnceLock<FontCache> = OnceLock::new();
pub const DIGITS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

pub struct FontCache {
    fonts: Vec<FontArc>,
}

impl FontCache {
    fn new() -> Self {
        let fonts = Self::load_fonts();
        // детерминированный порядок
        // fonts.sort_by(|a, b| a.as_font().full_name().cmp(&b.as_font().full_name()));
        if fonts.is_empty() {
            panic!("No fonts found in assets/fonts");
        }
        FontCache { fonts }
    }

    fn load_fonts() -> Vec<FontArc> {
        let font_dir = Path::new("assets/fonts");

        read_dir(font_dir)
            .ok()
            .into_iter()
            .flat_map(|rd| rd.filter_map(|e| e.ok()))
            .map(|e| e.path())
            .filter(|p| {
                matches!(
                    p.extension().and_then(|s| s.to_str()),
                    Some("ttf") | Some("otf")
                )
            })
            .filter_map(|path| {
                std::fs::read(&path)
                    .ok()
                    .and_then(|bytes| FontArc::try_from_vec(bytes).ok())
                    .filter(|f| ('0'..='9').all(|ch| f.glyph_id(ch).0 != 0))
            })
            .collect()
    }

    pub fn get_random<'a>(&'a self, rng: &mut SmallRng) -> &'a FontArc {
        // &self.fonts[rng.random_range(0..self.fonts.len())]
        &self.fonts[0]
    }

    pub fn global() -> &'static FontCache {
        FONT_CACHE.get_or_init(FontCache::new)
    }
}
