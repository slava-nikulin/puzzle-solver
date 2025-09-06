use std::{
    collections::HashMap,
    fs::read_dir,
    path::Path,
    sync::{OnceLock, RwLock},
};

use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use rand::{Rng, rngs::SmallRng};

static FONT_CACHE: OnceLock<FontCache> = OnceLock::new();
pub const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

#[derive(Clone, Copy)]
pub struct DigitMetrics {
    pub bw: f32,
}

#[derive(Clone, Copy)]
pub struct MetricsPack {
    pub height: f32,
    pub digits: [DigitMetrics; 10],
}

static GLYPH_CACHE: OnceLock<RwLock<HashMap<(String, u32), MetricsPack>>> = OnceLock::new();

fn glyph_cache() -> &'static RwLock<HashMap<(String, u32), MetricsPack>> {
    GLYPH_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn get_metrics_pack(font_name: &str, font: &FontArc, px: u32) -> MetricsPack {
    let key = (font_name.to_string(), px);
    if let Some(p) = glyph_cache().read().unwrap().get(&key).cloned() {
        return p;
    }

    let scale = PxScale {
        x: px as f32,
        y: px as f32,
    };
    let sf = font.as_scaled(scale);
    let height = sf.height();

    let digits: [DigitMetrics; 10] = std::array::from_fn(|i| {
        let ch: char = DIGITS[i].chars().next().unwrap();
        let id = sf.glyph_id(ch);

        let g = id.with_scale_and_position(scale, point(0.0, 0.0));
        let bw = font
            .outline_glyph(g)
            .map(|og| {
                let b = og.px_bounds();
                b.max.x - b.min.x
            })
            .unwrap_or_else(|| sf.h_advance(id));

        DigitMetrics { bw }
    });

    let pack = MetricsPack { height, digits };
    glyph_cache().write().unwrap().insert(key, pack);
    pack
}

pub struct FontEntry {
    pub name: String,
    pub font: FontArc,
}

pub struct FontCache {
    fonts: Vec<FontEntry>,
}

impl FontCache {
    pub fn global() -> &'static FontCache {
        FONT_CACHE.get_or_init(|| FontCache {
            fonts: load_fonts(),
        })
    }

    pub fn get_random_named<'a>(&'a self, rng: &mut SmallRng) -> (&'a FontArc, &'a str) {
        let i = rng.random_range(0..self.fonts.len());
        let e = &self.fonts[i];
        (&e.font, &e.name)
    }
}

fn load_fonts() -> Vec<FontEntry> {
    let font_dir = Path::new("assets/fonts");
    let mut paths: Vec<_> = read_dir(font_dir)
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
        .collect();
    paths.sort();

    let mut out = Vec::new();
    for p in paths {
        if let Ok(bytes) = std::fs::read(&p)
            && let Ok(f) = FontArc::try_from_vec(bytes)
            && ('0'..='9').all(|ch| f.glyph_id(ch).0 != 0)
        {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            out.push(FontEntry { name, font: f });
        }
    }
    if out.is_empty() {
        panic!("fonts not found");
    }
    out
}
