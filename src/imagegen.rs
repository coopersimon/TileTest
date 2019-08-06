// Generate a tile in an image.
/*use super::Vertex;

use vulkano::{
    device::Device,
    image::attachment::AttachmentImage,
    format::Format,
    sampler::{
        Filter,
        MipmapMode,
        Sampler,
        SamplerAddressMode
    }
};

use std::sync::Arc;*/


/*pub fn new_atlas(device: &Arc<Device>) -> (Arc<AttachmentImage>, Arc<Sampler>) {
    let atlas = AttachmentImage::sampled_input_attachment(
        device.clone(),
        [ATLAS_SIZE, ATLAS_SIZE],
        Format::R8Uint
    ).expect("Couldn't create atlas!");
    let sampler = Sampler::new(
        device.clone(),
        Filter::Nearest,
        Filter::Nearest,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        0.0, 1.0, 0.0, 0.0
    ).expect("Couldn't create sampler!");
    (atlas, sampler)
}*/

// Atlas of all tile textures. Must be square.
pub struct TextureAtlas {
    pub textures: Vec<u32>,
    atlas_size: usize,
    tex_size: usize,
    texel_size: usize
}

impl TextureAtlas {
    // Atlas size: size of the atlas in textures.
    // Tex size: size of a texture in texels.
    // Texel size: size of a texel in bits (keep this to a power of 2)
    pub fn new(atlas_size: usize, tex_size: usize, texel_size: usize) -> Self {
        assert!(texel_size & (texel_size - 1) == 0);    // Texel size must be a power of 2

        let width_texels = atlas_size * tex_size;       // Width and height must be the same.
        let area_texels = width_texels * width_texels;
        let area_bytes = (area_texels * texel_size) / 8;

        TextureAtlas {
            textures: vec![0; area_bytes / 4],
            atlas_size: atlas_size,
            tex_size: tex_size,
            texel_size: texel_size
        }
    }

    // Generate a new tile texture in the atlas.
    pub fn generate_tile_tex(&mut self, x: usize, y: usize) {
        let texels_per_word = 32 / self.texel_size;
        let base_x = x * self.tex_size;
        let base_y = y * self.tex_size;

        for y in base_y..(base_y + self.tex_size) {
            let y_offset = y * self.atlas_size * self.tex_size;
            let xy_offset = y_offset + base_x;
            // i is the texel number.
            for i in xy_offset..(xy_offset + self.tex_size) {
                assert!(i < (self.atlas_size * self.tex_size).pow(2));
                let word = i / texels_per_word;
                assert!(word < self.textures.len());
                let word_offset = i % texels_per_word;
                let mask = make_mask(word_offset, self.texel_size);
                // Mask the relevant bits.
                self.textures[word] &= !mask;
                self.textures[word] |= mask & rand::random::<u32>(); // rand
            }
        }
    }
}

// Helper fns:
fn make_mask(offset: usize, bit_length: usize) -> u32 {
    let mut mask = 0;
    for i in offset..(offset + bit_length) {
        mask |= 1 << i;
    }
    mask
}