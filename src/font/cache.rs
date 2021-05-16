use fontdue::{Font, layout::{GlyphPosition, GlyphRasterConfig}};
use std::collections::HashMap;
use rect_packer::{Config, Packer, Rect};

struct UsizeRect {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
}
impl From<Rect> for UsizeRect {
    fn from(rect: Rect) -> Self {
        Self {
            width: rect.width as usize,
            height: rect.height as usize,
            x: rect.x as usize,
            y: rect.y as usize,
        }
    }
}

#[derive(Debug)]
pub struct UVRect {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
}
impl UVRect {
    fn pixel_to_uv(rect: &UsizeRect, width: usize, height: usize) -> Self {
        let width = width as f32;
        let height = height as f32;
        Self {
            width: rect.width as f32 / width,
            height: rect.height as f32 / height,
            x: rect.x as f32 / width,
            y: rect.y as f32 / height,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum AtlasFormat {
    Greyscale,
    Subpixel,
}
impl AtlasFormat {
    fn bpp(&self) -> usize {
        match self {
            AtlasFormat::Greyscale => 1,
            AtlasFormat::Subpixel => 3,
        }
    }
}

pub struct GlyphCache {
    width: usize,
    height: usize,
    format: AtlasFormat,
    image: Vec<u8>,
    packer: Packer,
    map: HashMap<GlyphRasterConfig, UsizeRect>
}

impl GlyphCache {
    pub fn new(width: usize, height: usize, format: AtlasFormat) -> Self {
        let mut image = Vec::with_capacity(width * height * format.bpp());
        unsafe { image.set_len(width * height * format.bpp()) };
        
        Self {
            width, height, format, image,
            packer: Packer::new(Config {
                width: width as i32,
                height: height as i32,
                border_padding: 1,
                rectangle_padding: 1,
            }),
            map: HashMap::new(),
        }
    }
    
    pub fn get_image(&self) -> &[u8] {
        self.image.as_slice()
    }
    
    pub fn get_uv(&self, g: &GlyphRasterConfig) -> Option<UVRect> {
        self.map.get(g).map(|rect| UVRect::pixel_to_uv(rect, self.width, self.height))
    }
    
    pub fn cache<U: Copy>(&mut self, fonts: &[Font], glyphs: &[GlyphPosition<U>]) {
        for glyph in glyphs {
            if !self.map.contains_key(&glyph.key) && !glyph.key.c.is_whitespace() {
                // TODO: expiriment with rotated characters
                if let Some(rect) = self.packer.pack(glyph.width as i32, glyph.height as i32, false) {
                    let rect = UsizeRect::from(rect);
                    
                    let index = glyph.key.font_index;
                    let (_, bitmap) = match self.format {
                        AtlasFormat::Greyscale => fonts[index].rasterize_config(glyph.key),
                        AtlasFormat::Subpixel => fonts[index].rasterize_config_subpixel(glyph.key)
                    };
                    
                    let bpp = self.format.bpp();
                    for y in 0..rect.height {
                        let dest_offset = ((y + rect.y) * self.width + rect.x) * bpp;
                        let dest = &mut self.image[dest_offset..dest_offset + (rect.width * bpp)];
                        
                        let src_offset = (y * rect.width) * bpp;
                        let src = &bitmap[src_offset..src_offset + (rect.width * bpp)];
                        
                        dest.copy_from_slice(src);
                    }
                    
                    self.map.insert(glyph.key, rect);
                }
            }
        }
        // use std::fs::File;
        // use std::io::Write;
        // let mut o = File::create("atlas.ppm").unwrap();
        // let _ = o.write(format!("P6\n{} {}\n255\n", self.width, self.height).as_bytes());
        // let _ = o.write(&self.image);
    }
}
