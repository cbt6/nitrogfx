use std::{io::Write, vec};

use crate::{
    enums::{NtrCharacterFormat, NtrFileVersion, NtrMappingType, NtrTextureFormat},
    format::FileFormat,
    image::{pixels_to_tiles, tiles_to_pixels, Image, TILE_LENGTH},
    ntr::{NtrFile, NtrFileBlock, NtrFormat, NtrMetadata},
    read_write_ext::{ReadExt, WriteExt},
};

type Tile = [u8; TILE_LENGTH * TILE_LENGTH];

#[derive(Debug)]
enum Mapping1DVariant {
    Vram32,
    Vram64,
    Vram128,
    Vram256,
}

#[derive(Debug)]
enum MappingData {
    TwoD((usize, usize)),
    OneD(Mapping1DVariant),
}

#[derive(Debug)]
enum CharacterData {
    Character(Vec<Tile>, u32),
    Bitmap(Vec<u8>),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NcgrMetadata {
    pub version: NtrFileVersion,
    pub texture_format: NtrTextureFormat,
    pub mapping_type: NtrMappingType,
    pub character_format: NtrCharacterFormat,

    /// Whether the CPOS block is included. Defaults to false.
    pub include_cpos: bool,
}

impl Into<NtrMetadata> for NcgrMetadata {
    fn into(self) -> NtrMetadata {
        NtrMetadata {
            version: self.version,
        }
    }
}

impl NcgrMetadata {
    pub fn with_version(self, version: NtrFileVersion) -> Self {
        Self { version, ..self }
    }

    pub fn with_texture_format(self, texture_format: NtrTextureFormat) -> Self {
        Self {
            texture_format,
            ..self
        }
    }

    pub fn with_mapping_type(self, mapping_type: NtrMappingType) -> Self {
        Self {
            mapping_type,
            ..self
        }
    }

    pub fn with_character_format(self, character_format: NtrCharacterFormat) -> Self {
        Self {
            character_format,
            ..self
        }
    }

    pub fn include_cpos(self, include_cpos: bool) -> Self {
        Self {
            include_cpos,
            ..self
        }
    }
}

#[derive(Debug)]
pub struct Ncgr {
    version: NtrFileVersion,
    texture_format: NtrTextureFormat,
    mapping_data: MappingData,
    character_data: CharacterData,
    include_cpos: bool,
}

impl NtrFormat for Ncgr {
    fn read_from_ntr_file(file: &NtrFile) -> std::io::Result<Self> {
        assert!(file.id() == "RGCN");

        let char_block = &file.blocks()[0];
        assert!(char_block.id() == "RAHC");
        let mut char = char_block.contents();

        let height_in_tiles = char.read_u16()?;
        let width_in_tiles = char.read_u16()?;
        let texture_format: NtrTextureFormat = char.read_u16()?.into();
        match texture_format {
            NtrTextureFormat::Palette16 | NtrTextureFormat::Palette256 => {}
            _ => panic!(),
        };
        _ = char.read_u16()?;

        let mapping_type = NtrMappingType::from_u32_ncgr(char.read_u32()?);
        let mapping_data = if matches!(mapping_type, NtrMappingType::Mode2D) {
            assert!(height_in_tiles != 0xFFFF);
            assert!(width_in_tiles != 0xFFFF);
            MappingData::TwoD((width_in_tiles.into(), height_in_tiles.into()))
        } else {
            assert!(height_in_tiles == 0xFFFF);
            assert!(width_in_tiles == 0xFFFF);
            match mapping_type {
                NtrMappingType::Mode2D => unreachable!(),
                NtrMappingType::Mode1D32K => MappingData::OneD(Mapping1DVariant::Vram32),
                NtrMappingType::Mode1D64K => MappingData::OneD(Mapping1DVariant::Vram64),
                NtrMappingType::Mode1D128K => MappingData::OneD(Mapping1DVariant::Vram128),
                NtrMappingType::Mode1D256K => MappingData::OneD(Mapping1DVariant::Vram256),
            }
        };

        let character_format: NtrCharacterFormat = char.read_u32()?.into();
        let tiles_size = char.read_u32()?;
        let tiles_offset = char.read_u32()?;
        assert!(tiles_offset == 0x00000018);

        let raw_data = char.read_sized(tiles_size.try_into().unwrap())?;
        let character_data =
            Self::raw_data_to_character_data(&raw_data, texture_format, character_format);

        let include_cpos = if file.blocks().len() > 1 {
            let cpos_block = &file.blocks()[1];
            assert!(cpos_block.id() == "SOPC");
            true
        } else {
            false
        };

        Ok(Self {
            version: file.version(),
            texture_format,
            mapping_data,
            character_data,
            include_cpos,
        })
    }

    fn write_to_ntr_file(&self) -> std::io::Result<crate::ntr::NtrFile> {
        let mut blocks = vec![self.to_char_block()?];
        if self.include_cpos {
            blocks.push(self.to_cpos_block()?);
        }

        Ok(NtrFile::new("RGCN", self.version, blocks))
    }
}

impl Ncgr {
    pub fn from_image(image: Image, metadata: NcgrMetadata) -> Self {
        Self {
            version: metadata.version,
            texture_format: metadata.texture_format,
            mapping_data: match metadata.mapping_type {
                NtrMappingType::Mode2D => {
                    MappingData::TwoD((image.width_in_tiles(), image.height_in_tiles()))
                }
                NtrMappingType::Mode1D32K => MappingData::OneD(Mapping1DVariant::Vram32),
                NtrMappingType::Mode1D64K => MappingData::OneD(Mapping1DVariant::Vram64),
                NtrMappingType::Mode1D128K => MappingData::OneD(Mapping1DVariant::Vram128),
                NtrMappingType::Mode1D256K => MappingData::OneD(Mapping1DVariant::Vram256),
            },
            character_data: match metadata.character_format {
                NtrCharacterFormat::Character | NtrCharacterFormat::Character256 => {
                    CharacterData::Character(
                        pixels_to_tiles(image.pixels(), image.width_in_tiles()),
                        metadata.character_format.into(),
                    )
                }
                NtrCharacterFormat::Bitmap => CharacterData::Bitmap(image.pixels().to_vec()),
            },
            include_cpos: metadata.include_cpos,
        }
    }

    /// Can be called only when `mapping_type` in [NcgrMetadata] is 2D. Panics otherwise.
    pub fn to_image(&self) -> Image {
        assert!(matches!(self.mapping_type(), NtrMappingType::Mode2D));

        let width_in_tiles = match &self.mapping_data {
            MappingData::TwoD((w, _)) => *w,
            MappingData::OneD(_) => unreachable!(),
        };

        self.to_image_internal(width_in_tiles)
    }

    /// Can be called only when `mapping_type` in [NcgrMetadata] is 1D. Panics otherwise.
    pub fn to_image_with_width(&self, width: usize) -> Image {
        assert!(!matches!(self.mapping_type(), NtrMappingType::Mode2D));

        assert!(width % TILE_LENGTH == 0);
        let width_in_tiles = width / TILE_LENGTH;

        self.to_image_internal(width_in_tiles)
    }

    fn to_image_internal(&self, width_in_tiles: usize) -> Image {
        let pixels = match &self.character_data {
            CharacterData::Character(tiles, _) => &tiles_to_pixels(tiles, width_in_tiles),
            CharacterData::Bitmap(pixels) => pixels,
        };
        Image::new(width_in_tiles * TILE_LENGTH, pixels, None)
    }

    pub fn metadata(&self) -> NcgrMetadata {
        NcgrMetadata {
            version: self.version,
            texture_format: self.texture_format,
            mapping_type: self.mapping_type(),
            character_format: self.character_format(),
            include_cpos: self.include_cpos,
        }
    }

    pub fn cipher(self, key: u32) -> Self {
        let ciphered_data = cipher(&self.character_data_to_raw_data(), key);
        let character_data = Self::raw_data_to_character_data(
            &ciphered_data,
            self.texture_format,
            self.character_format(),
        );

        Self {
            character_data,
            ..self
        }
    }

    pub fn decipher(self) -> (Self, u32) {
        let (deciphered_data, key) = decipher(&self.character_data_to_raw_data());
        let character_data = Self::raw_data_to_character_data(
            &deciphered_data,
            self.texture_format,
            self.character_format(),
        );

        (
            Self {
                character_data,
                ..self
            },
            key,
        )
    }

    fn mapping_type(&self) -> NtrMappingType {
        match &self.mapping_data {
            MappingData::TwoD(_) => NtrMappingType::Mode2D,
            MappingData::OneD(mapping1_dvariant) => match mapping1_dvariant {
                Mapping1DVariant::Vram32 => NtrMappingType::Mode1D32K,
                Mapping1DVariant::Vram64 => NtrMappingType::Mode1D64K,
                Mapping1DVariant::Vram128 => NtrMappingType::Mode1D128K,
                Mapping1DVariant::Vram256 => NtrMappingType::Mode1D256K,
            },
        }
    }

    fn character_format(&self) -> NtrCharacterFormat {
        match &self.character_data {
            CharacterData::Character(_, value) => match value {
                0 => NtrCharacterFormat::Character,
                256 => NtrCharacterFormat::Character256,
                _ => panic!(),
            },
            CharacterData::Bitmap(_) => NtrCharacterFormat::Bitmap,
        }
    }

    fn character_data_to_raw_data(&self) -> Vec<u8> {
        let raw_data = match &self.character_data {
            CharacterData::Character(tiles, _) => {
                &tiles.iter().flatten().copied().collect::<Vec<u8>>()
            }
            CharacterData::Bitmap(pixels) => pixels,
        };

        match &self.texture_format {
            NtrTextureFormat::Palette16 => raw_data
                .chunks(2)
                .map(|chunk| chunk[0] | (chunk[1] << 4))
                .collect::<Vec<u8>>(),
            NtrTextureFormat::Palette256 => raw_data.to_vec(),
            _ => panic!(),
        }
    }

    fn raw_data_to_character_data(
        raw_data: &[u8],
        texture_format: NtrTextureFormat,
        character_format: NtrCharacterFormat,
    ) -> CharacterData {
        let pixels = match texture_format {
            NtrTextureFormat::Palette16 => Image::raw_data_4bpp_to_pixels(raw_data),
            NtrTextureFormat::Palette256 => Image::raw_data_8bpp_to_pixels(raw_data),
            _ => panic!(),
        };

        match character_format {
            NtrCharacterFormat::Character | NtrCharacterFormat::Character256 => {
                CharacterData::Character(
                    pixels
                        .chunks(TILE_LENGTH * TILE_LENGTH)
                        .map(|tile| tile.try_into().unwrap())
                        .collect(),
                    character_format.into(),
                )
            }
            NtrCharacterFormat::Bitmap => CharacterData::Bitmap(pixels),
        }
    }

    fn to_char_block(&self) -> std::io::Result<NtrFileBlock> {
        let mut char = vec![];
        let texture_format = self.texture_format;

        let (width_in_tiles, height_in_tiles) = match &self.mapping_data {
            MappingData::TwoD((w, h)) => (*w, *h),
            MappingData::OneD(_) => (0xFFFF, 0xFFFF),
        };

        char.write_u16(height_in_tiles.try_into().unwrap())?;
        char.write_u16(width_in_tiles.try_into().unwrap())?;
        char.write_u16(texture_format.into())?;
        char.write_u16(0x0000)?;
        char.write_u32(self.mapping_type().into_u32_ncgr())?;
        char.write_u32(self.character_format().into())?;

        let raw_data = self.character_data_to_raw_data();

        char.write_u32(raw_data.len().try_into().unwrap())?;
        char.write_u32(0x00000018)?;
        char.write_all(&raw_data)?;

        Ok(NtrFileBlock::new("RAHC", char))
    }

    fn to_cpos_block(&self) -> std::io::Result<NtrFileBlock> {
        let (width_in_tiles, height_in_tiles) = match &self.mapping_data {
            MappingData::TwoD((w, h)) => (*w, *h),
            MappingData::OneD(_) => unimplemented!(),
        };
        let mut cpos = vec![];
        cpos.write_u16(0x0000)?;
        cpos.write_u16(0x0000)?;
        cpos.write_u16(width_in_tiles.try_into().unwrap())?;
        cpos.write_u16(height_in_tiles.try_into().unwrap())?;
        Ok(NtrFileBlock::new("SOPC", cpos))
    }
}

impl FileFormat for Ncgr {
    fn extension() -> String {
        "NCGR".to_string()
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        NtrFormat::read_from_data(data)
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        NtrFormat::write_to_data(self)
    }
}

fn cipher(data: &[u8], key: u32) -> Vec<u8> {
    let mut out = vec![];
    let mut internal_key = key;
    for mut chunk in data.chunks(2).rev() {
        internal_key = ((internal_key as i32) - 24691) as u32;
        internal_key = ((internal_key as u64) * 4005161829) as u32;
        let val = chunk.read_u16().unwrap() ^ (internal_key as u16);
        out.push((val >> 8) as u8);
        out.push((val & 0xFF) as u8);
    }
    out.reverse();
    out
}

fn decipher(data: &[u8]) -> (Vec<u8>, u32) {
    let mut out = vec![];
    let mut key: u32 = u16::from_le_bytes(data[0..2].try_into().unwrap()).into();
    for mut chunk in data.chunks(2) {
        let val = chunk.read_u16().unwrap() ^ (key as u16);
        out.push((val & 0xFF) as u8);
        out.push((val >> 8) as u8);
        key = ((key as u64) * 1103515245) as u32;
        key += 24691;
    }
    (out, key)
}
