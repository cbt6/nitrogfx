#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl From<u16> for Color {
    fn from(value: u16) -> Self {
        let red = ((value >> 0 & 0x1F) as u8) * 8;
        let green = ((value >> 5 & 0x1F) as u8) * 8;
        let blue = ((value >> 10 & 0x1F) as u8) * 8;
        Self { red, green, blue }
    }
}

impl Into<u16> for Color {
    fn into(self) -> u16 {
        let r = ((self.red / 8) as u16 & 0x1F) << 0;
        let g = ((self.green / 8) as u16 & 0x1F) << 5;
        let b = ((self.blue / 8) as u16 & 0x1F) << 10;
        r + g + b
    }
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Color { red, green, blue }
    }
}

#[derive(Clone)]
pub struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    pub fn new(colors: Vec<Color>) -> Self {
        Palette { colors }
    }

    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    pub(crate) fn gen_16_colors() -> Self {
        let mut colors = vec![];
        for i in (0..=u8::MAX).step_by(0x11) {
            colors.push(Color::new(i, i, i));
        }
        Self { colors }
    }

    pub(crate) fn gen_256_colors() -> Self {
        let mut colors = vec![];
        for i in 0..=u8::MAX {
            colors.push(Color::new(i, i, i));
        }
        Self { colors }
    }
}

impl std::fmt::Debug for Palette {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in &self.colors {
            writeln!(f, "{:?}", c)?;
        }
        Ok(())
    }
}
