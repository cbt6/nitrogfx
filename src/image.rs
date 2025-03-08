use crate::palette::Palette;

pub(crate) const TILE_LENGTH: usize = 8;

type Tile = [u8; TILE_LENGTH * TILE_LENGTH];

#[derive(Clone)]
pub struct Image {
    /// Width of image in pixels.
    width: usize,

    pixels: Vec<u8>,
    palette: Option<Palette>,
}

pub fn pixels_to_tiles(pixels: &[u8], width_in_tiles: usize) -> Vec<Tile> {
    assert!(pixels.len() % (TILE_LENGTH * TILE_LENGTH) == 0);
    let num_tiles = pixels.len() / (TILE_LENGTH * TILE_LENGTH);
    assert!(num_tiles % width_in_tiles == 0);
    let mut tiles = vec![];
    for row_of_pixels in pixels.chunks(TILE_LENGTH * TILE_LENGTH * width_in_tiles) {
        for tile_num in 0..width_in_tiles {
            let mut tile = [0u8; TILE_LENGTH * TILE_LENGTH];
            for y in 0..TILE_LENGTH {
                for x in 0..TILE_LENGTH {
                    let dst_index = y * TILE_LENGTH + x;
                    let src_index = tile_num * 8 + y * 8 * width_in_tiles + x;
                    tile[dst_index] = row_of_pixels[src_index];
                }
            }
            tiles.push(tile);
        }
    }
    tiles
}

pub fn tiles_to_pixels(tiles: &[Tile], width_in_tiles: usize) -> Vec<u8> {
    assert!(tiles.len() % width_in_tiles == 0);
    let mut pixels = vec![];
    for row_of_tiles in tiles.chunks(width_in_tiles) {
        for y in 0..TILE_LENGTH {
            for tile in row_of_tiles {
                for x in 0..TILE_LENGTH {
                    pixels.push(tile[y * TILE_LENGTH + x]);
                }
            }
        }
    }
    pixels
}

impl Image {
    pub fn new(width: usize, pixels: &[u8], palette: Option<Palette>) -> Self {
        assert!(pixels.len() % width == 0);
        Self {
            width,
            pixels: pixels.to_vec(),
            palette,
        }
    }

    pub fn with_palette(self, palette: Palette) -> Self {
        Self {
            palette: Some(palette),
            ..self
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.pixels.len() / self.width
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn palette(&self) -> Option<Palette> {
        self.palette.clone()
    }

    pub fn crop(&self, top: usize, left: usize, bottom: usize, right: usize) -> Image {
        assert!(left < right && right < self.width());
        assert!(top < bottom && bottom < self.height());

        let mut new_pixels = vec![];
        for (i, pixel) in self.pixels.iter().enumerate() {
            let x = i % self.width();
            let y = i / self.width();
            if (left <= x && x <= right) && (top <= y && y <= bottom) {
                new_pixels.push(*pixel);
            }
        }

        Self {
            width: right - left + 1,
            pixels: new_pixels,
            palette: self.palette.clone(),
        }
    }

    pub(crate) fn width_in_tiles(&self) -> usize {
        assert!(self.width % TILE_LENGTH == 0);
        self.width / TILE_LENGTH
    }

    pub(crate) fn height_in_tiles(&self) -> usize {
        assert!(self.height() % TILE_LENGTH == 0);
        self.height() / TILE_LENGTH
    }

    pub(crate) fn raw_data_4bpp_to_pixels(raw_data: &[u8]) -> Vec<u8> {
        let mut pixels = vec![];
        for byte in raw_data {
            pixels.push(byte & 0xF);
            pixels.push(byte >> 4);
        }
        pixels
    }

    pub(crate) fn raw_data_8bpp_to_pixels(raw_data: &[u8]) -> Vec<u8> {
        raw_data.to_vec()
    }
}
