use std::{cell::RefCell, collections::HashMap, fs::read_dir, path::Path, sync::OnceLock};

use ab_glyph::{Font, FontArc, PxScale, ScaleFont, point};
use rand::{Rng, rngs::SmallRng};

static FONT_CACHE: OnceLock<FontCache> = OnceLock::new();
pub const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];

#[derive(Clone, Copy)]
pub struct DigitMetrics {
    pub bw: f32,
    // pub advance: f32,
}

#[derive(Clone, Copy)]
pub struct MetricsPack {
    pub height: f32,                // ascent - descent (в пикселях)
    pub digits: [DigitMetrics; 10], // для '0'..'9'
}

thread_local! {
    static GLYPH_CACHE: RefCell<HashMap<(String, u32), MetricsPack>> = RefCell::new(HashMap::new());
}
pub fn get_metrics_pack(font_name: &str, font: &FontArc, px: u32) -> MetricsPack {
    let key = (font_name.to_string(), px);
    if let Some(p) = GLYPH_CACHE.with(|c| c.borrow().get(&key).copied()) {
        return p;
    }

    let scale = PxScale {
        x: px as f32,
        y: px as f32,
    };
    let sf = font.as_scaled(scale); // scaled-шрифт
    let height = sf.height(); // уже в px, корректное значение

    let digits: [DigitMetrics; 10] = std::array::from_fn(|i| {
        let ch: char = DIGITS[i].chars().next().unwrap();
        let id = sf.glyph_id(ch);

        // ширина по контуру в px; fallback — advance
        let g = id.with_scale_and_position(scale, point(0.0, 0.0));
        let bw = font
            .outline_glyph(g)
            .map(|og| {
                let b = og.px_bounds();
                b.max.x - b.min.x
            })
            .unwrap_or_else(|| sf.h_advance(id));

        // let advance = sf.h_advance(id); // в px

        DigitMetrics {
            bw,
            // advance
        }
    });

    let pack = MetricsPack { height, digits };
    GLYPH_CACHE.with(|c| {
        c.borrow_mut().insert(key, pack);
    });
    pack
}

pub struct FontEntry {
    pub name: String, // имя файла как стабильный ключ
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
        {
            // пропускаем гарнитуры без цифр
            if ('0'..='9').all(|ch| f.glyph_id(ch).0 != 0) {
                let name = p.file_name().unwrap().to_string_lossy().into_owned();
                out.push(FontEntry { name, font: f });
            }
        }
    }
    if out.is_empty() {
        panic!("fonts not found");
    }
    out
}

// impl FontCache {
//     fn new() -> Self {
//         let mut fonts = Self::load_fonts();
//         fonts.sort_by_key(font_sort_key);
//         if fonts.is_empty() {
//             panic!("No fonts found in assets/fonts");
//         }
//         FontCache { fonts }
//     }

//     fn load_fonts() -> Vec<FontArc> {
//         let font_dir = Path::new("assets/fonts");

//         read_dir(font_dir)
//             .ok()
//             .into_iter()
//             .flat_map(|rd| rd.filter_map(|e| e.ok()))
//             .map(|e| e.path())
//             .filter(|p| {
//                 matches!(
//                     p.extension().and_then(|s| s.to_str()),
//                     Some("ttf") | Some("otf")
//                 )
//             })
//             .filter_map(|path| {
//                 std::fs::read(&path)
//                     .ok()
//                     .and_then(|bytes| FontArc::try_from_vec(bytes).ok())
//                     .filter(|f| ('0'..='9').all(|ch| f.glyph_id(ch).0 != 0))
//             })
//             .collect()
//     }

//     pub fn get_random<'a>(&'a self, rng: &mut SmallRng) -> &'a FontArc {
//         let idx = rng.random_range(0..self.fonts.len());
//         &self.fonts[idx]
//         // &self.fonts[0]
//     }

//     pub fn global() -> &'static FontCache {
//         FONT_CACHE.get_or_init(FontCache::new)
//     }
// }

// Вспомогательно: извлечь имена по face_index
// pub fn font_names(font: &FontArc, face_index: u32) -> (Option<String>, Option<String>) {
//     let mut family = None;
//     let mut full = None;
//     if let Ok(face) = Face::parse(font.font_data(), face_index) {
//         let names = face.names();
//         for i in 0..names.len() {
//             if let Some(n) = names.get(i) {
//                 match n.name_id {
//                     1 | 16 => {
//                         // Family / Typographic Family
//                         if family.is_none() {
//                             family = n
//                                 .to_string()
//                                 .or_else(|| Some(String::from_utf8_lossy(n.name).into_owned()));
//                         }
//                     }
//                     4 => {
//                         // Full name
//                         if full.is_none() {
//                             full = n
//                                 .to_string()
//                                 .or_else(|| Some(String::from_utf8_lossy(n.name).into_owned()));
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//         }
//     }
//     (family, full)
// }

// fn font_sort_key(font: &FontArc) -> String {
//     let face_index = 0;
//     let (family, full) = font_names(font, face_index);
//     let f1 = family.unwrap_or_default();
//     let f2 = full.unwrap_or_default();
//     format!("{}|{}", f1.to_lowercase(), f2.to_lowercase())
// }

// pub fn scan_all_fonts_debug(cache: &FontCache, scale: PxScale) {
//     for (idx, font) in cache.fonts.iter().enumerate() {
//         let faces = ttf_parser::fonts_in_collection(font.font_data()).unwrap_or(1);
//         let face_index: u32 = 0; // одиночные TTF/OTF → 0; для TTC подберите нужный индекс [21][2]
//         let (family, full) = font_names(font, face_index);
//         eprintln!(
//             "#{} faces={} face_index={} family={:?} full={:?}",
//             idx, faces, face_index, family, full
//         ); // имена через name::Name::to_string() внутри font_names [22][2]

//         let mut any_outline = false;
//         let mut any_svg = false;
//         let mut any_bitmap = false;

//         for ch in '0'..='9' {
//             let gid = font.glyph_id(ch);
//             let has_outline = font
//                 .outline_glyph(gid.with_scale_and_position(scale, point(0.0, 0.0)))
//                 .is_some();
//             let has_svg = font.glyph_svg_image(gid).is_some();
//             let has_bitmap = font.glyph_raster_image2(gid, u16::MAX).is_some();
//             any_outline |= has_outline;
//             any_svg |= has_svg;
//             any_bitmap |= has_bitmap;
//             eprintln!(
//                 "  '{}' U+{:04X} gid={:?} outline={} svg={} bitmap={}",
//                 ch, ch as u32, gid, has_outline, has_svg, has_bitmap
//             ); // статус глифа по трём источникам [22]
//         }

//         if !any_outline && !any_svg && !any_bitmap {
//             eprintln!(
//                 "  => UNSUPPORTED for digits: no outline/svg/bitmap, можно удалить этот файл"
//             ); // итог по шрифту [22]
//         } else if !any_outline && (any_svg || any_bitmap) {
//             eprintln!("  => COLOR/bitmap digits only: outline_glyph не сработает, лучше исключить"); // цветной/битмап [7][10]
//         } else {
//             eprintln!("  => OK: есть контуры для цифр, оставить"); // пригоден [2]
//         }
//     }
// }
