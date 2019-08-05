// Generate a tile in an image.
use super::Vertex;

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

use std::sync::Arc;

const TILE_SIZE: usize = 8;
const ATLAS_SIZE: usize = 16;

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

// Generate a new tile on the CPU
pub fn generate_tile_tex(atlas: &mut [u8], x: usize, y: usize) {
    let base_x = x * TILE_SIZE;
    let base_y = y * TILE_SIZE;

    for y in base_y..(base_y + TILE_SIZE) {
        let y_offset = y * ATLAS_SIZE;
        let xy_offset = y_offset + base_x;
        for i in xy_offset..(xy_offset + TILE_SIZE) {
            atlas[i] = 0; // rand
        }
    }
}

// Generate a list of vertices for a specified grid size.
pub fn generate_vertices(x_size: usize, y_size: usize) -> Vec<Vertex> {
    let mut vertices = Vec::new();

    let x_frac = 2.0 / x_size as f32;
    let y_frac = 2.0 / y_size as f32;
    let mut lo_y = -1.0;
    let mut hi_y = y_frac;

    for y in 0..y_size {
        let mut lo_x = -1.0;
        let mut hi_x = x_frac;
        for x in 0..x_size {
            vertices.push(Vertex{ position: [lo_x, lo_y], tex_coord: [0.0, 0.0] });
            vertices.push(Vertex{ position: [lo_x, hi_y], tex_coord: [0.0, 1.0] });
            vertices.push(Vertex{ position: [hi_x, lo_y], tex_coord: [1.0, 0.0] });
            vertices.push(Vertex{ position: [lo_x, hi_y], tex_coord: [0.0, 1.0] });
            vertices.push(Vertex{ position: [hi_x, lo_y], tex_coord: [1.0, 0.0] });
            vertices.push(Vertex{ position: [hi_x, hi_y], tex_coord: [1.0, 1.0] });

            lo_x = hi_x;
            hi_x += x_frac;
        }
        lo_y = hi_y;
        hi_y += y_frac;
    }

    vertices
}