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

const TILE_SIZE: usize = 8;     // In pixels
const ATLAS_SIZE: usize = 2;    // In tiles

pub struct VertexGrid {
    pub vertices: Vec<Vertex>,
    x_size: usize,
    y_size: usize
}

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

// Sets the tex coords for a tile.
pub fn set_tile(vertex_grid: &mut VertexGrid, tile_x: usize, tile_y: usize, tex_x: usize, tex_y: usize) {
    let y_offset = tile_y * vertex_grid.y_size * 6;
    let index = y_offset + (tile_x * 6);

    let atlas_size = ATLAS_SIZE as f32;
    let top_left = (tex_x as f32 / atlas_size, tex_y as f32 / atlas_size);
    let bottom_right = (top_left.0 + 1.0 / atlas_size, top_left.1 + 1.0 / atlas_size);

    vertex_grid.vertices[index].tex_coord = [top_left.0, top_left.1];
    vertex_grid.vertices[index + 1].tex_coord = [top_left.0, bottom_right.1];
    vertex_grid.vertices[index + 2].tex_coord = [bottom_right.0, top_left.1];
    vertex_grid.vertices[index + 3].tex_coord = [top_left.0, bottom_right.1];
    vertex_grid.vertices[index + 4].tex_coord = [bottom_right.0, top_left.1];
    vertex_grid.vertices[index + 5].tex_coord = [bottom_right.0, bottom_right.1];
}

// Generate a list of vertices for a specified grid size.
pub fn generate_vertices(x_size: usize, y_size: usize) -> VertexGrid {
    let mut grid = VertexGrid {
        vertices: Vec::new(),
        x_size: x_size,
        y_size: y_size
    };

    let x_frac = 2.0 / x_size as f32;
    let y_frac = 2.0 / y_size as f32;
    let mut lo_y = -1.0;
    let mut hi_y = lo_y + y_frac;

    for _ in 0..y_size {
        let mut lo_x = -1.0;
        let mut hi_x = lo_x + x_frac;
        for _ in 0..x_size {
            grid.vertices.push(Vertex{ position: [lo_x, lo_y], tex_coord: [0.0, 0.0] });
            grid.vertices.push(Vertex{ position: [lo_x, hi_y], tex_coord: [0.0, 1.0] });
            grid.vertices.push(Vertex{ position: [hi_x, lo_y], tex_coord: [1.0, 0.0] });
            grid.vertices.push(Vertex{ position: [lo_x, hi_y], tex_coord: [0.0, 1.0] });
            grid.vertices.push(Vertex{ position: [hi_x, lo_y], tex_coord: [1.0, 0.0] });
            grid.vertices.push(Vertex{ position: [hi_x, hi_y], tex_coord: [1.0, 1.0] });

            lo_x = hi_x;
            hi_x += x_frac;
        }
        lo_y = hi_y;
        hi_y += y_frac;
    }

    grid
}