use winit::VirtualKeyCode;

// Command to alter visual data.
pub enum Command {
    ModifyTilePalette{
        palette: u32,
        x: usize,
        y: usize
    },
    ModifyTileTexture{
        tex_x: usize,
        tex_y: usize,
        x: usize,
        y: usize
    },
    GenerateTexture{
        tex_x: usize,
        tex_y: usize
    }
}

// State machine to track key input and react accordingly.
pub enum KeyState {
    Neutral,
    TileSelect(usize, usize),
    TexSelect(usize, usize)
}

impl KeyState {
    pub fn new() -> Self {
        KeyState::Neutral
    }

    pub fn process_key(&mut self, k: VirtualKeyCode) -> (KeyState, Option<Command>) {
        use self::KeyState::*;
        use winit::VirtualKeyCode::*;

        match self {
            Neutral => match k {
                Key1 => (TileSelect(0, 0), None),
                Key2 => (TileSelect(1, 0), None),
                Key3 => (TileSelect(2, 0), None),
                Key4 => (TileSelect(3, 0), None),
                Q => (TileSelect(0, 1), None),
                W => (TileSelect(1, 1), None),
                E => (TileSelect(2, 1), None),
                R => (TileSelect(3, 1), None),
                A => (TileSelect(0, 2), None),
                S => (TileSelect(1, 2), None),
                D => (TileSelect(2, 2), None),
                F => (TileSelect(3, 2), None),
                Z => (TileSelect(0, 3), None),
                X => (TileSelect(1, 3), None),
                C => (TileSelect(2, 3), None),
                V => (TileSelect(3, 3), None),
                G => (TexSelect(0, 0), None),
                H => (TexSelect(1, 0), None),
                J => (TexSelect(0, 1), None),
                K => (TexSelect(1, 1), None),
                _ => (Neutral, None)
            },
            TileSelect(x, y) => match k {
                T => (Neutral, Some(Command::ModifyTilePalette{palette: 0, x: *x, y: *y})),
                Y => (Neutral, Some(Command::ModifyTilePalette{palette: 1, x: *x, y: *y})),
                U => (Neutral, Some(Command::ModifyTilePalette{palette: 2, x: *x, y: *y})),
                I => (Neutral, Some(Command::ModifyTilePalette{palette: 3, x: *x, y: *y})),
                G => (Neutral, Some(Command::ModifyTileTexture{tex_x: 0, tex_y: 0, x: *x, y: *y})),
                H => (Neutral, Some(Command::ModifyTileTexture{tex_x: 1, tex_y: 0, x: *x, y: *y})),
                J => (Neutral, Some(Command::ModifyTileTexture{tex_x: 0, tex_y: 1, x: *x, y: *y})),
                K => (Neutral, Some(Command::ModifyTileTexture{tex_x: 1, tex_y: 1, x: *x, y: *y})),
                _ => (Neutral, None)
            },
            TexSelect(x, y) => match k {
                Return => (Neutral, Some(Command::GenerateTexture{tex_x: *x, tex_y: *y})),
                _ => (Neutral, None)
            }
        }
    }
}