// Generate a tile in an image.
use vulkano::{
    command_buffer::{
        AutoCommandBuffer,
        CommandBufferExecFuture,
    },
    device::Queue,
    image::{
        Dimensions,
        immutable::ImmutableImage
    },
    format::{
        R8Uint
    },
    sync::NowFuture
};

use std::sync::Arc;

// Atlas of all tile textures. Must be square.
pub struct TextureAtlas {
    pub textures: Vec<u8>,
    atlas_size: usize,
    tex_size: usize
}

impl TextureAtlas {
    // Atlas size: size of the atlas in textures.
    // Tex size: size of a texture in texels.
    pub fn new(atlas_size: usize, tex_size: usize) -> Self {
        let width = atlas_size * tex_size;       // Width and height must be the same.
        let area = width * width;

        TextureAtlas {
            textures: vec![0; area],
            atlas_size: atlas_size,
            tex_size: tex_size
        }
    }

    // Generate a new tile texture in the atlas.
    pub fn generate_tile_tex(&mut self, x: usize, y: usize) {
        let base_x = x * self.tex_size;
        let base_y = y * self.tex_size;

        for y in base_y..(base_y + self.tex_size) {
            let y_offset = y * self.atlas_size * self.tex_size;
            let xy_offset = y_offset + base_x;
            // i is the texel number.
            for i in xy_offset..(xy_offset + self.tex_size) {
                self.textures[i] = rand::random::<u8>() & 0b11;
            }
        }
    }

    // Make an image from the atlas.
    pub fn make_image(&self, queue: Arc<Queue>) -> (Arc<ImmutableImage<R8Uint>>, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
        let width = (self.atlas_size * self.tex_size) as u32;
        ImmutableImage::from_iter(
            self.textures.clone().into_iter(),
            Dimensions::Dim2d { width: width, height: width },
            R8Uint,
            queue
        ).expect("Couldn't create image.")
    }
}