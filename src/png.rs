use png::{Info, Reader};

use crate::{
    format::FileFormat,
    image::Image,
    palette::{Color, Palette},
};

pub struct Png {
    image: Image,
}

impl FileFormat for Png {
    fn extension() -> String {
        "png".to_string()
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        let decoder = png::Decoder::new(data);
        let mut reader = decoder.read_info()?;
        let image = Self::read_image(&mut reader);
        Ok(Self { image })
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        let width = self.image.width().try_into().unwrap();
        let height = self.image.height().try_into().unwrap();

        let palette = self.image.palette().unwrap_or_else(|| {
            match self.image.pixels().iter().max().unwrap() {
                0..16 => Palette::gen_16_colors(),
                16..=255 => Palette::gen_256_colors(),
            }
        });
        assert!(palette.colors().len() <= 256);

        let mut data = vec![];
        let buf_writer = std::io::BufWriter::new(&mut data);
        let mut encoder = png::Encoder::new(buf_writer, width, height);
        encoder.set_color(png::ColorType::Indexed);
        let bit_depth = match palette.colors().len() {
            0..=8 => unimplemented!(),
            9..=16 => png::BitDepth::Four,
            17..=256 => png::BitDepth::Eight,
            _ => unreachable!(),
        };
        encoder.set_depth(bit_depth);
        encoder.set_palette(Self::write_palette(&palette));
        let mut writer = encoder.write_header()?;
        let pixels = match bit_depth {
            png::BitDepth::Four => &self
                .image
                .pixels()
                .chunks(2)
                .map(|chunk| chunk[0] << 4 | chunk[1])
                .collect::<Vec<u8>>(),
            png::BitDepth::Eight => self.image.pixels(),
            _ => unimplemented!(),
        };

        writer.write_image_data(pixels).unwrap();
        writer.finish().unwrap();
        Ok(data)
    }
}

impl Png {
    pub fn from_image(image: Image) -> Self {
        let image = match image.palette() {
            Some(palette) => {
                if palette.colors().len() > 256 {
                    eprintln!("truncating palette to first 256 colors");
                    image.with_palette(Palette::new(palette.colors()[0..256].to_vec()))
                } else {
                    image
                }
            }
            None => image,
        };
        Self { image }
    }

    pub fn to_image(&self) -> Image {
        self.image.clone()
    }

    fn read_image(reader: &mut Reader<&[u8]>) -> Image {
        let (color_type, bit_depth) = reader.output_color_type();
        assert!(matches!(color_type, png::ColorType::Indexed));
        let info = reader.info();
        let width: usize = info.width.try_into().unwrap();
        let palette = Self::read_palette(info);

        let mut buf = vec![0; reader.output_buffer_size()];
        let frame_info = reader.next_frame(&mut buf).unwrap();
        let bytes = &buf[..frame_info.buffer_size()];

        let pixels = match bit_depth {
            png::BitDepth::Four => {
                let mut pixels = vec![];
                for byte in bytes {
                    pixels.push(byte >> 4);
                    pixels.push(byte & 0xF);
                }
                pixels
            }
            png::BitDepth::Eight => bytes.to_vec(),
            _ => unimplemented!(),
        };

        assert!(pixels.len() % width == 0);

        Image::new(width, &pixels, Some(palette))
    }

    fn read_palette(info: &Info) -> Palette {
        let mut colors = vec![];
        let raw_palette = info.palette.clone().unwrap();
        assert!(raw_palette.len() % 3 == 0);
        for chunk in raw_palette.chunks(3) {
            let red = chunk[0];
            let green = chunk[1];
            let blue = chunk[2];
            colors.push(Color::new(red, green, blue));
        }
        Palette::new(colors)
    }

    fn write_palette(palette: &Palette) -> Vec<u8> {
        let mut out = vec![];
        for color in palette.colors() {
            out.push(color.red);
            out.push(color.green);
            out.push(color.blue);
        }
        out
    }
}
