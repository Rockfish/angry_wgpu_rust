use glam::Vec3;
use spark_gap::texture::Texture;

#[derive(Debug)]
pub struct SpriteSheet {
    pub texture: Texture,
    pub num_columns: i32,
    pub time_per_sprite: f32,
}

impl SpriteSheet {
    pub const fn new(texture_unit: Texture, num_columns: i32, time_per_sprite: f32) -> Self {
        Self {
            texture: texture_unit,
            num_columns,
            time_per_sprite,
        }
    }
}

pub struct SpriteSheetSprite {
    pub world_position: Vec3,
    pub age: f32,
}

impl SpriteSheetSprite {
    pub const fn new(world_position: Vec3) -> Self {
        Self { world_position, age: 0.0 }
    }
}
