use crate::bitmap::IndexedBitmap;
use crate::color::Rgba;
use crate::consts::{HEIGHT, WIDTH};
use crate::video::{Renderer, TextureId};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaletteIndex(pub u8);

impl PaletteIndex {
    pub const TRANSPARENT: Self = Self(0);

    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct SpriteData {
    pub pixels: Box<[u8]>,
    pub width: u16,
    pub height: u16,
    pub center_x: i8,
    pub center_y: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteMaterialId(u64);

impl SpriteMaterialId {
    pub const DEFAULT: Self = Self(0);

    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl SpriteMaterial {
    #[must_use]
    pub fn id(&self) -> SpriteMaterialId {
        self.id
    }

    #[must_use]
    pub fn overrides(&self) -> &[(PaletteIndex, Rgba)] {
        &self.overrides
    }
}

#[derive(Debug, Clone)]
pub struct SpriteMaterial {
    id: SpriteMaterialId,
    overrides: Box<[(PaletteIndex, Rgba)]>,
}

impl PartialEq for SpriteMaterial {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SpriteMaterial {}

impl Hash for SpriteMaterial {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl SpriteMaterial {
    #[must_use]
    pub fn new(overrides: &[(u8, Rgba)]) -> Self {
        let mut overrides: Vec<_> = overrides
            .iter()
            .map(|&(from, to)| (PaletteIndex(from), to))
            .collect();
        overrides.sort_by_key(|(from, _)| from.value());
        let id = Self::id_for_overrides(&overrides);
        Self {
            id,
            overrides: overrides.into_boxed_slice(),
        }
    }

    #[must_use]
    pub fn with_id(id: SpriteMaterialId, overrides: &[(u8, Rgba)]) -> Self {
        let mut overrides: Vec<_> = overrides
            .iter()
            .map(|&(from, to)| (PaletteIndex(from), to))
            .collect();
        overrides.sort_by_key(|(from, _)| from.value());
        Self {
            id,
            overrides: overrides.into_boxed_slice(),
        }
    }

    fn id_for_overrides(overrides: &[(PaletteIndex, Rgba)]) -> SpriteMaterialId {
        let mut hasher = DefaultHasher::new();
        overrides.hash(&mut hasher);
        SpriteMaterialId(hasher.finish().max(1))
    }

    #[must_use]
    pub fn color_override(&self, source: PaletteIndex) -> Option<Rgba> {
        for &(from, to) in &self.overrides {
            if from == source {
                return Some(to);
            }
        }
        None
    }

    #[must_use]
    pub fn get(&self, source: u8) -> Option<Rgba> {
        self.color_override(PaletteIndex(source))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BakedSpriteTextureKey {
    sprite_idx: u16,
    material: SpriteMaterial,
}

#[derive(Debug, Clone)]
pub struct BakedSpriteTexture {
    pub texture_id: TextureId,
    pub center_x: i8,
    pub center_y: i8,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Default)]
pub struct BakedSpriteTextures {
    defaults: Vec<Option<BakedSpriteTexture>>,
    materials: HashMap<BakedSpriteTextureKey, BakedSpriteTexture>,
}

impl BakedSpriteTextures {
    #[must_use]
    pub fn new() -> Self {
        Self {
            defaults: Vec::new(),
            materials: HashMap::new(),
        }
    }

    pub fn add_default(&mut self, idx: u16, texture: BakedSpriteTexture) {
        while self.defaults.len() <= idx as usize {
            self.defaults.push(None);
        }
        self.defaults[idx as usize] = Some(texture);
    }

    pub fn add_material(
        &mut self,
        sprite_idx: u16,
        material: SpriteMaterial,
        texture: BakedSpriteTexture,
    ) {
        self.materials.insert(
            BakedSpriteTextureKey {
                sprite_idx,
                material,
            },
            texture,
        );
    }

    #[must_use]
    pub fn default_sprite(&self, sprite_idx: u16) -> Option<&BakedSpriteTexture> {
        self.defaults
            .get(sprite_idx as usize)
            .and_then(std::option::Option::as_ref)
    }

    #[must_use]
    pub fn material_sprite(
        &self,
        sprite_idx: u16,
        material: &SpriteMaterial,
    ) -> Option<&BakedSpriteTexture> {
        self.materials.get(&BakedSpriteTextureKey {
            sprite_idx,
            material: material.clone(),
        })
    }

    /// Bake material variants from base RGBA sprites by matching source
    /// pixel colors and replacing with override colors.
    ///
    /// `base_sprites` maps sprite_idx → (rgba_bytes, width, height, center_x, center_y).
    /// `source_colors` maps palette_idx → the original RGBA value of that
    ///   palette entry (from the source palette). Used to identify which
    ///   pixels in the base sprite need recoloring.
    /// `material_variants` is the list of (sprite_idx, material) pairs to bake.
    pub fn bake_with_png(
        renderer: &mut Renderer,
        base_sprites: &[BaseSprite],
        source_colors: &SourceColorMap,
        material_variants: &[(u16, SpriteMaterial)],
    ) -> Result<Self, String> {
        let mut defaults = vec![None; base_sprites.len()];
        let mut materials = HashMap::new();

        for base in base_sprites {
            let texture_id = renderer.create_rgba_texture(&base.rgba, base.width, base.height)?;
            let idx = base.sprite_idx as usize;
            while defaults.len() <= idx {
                defaults.push(None);
            }
            defaults[idx] = Some(BakedSpriteTexture {
                texture_id,
                center_x: base.center_x,
                center_y: base.center_y,
                width: base.width as u16,
                height: base.height as u16,
            });
        }

        let mut scratch = Vec::new();
        for (sprite_idx, material) in material_variants {
            let Some(base) = base_sprites.iter().find(|b| b.sprite_idx == *sprite_idx) else {
                continue;
            };
            let len = base.rgba.len();
            scratch.clear();
            scratch.reserve(len);
            scratch.extend_from_slice(&base.rgba);

            for &(source_idx, override_color) in material.overrides() {
                let Some(&orig_color) = source_colors.map.get(&source_idx.value()) else {
                    continue;
                };
                let orig = [orig_color.r, orig_color.g, orig_color.b, orig_color.a];
                let repl = [
                    override_color.r,
                    override_color.g,
                    override_color.b,
                    override_color.a,
                ];
                for chunk in scratch.chunks_exact_mut(4) {
                    if chunk == orig {
                        chunk.copy_from_slice(&repl);
                    }
                }
            }

            let texture_id = renderer.create_rgba_texture(&scratch, base.width, base.height)?;
            materials.insert(
                BakedSpriteTextureKey {
                    sprite_idx: *sprite_idx,
                    material: material.clone(),
                },
                BakedSpriteTexture {
                    texture_id,
                    center_x: base.center_x,
                    center_y: base.center_y,
                    width: base.width as u16,
                    height: base.height as u16,
                },
            );
        }

        Ok(Self {
            defaults,
            materials,
        })
    }
}

/// Pre-loaded base sprite RGBA data for `BakedSpriteTextures::bake_with_png`.
#[derive(Debug, Clone)]
pub struct BaseSprite {
    pub sprite_idx: u16,
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub center_x: i8,
    pub center_y: i8,
}

/// Maps palette index → its original RGBA color (from the source palette).
/// Used by bake to identify pixels that need recoloring.
#[derive(Debug, Clone)]
pub struct SourceColorMap {
    map: HashMap<u8, Rgba>,
}

impl SourceColorMap {
    pub fn from_entries(entries: &[(u8, Rgba)]) -> Self {
        Self {
            map: entries.iter().copied().collect(),
        }
    }
}

impl Default for SpriteMaterial {
    fn default() -> Self {
        Self {
            id: SpriteMaterialId::DEFAULT,
            overrides: Box::new([]),
        }
    }
}

impl SpriteData {
    /// Render sprite to a minimal indexed bitmap suitable for GPU upload.
    /// Returns `None` when the sprite is fully off-screen or has no
    /// non-zero visible pixels.
    #[must_use]
    pub fn render_bitmap(&self, dst_x: i32, dst_y: i32) -> Option<IndexedBitmap> {
        let start_x = dst_x - i32::from(self.center_x);
        let start_y = dst_y - i32::from(self.center_y);

        let vis_left = start_x.max(0);
        let vis_top = start_y.max(0);
        let vis_right = (start_x + i32::from(self.width)).min(WIDTH as i32);
        let vis_bottom = (start_y + i32::from(self.height)).min(HEIGHT as i32);
        let vis_w = (vis_right - vis_left).max(0) as u32;
        let vis_h = (vis_bottom - vis_top).max(0) as u32;

        if vis_w == 0 || vis_h == 0 {
            return None;
        }

        let mut pixels = vec![0u8; (vis_w * vis_h) as usize];
        let mut has_opaque_pixel = false;

        for src_y in 0..i32::from(self.height) {
            let screen_y = start_y + src_y;
            if screen_y < vis_top || screen_y >= vis_bottom {
                continue;
            }
            for src_x in 0..i32::from(self.width) {
                let screen_x = start_x + src_x;
                if screen_x < vis_left || screen_x >= vis_right {
                    continue;
                }
                let src_idx = src_y as usize * self.width as usize + src_x as usize;
                if src_idx >= self.pixels.len() {
                    continue;
                }
                let pixel = self.pixels[src_idx];
                if pixel == 0 {
                    continue;
                }
                has_opaque_pixel = true;
                let dx = (screen_x - vis_left) as u32;
                let dy = (screen_y - vis_top) as u32;
                pixels[(dy * vis_w + dx) as usize] = pixel;
            }
        }

        if !has_opaque_pixel {
            return None;
        }

        Some(IndexedBitmap {
            pixels,
            x: vis_left,
            y: vis_top,
            width: vis_w,
            height: vis_h,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::WIDTH;

    // ---------------------------------------------------------------------------
    // IndexedBitmap rendering tests
    // ---------------------------------------------------------------------------

    #[test]
    fn render_bitmap_basic() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        assert_eq!(bm.width, 2);
        assert_eq!(bm.height, 2);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.y, 0);
        assert_eq!(bm.pixels, vec![1, 2, 3, 4]);
    }

    #[test]
    fn render_bitmap_skips_transparent() {
        let sprite = SpriteData {
            pixels: vec![0, 2, 3, 0].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        // transparent pixels should remain 0
        assert_eq!(bm.pixels[0], 0, "top-left pixel was 0");
        assert_eq!(bm.pixels[3], 0, "bottom-right pixel was 0");
        // opaque pixels preserved
        assert_eq!(bm.pixels[1], 2);
        assert_eq!(bm.pixels[2], 3);
    }

    #[test]
    fn render_bitmap_clips_right_edge() {
        let sprite = SpriteData {
            pixels: vec![1, 2].into_boxed_slice(),
            width: 2,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        // Place so only 1 pixel visible at the right edge.
        // First sprite pixel at x=WIDTH-1 is visible, second at x=WIDTH is clipped.
        let bm = sprite.render_bitmap((WIDTH - 1) as i32, 0).unwrap();
        assert_eq!(bm.width, 1, "only leftmost pixel visible");
        assert_eq!(bm.pixels[0], 1);
    }

    #[test]
    fn render_bitmap_clips_bottom_edge() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        // Place at y=HEIGHT-1 so only top row visible (bottom row clipped)
        let bm = sprite.render_bitmap(0, (HEIGHT - 1) as i32).unwrap();
        assert_eq!(bm.height, 1, "only top row visible");
        assert_eq!(bm.pixels[0], 1);
        assert_eq!(bm.pixels[1], 2);
    }

    #[test]
    fn render_bitmap_clips_left_negative() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        // Place at x=-1 so only right column visible
        let bm = sprite.render_bitmap(-1, 0).unwrap();
        assert_eq!(bm.width, 1, "only right column visible");
        assert_eq!(bm.y, 0);
        assert_eq!(bm.pixels[0], 2, "(1,0) was pixel index 2");
        assert_eq!(bm.pixels[1], 4, "(1,1) was pixel index 4");
    }

    #[test]
    fn render_bitmap_clips_top_negative() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        // Place at y=-1 so only bottom row visible
        let bm = sprite.render_bitmap(0, -1).unwrap();
        assert_eq!(bm.height, 1, "only bottom row visible");
        assert_eq!(bm.pixels[0], 3, "(0,1) was pixel index 3");
        assert_eq!(bm.pixels[1], 4, "(1,1) was pixel index 4");
    }

    #[test]
    fn render_bitmap_fully_offscreen_right_returns_none() {
        let sprite = SpriteData {
            pixels: vec![1].into_boxed_slice(),
            width: 1,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        assert!(sprite.render_bitmap(WIDTH as i32 + 100, 0).is_none());
    }

    #[test]
    fn render_bitmap_center_offset() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 1,
            center_y: 1,
        };
        // With center=(1,1), the anchor (dst_x, dst_y) is at the center.
        // start_x = dst_x - center_x = 10 - 1 = 9
        // start_y = dst_y - center_y = 20 - 1 = 19
        let bm = sprite.render_bitmap(10, 20).unwrap();
        assert_eq!(bm.x, 9);
        assert_eq!(bm.y, 19);
        assert_eq!(bm.width, 2);
        assert_eq!(bm.height, 2);
    }

    #[test]
    fn render_bitmap_all_transparent_returns_none() {
        let sprite = SpriteData {
            pixels: vec![0, 0, 0, 0].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        assert!(sprite.render_bitmap(0, 0).is_none());
    }

    #[test]
    fn render_bitmap_preserves_original_values_no_remap() {
        let sprite = SpriteData {
            pixels: vec![5, 10, 15, 20].into_boxed_slice(),
            width: 2,
            height: 2,
            center_x: 0,
            center_y: 0,
        };
        let bm = sprite.render_bitmap(0, 0).unwrap();
        assert_eq!(bm.pixels, vec![5, 10, 15, 20]);
    }

    #[test]
    fn render_bitmap_partially_offscreen_left() {
        let sprite = SpriteData {
            pixels: vec![1, 2, 3, 4].into_boxed_slice(),
            width: 3,
            height: 1,
            center_x: 0,
            center_y: 0,
        };
        // Place at x=-2 so only 1 pixel visible
        let bm = sprite.render_bitmap(-2, 0).unwrap();
        assert_eq!(bm.width, 1);
        assert_eq!(bm.x, 0);
        assert_eq!(bm.pixels[0], 3, "third pixel (index 2) is visible");
    }
}
