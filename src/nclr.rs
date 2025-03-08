use crate::{
    enums::{NtrFileVersion, NtrTextureFormat},
    ntr::{NtrFile, NtrFileBlock, NtrFormat, NtrMetadata},
    palette::Palette,
    read_write_ext::{ReadExt, WriteExt},
    FileFormat,
};

#[derive(Debug, Default, Clone)]
pub struct NclrMetadata {
    pub version: NtrFileVersion,
    pub texture_format: NtrTextureFormat,

    /// The value to write to offset 0x0002 of the PLTT block. Defaults to `0`.
    pub pltt_0002: u16,

    /// Whether the extended palette is used. Defaults to false.
    pub extended: bool,

    /// Whether the size of data is stored as `(0x200 - size)` instead.
    /// Defaults to false.
    pub invert_size: bool,

    /// Whether the unused high bit for each color is set. Defaults to false.
    pub high_color_bit: bool,

    /// The palette indexes stored in the PCMP block. If empty, no PCMP block
    /// is included. Defaults to an empty vector.
    pub palette_indexes: Vec<u16>,
}

impl Into<NtrMetadata> for NclrMetadata {
    fn into(self) -> NtrMetadata {
        NtrMetadata {
            version: self.version,
        }
    }
}

impl NclrMetadata {
    pub fn with_version(self, version: NtrFileVersion) -> Self {
        Self { version, ..self }
    }

    pub fn with_texture_format(self, texture_format: NtrTextureFormat) -> Self {
        Self {
            texture_format,
            ..self
        }
    }

    pub fn pltt_0002(self, value: u16) -> Self {
        Self {
            pltt_0002: value,
            ..self
        }
    }

    pub fn extended(self, extended: bool) -> Self {
        Self { extended, ..self }
    }

    pub fn invert_size(self, invert_size: bool) -> Self {
        Self {
            invert_size,
            ..self
        }
    }

    pub fn high_color_bit(self, high_color_bit: bool) -> Self {
        Self {
            high_color_bit,
            ..self
        }
    }

    pub fn with_palette_indexes(self, palette_indexes: Vec<u16>) -> Self {
        Self {
            palette_indexes,
            ..self
        }
    }
}

pub struct Nclr {
    metadata: NclrMetadata,
    palette: Palette,
}

impl NtrFormat for Nclr {
    fn read_from_ntr_file(file: &NtrFile) -> std::io::Result<Self> {
        assert!(file.id() == "RLCN");

        let pltt_block = &file.blocks()[0];
        assert!(pltt_block.id() == "TTLP");
        let mut pltt = pltt_block.contents();
        let texture_format = pltt.read_u16()?.into();
        match texture_format {
            NtrTextureFormat::Palette16 | NtrTextureFormat::Palette256 => {}
            _ => panic!(),
        };
        let pltt_0002 = pltt.read_u16()?;
        let extended = match pltt.read_u32()? {
            0 => false,
            1 => true,
            _ => panic!(),
        };
        let palette_size = u32::try_from(pltt_block.contents().len() - 16).unwrap();
        let read_palette_size = pltt.read_u32()?;
        let invert_size = if read_palette_size == palette_size {
            false
        } else if read_palette_size == 0x200 - palette_size {
            true
        } else {
            panic!();
        };
        let palette_offset = pltt.read_u32()?;
        assert!(palette_offset == 0x00000010);

        let mut colors = vec![];
        let mut high_color_bit = false;
        let num_colors = palette_size / 2; // each color takes up 2 bytes
        for _ in 0..num_colors {
            let value = pltt.read_u16()?;
            high_color_bit |= (value >> 0xf) != 0;
            colors.push(value.into());
        }

        let palette_indexes = if file.blocks().len() > 1 {
            let pcmp_block = &file.blocks()[1];
            assert!(pcmp_block.id() == "PMCP");
            let mut pcmp = pcmp_block.contents();
            let num_palette_indexes = pcmp.read_u16()?;
            assert!(pcmp.read_u16()? == 0xBEEF);
            assert!(pcmp.read_u32()? == 0x00000008);
            let mut palette_indexes = vec![];
            for _ in 0..num_palette_indexes {
                palette_indexes.push(pcmp.read_u16()?);
            }
            palette_indexes
        } else {
            vec![]
        };

        let metadata = NclrMetadata {
            version: file.version(),
            texture_format,
            pltt_0002,
            extended,
            invert_size,
            high_color_bit,
            palette_indexes,
        };

        Ok(Self {
            metadata,
            palette: Palette::new(colors),
        })
    }

    fn write_to_ntr_file(&self) -> std::io::Result<NtrFile> {
        let mut blocks = vec![self.to_pltt_block(&self.metadata)?];
        if !self.metadata.palette_indexes.is_empty() {
            blocks.push(self.to_pcmp_block(&self.metadata.palette_indexes)?);
        }

        Ok(NtrFile::new("RLCN", self.metadata.version, blocks))
    }
}

impl Nclr {
    pub fn from_palette(palette: Palette, metadata: NclrMetadata) -> Self {
        Self { metadata, palette }
    }

    pub fn to_palette(&self) -> Palette {
        self.palette.clone()
    }

    pub fn metadata(&self) -> NclrMetadata {
        self.metadata.clone()
    }

    fn to_pltt_block(&self, metadata: &NclrMetadata) -> std::io::Result<NtrFileBlock> {
        let mut pltt = vec![];
        pltt.write_u16(metadata.texture_format.into())?;
        pltt.write_u16(metadata.pltt_0002)?;
        pltt.write_u32(if metadata.extended { 1 } else { 0 })?;

        let mut data: Vec<u16> = vec![];
        for color in self.palette.colors() {
            let value: u16 = (*color).into();
            data.push(value | (if metadata.high_color_bit { 1 << 0xf } else { 0 }));
        }

        let data_size = (data.len() * 2).try_into().unwrap();
        pltt.write_u32(if metadata.invert_size {
            0x200 - data_size
        } else {
            data_size
        })?;
        pltt.write_u32(0x00000010)?;
        for color in data {
            pltt.write_u16(color)?;
        }

        Ok(NtrFileBlock::new("TTLP", pltt))
    }

    fn to_pcmp_block(&self, palette_indexes: &[u16]) -> std::io::Result<NtrFileBlock> {
        let mut pcmp = vec![];
        pcmp.write_u16(palette_indexes.len().try_into().unwrap())?;
        pcmp.write_u16(0xBEEF)?;
        pcmp.write_u32(0x00000008)?;

        for palette_index in palette_indexes {
            pcmp.write_u16(*palette_index)?;
        }

        Ok(NtrFileBlock::new("PMCP", pcmp))
    }
}

impl FileFormat for Nclr {
    fn extension() -> String {
        "NCLR".to_string()
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        NtrFormat::read_from_data(data)
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        NtrFormat::write_to_data(self)
    }
}
