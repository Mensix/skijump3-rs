use crate::color::Rgba;
use crate::video::{Renderer, TextureId};
use std::cell::RefCell;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaletteIndex(pub u8);

impl PaletteIndex {
    pub const TRANSPARENT: Self = Self(0);

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteMaterialId(u64);

impl SpriteMaterialId {
    pub const DEFAULT: Self = Self(0);

    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl SpriteMaterial {
    pub fn id(&self) -> SpriteMaterialId {
        self.id
    }

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

    pub fn color_override(&self, source: PaletteIndex) -> Option<Rgba> {
        for &(from, to) in &self.overrides {
            if from == source {
                return Some(to);
            }
        }
        None
    }

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
    materials: RefCell<HashMap<BakedSpriteTextureKey, BakedSpriteTexture>>,
    base_sprites: Vec<BaseSprite>,
}

impl BakedSpriteTextures {
    pub fn new() -> Self {
        Self {
            defaults: Vec::new(),
            materials: RefCell::new(HashMap::new()),
            base_sprites: Vec::new(),
        }
    }

    pub fn add_default(&mut self, idx: u16, texture: BakedSpriteTexture) {
        while self.defaults.len() <= idx as usize {
            self.defaults.push(None);
        }
        self.defaults[idx as usize] = Some(texture);
    }

    pub fn ensure_material(
        &self,
        sprite_idx: u16,
        material: &SpriteMaterial,
        renderer: &mut Renderer,
    ) {
        let key = BakedSpriteTextureKey {
            sprite_idx,
            material: material.clone(),
        };
        if self.materials.borrow().contains_key(&key) {
            return;
        }
        let Some(base) = self
            .base_sprites
            .iter()
            .find(|b| b.sprite_idx == sprite_idx)
        else {
            return;
        };
        let mut resolved = base.palette;
        for &(source_idx, override_color) in material.overrides() {
            resolved[source_idx.value() as usize] = override_color;
        }
        let mut scratch = Vec::new();
        indices_to_rgba(&base.indices, &resolved, &mut scratch);
        if let Ok(texture_id) = renderer.create_rgba_texture(&scratch, base.width, base.height) {
            self.materials.borrow_mut().insert(
                key,
                BakedSpriteTexture {
                    texture_id,
                    center_x: base.center_x,
                    center_y: base.center_y,
                    width: base.width as u16,
                    height: base.height as u16,
                },
            );
        }
    }

    pub fn default_sprite(&self, sprite_idx: u16) -> Option<&BakedSpriteTexture> {
        self.defaults
            .get(sprite_idx as usize)
            .and_then(std::option::Option::as_ref)
    }

    pub fn material_sprite(
        &self,
        sprite_idx: u16,
        material: &SpriteMaterial,
    ) -> Option<BakedSpriteTexture> {
        self.materials
            .borrow()
            .get(&BakedSpriteTextureKey {
                sprite_idx,
                material: material.clone(),
            })
            .cloned()
    }

    pub fn bake_with_png(
        renderer: &mut Renderer,
        base_sprites: &[BaseSprite],
        material_variants: &[(u16, SpriteMaterial)],
    ) -> Result<Self, String> {
        let mut defaults = vec![None; base_sprites.len()];
        let mut materials = HashMap::new();

        let mut scratch = Vec::new();
        for base in base_sprites {
            indices_to_rgba(&base.indices, &base.palette, &mut scratch);
            let texture_id = renderer.create_rgba_texture(&scratch, base.width, base.height)?;
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

        scratch.clear();
        for (sprite_idx, material) in material_variants {
            let Some(base) = base_sprites.iter().find(|b| b.sprite_idx == *sprite_idx) else {
                continue;
            };

            let mut resolved = base.palette;
            for &(source_idx, override_color) in material.overrides() {
                resolved[source_idx.value() as usize] = override_color;
            }

            indices_to_rgba(&base.indices, &resolved, &mut scratch);
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
            materials: RefCell::new(materials),
            base_sprites: base_sprites.to_vec(),
        })
    }
}

fn indices_to_rgba(indices: &[u8], palette: &[Rgba; 256], out: &mut Vec<u8>) {
    out.clear();
    out.reserve(indices.len() * 4);
    for &idx in indices {
        let c = palette[idx as usize];
        out.push(c.r);
        out.push(c.g);
        out.push(c.b);
        out.push(c.a);
    }
}

#[derive(Debug, Clone)]
pub struct BaseSprite {
    pub sprite_idx: u16,
    pub indices: Vec<u8>,
    pub palette: [Rgba; 256],
    pub width: u32,
    pub height: u32,
    pub center_x: i8,
    pub center_y: i8,
}

impl Default for SpriteMaterial {
    fn default() -> Self {
        Self {
            id: SpriteMaterialId::DEFAULT,
            overrides: Box::new([]),
        }
    }
}
