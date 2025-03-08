use crate::{
    format::FileFormat,
    palette::{Color, Palette},
};

pub struct Jasc {
    palette: Palette,
}

impl Jasc {
    pub fn from_palette(palette: Palette) -> Self {
        Jasc { palette }
    }

    pub fn to_palette(self) -> Palette {
        self.palette
    }
}

impl FileFormat for Jasc {
    fn extension() -> String {
        "pal".to_string()
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        let mut colors = vec![];
        let s = std::str::from_utf8(data).unwrap();
        let mut lines = s.lines();

        assert!(lines.next().unwrap() == "JASC-PAL");
        assert!(lines.next().unwrap() == "0100");
        let num_colors = lines.next().unwrap().parse::<usize>().unwrap();
        for _ in 0..num_colors {
            let mut components = lines.next().unwrap().split_ascii_whitespace();
            let red = components.next().unwrap().parse::<u8>().unwrap();
            let green = components.next().unwrap().parse::<u8>().unwrap();
            let blue = components.next().unwrap().parse::<u8>().unwrap();
            colors.push(Color::new(red, green, blue));
            assert!(components.next().is_none());
        }
        assert!(lines.next().is_none());

        Ok(Jasc {
            palette: Palette::new(colors),
        })
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        let mut lines = vec![];
        lines.push("JASC-PAL".to_string());
        lines.push("0100".to_string());
        lines.push(self.palette.colors().len().to_string());
        for color in self.palette.colors() {
            lines.push(format!("{} {} {}", color.red, color.green, color.blue));
        }
        lines.push("".to_string());

        Ok(lines.join("\r\n").as_bytes().to_vec())
    }
}
