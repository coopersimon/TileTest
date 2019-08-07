use super::Vertex;

// Struct that contains the vertices to be used for rendering.
pub struct VertexGrid {
    pub vertices: Vec<Vertex>,
    row_len: usize,
    atlas_size: f32
}

impl VertexGrid {
    // Make a new 2D Vertex Grid of size (x_size * y_size). Also input atlas size (square).
    pub fn new(x_size: usize, y_size: usize, atlas_size: usize) -> Self {
        let mut grid = VertexGrid {
            vertices: Vec::new(),
            row_len: x_size,
            atlas_size: atlas_size as f32
        };

        if (x_size > 0) && (y_size > 0) {
            let x_frac = 2.0 / x_size as f32;
            let y_frac = 2.0 / y_size as f32;
            let mut lo_y = -1.0;
            let mut hi_y = lo_y + y_frac;

            for _ in 0..y_size {
                let mut lo_x = -1.0;
                let mut hi_x = lo_x + x_frac;
                for _ in 0..x_size {
                    grid.vertices.push(Vertex{ position: [lo_x, lo_y], tex_coord: [0.0, 0.0], palette_index: 0 });
                    grid.vertices.push(Vertex{ position: [lo_x, hi_y], tex_coord: [0.0, 1.0], palette_index: 0 });
                    grid.vertices.push(Vertex{ position: [hi_x, lo_y], tex_coord: [1.0, 0.0], palette_index: 0 });
                    grid.vertices.push(Vertex{ position: [lo_x, hi_y], tex_coord: [0.0, 1.0], palette_index: 0 });
                    grid.vertices.push(Vertex{ position: [hi_x, lo_y], tex_coord: [1.0, 0.0], palette_index: 0 });
                    grid.vertices.push(Vertex{ position: [hi_x, hi_y], tex_coord: [1.0, 1.0], palette_index: 0 });

                    lo_x = hi_x;
                    hi_x += x_frac;
                }
                lo_y = hi_y;
                hi_y += y_frac;
            }
        }

        grid
    }

    // Sets the tex coords for a tile.
    pub fn set_tile_texture(&mut self, tile_x: usize, tile_y: usize, tex_x: usize, tex_y: usize) {
        let y_offset = tile_y * self.row_len * 6;
        let index = y_offset + (tile_x * 6);

        let top_left = (tex_x as f32 / self.atlas_size, tex_y as f32 / self.atlas_size);
        let bottom_right = (top_left.0 + 1.0 / self.atlas_size, top_left.1 + 1.0 / self.atlas_size);

        self.vertices[index].tex_coord =        [top_left.0, top_left.1];
        self.vertices[index + 1].tex_coord =    [top_left.0, bottom_right.1];
        self.vertices[index + 2].tex_coord =    [bottom_right.0, top_left.1];
        self.vertices[index + 3].tex_coord =    [top_left.0, bottom_right.1];
        self.vertices[index + 4].tex_coord =    [bottom_right.0, top_left.1];
        self.vertices[index + 5].tex_coord =    [bottom_right.0, bottom_right.1];
    }

    // Sets the palette for a tile.
    pub fn set_tile_palette(&mut self, tile_x: usize, tile_y: usize, palette_index: u32) {
        let y_offset = tile_y * self.row_len * 6;
        let index = y_offset + (tile_x * 6);

        self.vertices[index].palette_index = palette_index;
        self.vertices[index + 1].palette_index = palette_index;
        self.vertices[index + 2].palette_index = palette_index;
        self.vertices[index + 3].palette_index = palette_index;
        self.vertices[index + 4].palette_index = palette_index;
        self.vertices[index + 5].palette_index = palette_index;
    }

    pub fn get_vertices<'a>(&'a self) -> &'a [Vertex] {
        &self.vertices
    }
}