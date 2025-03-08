use crate::{
    image::{pixels_to_tiles, tiles_to_pixels, TILE_LENGTH},
    ntr::{NtrFile, NtrFormat},
    read_write_ext::ReadExt,
    FileFormat, Image, NtrTextureFormat,
};

struct ScreenEntry {
    tile_index: usize,
    h_flip: bool,
    v_flip: bool,
    palette_index: usize,
}

impl From<u16> for ScreenEntry {
    fn from(value: u16) -> Self {
        Self {
            tile_index: ((value >> 0) & ((1 << 0xa) - 1)).try_into().unwrap(),
            h_flip: ((value >> 0xa) & 1) != 0,
            v_flip: ((value >> 0xb) & 1) != 0,
            palette_index: ((value >> 0xc) & ((1 << 4) - 1)).try_into().unwrap(),
        }
    }
}

pub struct Nscr {
    width_in_tiles: usize,
    texture_format: NtrTextureFormat,
    screen_entries: Vec<ScreenEntry>,
}

impl NtrFormat for Nscr {
    fn read_from_ntr_file(file: &NtrFile) -> std::io::Result<Self> {
        assert!(file.id() == "RCSN");

        assert!(file.blocks().len() == 1);

        let scrn_block = &file.blocks()[0];
        assert!(scrn_block.id() == "NRCS");
        let mut scrn = scrn_block.contents();

        let width: usize = scrn.read_u16()?.try_into().unwrap();
        assert!(width % TILE_LENGTH == 0);
        let height: usize = scrn.read_u16()?.try_into().unwrap();
        assert!(height % TILE_LENGTH == 0);
        let texture_format = match scrn.read_u16()? {
            0 => NtrTextureFormat::Palette16,
            1 | 2 => NtrTextureFormat::Palette256,
            _ => panic!(),
        };
        let bg_type = scrn.read_u16()?;

        let screen_size = scrn.read_u32()?.try_into().unwrap();
        match bg_type {
            0 | 2 => assert!(screen_size * TILE_LENGTH * TILE_LENGTH / 2 == width * height),
            1 => assert!(screen_size * TILE_LENGTH * TILE_LENGTH == width * height),
            _ => panic!(),
        }

        let raw_data = scrn.read_sized(screen_size)?;
        let screen_entries = Self::read_screen_data(&raw_data, bg_type);

        Ok(Self {
            width_in_tiles: width / TILE_LENGTH,
            texture_format,
            screen_entries,
        })
    }

    fn write_to_ntr_file(&self) -> std::io::Result<NtrFile> {
        todo!()
    }
}

impl Nscr {
    fn read_screen_data(raw_data: &Vec<u8>, bg_type: u16) -> Vec<ScreenEntry> {
        match bg_type {
            0 | 2 => raw_data
                .chunks(2)
                .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()).into())
                .collect(),
            1 => raw_data
                .iter()
                .map(|tile| ScreenEntry {
                    tile_index: (*tile).try_into().unwrap(),
                    h_flip: false,
                    v_flip: false,
                    palette_index: 0,
                })
                .collect(),
            _ => panic!(),
        }
    }

    pub fn to_image(&self, tileset: &Image) -> Image {
        let palette = tileset.palette().unwrap().clone();
        let tiles = {
            let tiles = pixels_to_tiles(tileset.pixels(), tileset.width_in_tiles());
            assert!(palette.colors().len() >= min_colors_in_palette(&self.screen_entries, &tiles));
            match self.texture_format {
                NtrTextureFormat::Palette16 => tiles
                    .iter()
                    .map(|tile| tile.map(|pixel| pixel % 16))
                    .collect::<Vec<_>>(),
                NtrTextureFormat::Palette256 => {
                    assert!(self.screen_entries.iter().all(|e| e.palette_index == 0));
                    tiles
                }
                _ => panic!(),
            }
        };

        let arrangement = self
            .screen_entries
            .iter()
            .map(|entry| {
                let mut tile = tiles[entry.tile_index];
                if entry.h_flip {
                    flip_tile_horizontal(&mut tile);
                }
                if entry.v_flip {
                    flip_tile_vertical(&mut tile);
                }
                tile
            })
            .collect::<Vec<_>>();

        let pixels = tiles_to_pixels(&arrangement, self.width_in_tiles);

        Image::new(self.width_in_tiles * TILE_LENGTH, &pixels, Some(palette))
    }
}

impl FileFormat for Nscr {
    fn extension() -> String {
        "NSCR".to_string()
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        NtrFormat::read_from_data(data)
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        NtrFormat::write_to_data(self)
    }
}

fn flip_tile_horizontal(tile: &mut [u8; TILE_LENGTH * TILE_LENGTH]) {
    for y in 0..TILE_LENGTH {
        for x in 0..(TILE_LENGTH / 2) {
            let left = y * TILE_LENGTH + x;
            let right = y * TILE_LENGTH + TILE_LENGTH - x - 1;
            tile.swap(left, right);
        }
    }
}

fn flip_tile_vertical(tile: &mut [u8; TILE_LENGTH * TILE_LENGTH]) {
    for y in 0..(TILE_LENGTH / 2) {
        for x in 0..TILE_LENGTH {
            let top = y * TILE_LENGTH + x;
            let bottom = (TILE_LENGTH - y - 1) * TILE_LENGTH + x;
            tile.swap(top, bottom);
        }
    }
}

fn min_colors_in_palette(
    screen_entries: &[ScreenEntry],
    tiles: &[[u8; TILE_LENGTH * TILE_LENGTH]],
) -> usize {
    let biggest_palette_index = screen_entries
        .iter()
        .map(|entry| entry.palette_index)
        .max()
        .unwrap();
    usize::from(
        screen_entries
            .iter()
            .filter(|entry| entry.palette_index == biggest_palette_index)
            .map(|entry| tiles[entry.tile_index])
            .flatten()
            .max()
            .unwrap(),
    ) + biggest_palette_index * 16
}
