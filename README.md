# Tile Test
This project is a demonstration of how tile and palette based graphics can be emulated in a modern GPU, using Vulkan.

Practical applications include retro video game emulation, and font rendering systems.

### The demo
The demo involves a grid of 4x4 tiles (each 8x8 pixels) displayed on the screen. Each rendered tile takes a tile texture and applies a palette to it. The textures are stored in a single texture atlas. The atlas is 16x16 pixels and therefore can store 4 tile textures.

Each texture is generated randomly when the program starts. More can be generated at runtime as described below.

4 palettes are also hard-coded.

### How to use
Run with `cargo run`.

Each tile can be selected with a key corresponding to the place on the grid:

```
1 2 3 4
q w e r
a s d f
z x c v
```

After selecting a tile, the palette can be chosen with one of `t, y, u, i`. Or, the tile texture can be swapped out with `g, h, j, k`.

To generate a new tile texture for the corresponding slot, first type the texture you want to swap out (`g, h, j, k`), then press `enter` to generate a new texture and replace the old one.