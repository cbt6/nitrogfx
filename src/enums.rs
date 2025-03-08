use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum NtrFileVersion {
    Version0100,
    Version0101,
}

impl Default for NtrFileVersion {
    fn default() -> Self {
        Self::Version0100
    }
}

impl From<u16> for NtrFileVersion {
    fn from(value: u16) -> Self {
        match value {
            0x0100 => Self::Version0100,
            0x0101 => Self::Version0101,
            _ => panic!(),
        }
    }
}

impl Into<u16> for NtrFileVersion {
    fn into(self) -> u16 {
        match self {
            NtrFileVersion::Version0100 => 0x0100,
            NtrFileVersion::Version0101 => 0x0101,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NtrTextureFormat {
    None,
    A3i5,
    Palette4,
    Palette16,
    Palette256,
    Compressed,
    A5i3,
    Direct,
}

impl Default for NtrTextureFormat {
    fn default() -> Self {
        Self::Palette16
    }
}

impl From<u16> for NtrTextureFormat {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::None,
            1 => Self::A3i5,
            2 => Self::Palette4,
            3 => Self::Palette16,
            4 => Self::Palette256,
            5 => Self::Compressed,
            6 => Self::A5i3,
            7 => Self::Direct,
            _ => panic!(),
        }
    }
}

impl Into<u16> for NtrTextureFormat {
    fn into(self) -> u16 {
        match self {
            NtrTextureFormat::None => 0,
            NtrTextureFormat::A3i5 => 1,
            NtrTextureFormat::Palette4 => 2,
            NtrTextureFormat::Palette16 => 3,
            NtrTextureFormat::Palette256 => 4,
            NtrTextureFormat::Compressed => 5,
            NtrTextureFormat::A5i3 => 6,
            NtrTextureFormat::Direct => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum NtrMappingType {
    Mode2D,
    Mode1D32K,
    Mode1D64K,
    Mode1D128K,
    Mode1D256K,
}

impl Default for NtrMappingType {
    fn default() -> Self {
        Self::Mode2D
    }
}

impl NtrMappingType {
    pub fn from_u32_ncgr(value: u32) -> Self {
        match value {
            0 => Self::Mode2D,
            0x00000010 => Self::Mode1D32K,
            0x00100010 => Self::Mode1D64K,
            0x00200010 => Self::Mode1D128K,
            0x00300010 => Self::Mode1D256K,
            _ => panic!(),
        }
    }

    pub fn into_u32_ncgr(self) -> u32 {
        match self {
            NtrMappingType::Mode2D => 0,
            NtrMappingType::Mode1D32K => 0x00000010,
            NtrMappingType::Mode1D64K => 0x00100010,
            NtrMappingType::Mode1D128K => 0x00200010,
            NtrMappingType::Mode1D256K => 0x00300010,
        }
    }

    pub fn from_u32_ncer(value: u32) -> Self {
        match value {
            0x00000000 => Self::Mode1D32K,
            0x00000001 => Self::Mode1D64K,
            0x00000002 => Self::Mode1D128K,
            0x00000003 => Self::Mode1D256K,
            0x00000004 => Self::Mode2D,
            _ => panic!(),
        }
    }

    pub fn into_u32_ncer(self) -> u32 {
        match self {
            NtrMappingType::Mode1D32K => 0x00000000,
            NtrMappingType::Mode1D64K => 0x00000001,
            NtrMappingType::Mode1D128K => 0x00000002,
            NtrMappingType::Mode1D256K => 0x00000003,
            NtrMappingType::Mode2D => 0x00000004,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NtrCharacterFormat {
    /// Data is arranged in 8x8 tiles. Also sometimes known as "tiled".
    Character,

    /// Data is arranged linearly in sequence like in scanlines. Also sometimes
    /// known as "scanned".
    Bitmap,

    /// Functionally equivalent to [`Character`](NtrCharacterFormat#variant.Character).
    Character256,
}

impl Default for NtrCharacterFormat {
    fn default() -> Self {
        Self::Character
    }
}

impl From<u32> for NtrCharacterFormat {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Character,
            1 => Self::Bitmap,
            256 => Self::Character256,
            _ => panic!(),
        }
    }
}

impl Into<u32> for NtrCharacterFormat {
    fn into(self) -> u32 {
        match self {
            NtrCharacterFormat::Character => 0,
            NtrCharacterFormat::Bitmap => 1,
            NtrCharacterFormat::Character256 => 256,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum OamSize {
    Oam8x8,
    Oam16x16,
    Oam32x32,
    Oam64x64,
    Oam16x8,
    Oam32x8,
    Oam32x16,
    Oam64x32,
    Oam8x16,
    Oam8x32,
    Oam16x32,
    Oam32x64,
}

impl From<(u8, u8)> for OamSize {
    fn from(value: (u8, u8)) -> Self {
        let (shape, size) = value;
        match (shape, size) {
            (0, 0) => Self::Oam8x8,
            (0, 1) => Self::Oam16x16,
            (0, 2) => Self::Oam32x32,
            (0, 3) => Self::Oam64x64,
            (1, 0) => Self::Oam16x8,
            (1, 1) => Self::Oam32x8,
            (1, 2) => Self::Oam32x16,
            (1, 3) => Self::Oam64x32,
            (2, 0) => Self::Oam8x16,
            (2, 1) => Self::Oam8x32,
            (2, 2) => Self::Oam16x32,
            (2, 3) => Self::Oam32x64,
            _ => panic!(),
        }
    }
}

impl Into<(u8, u8)> for OamSize {
    fn into(self) -> (u8, u8) {
        match self {
            OamSize::Oam8x8 => (0, 0),
            OamSize::Oam16x16 => (0, 1),
            OamSize::Oam32x32 => (0, 2),
            OamSize::Oam64x64 => (0, 3),
            OamSize::Oam16x8 => (1, 0),
            OamSize::Oam32x8 => (1, 1),
            OamSize::Oam32x16 => (1, 2),
            OamSize::Oam64x32 => (1, 3),
            OamSize::Oam8x16 => (2, 0),
            OamSize::Oam8x32 => (2, 1),
            OamSize::Oam16x32 => (2, 2),
            OamSize::Oam32x64 => (2, 3),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ObjMode {
    Normal,
    Translucent,
    Window,
    Bitmap,
}

impl From<u16> for ObjMode {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Normal,
            1 => Self::Translucent,
            2 => Self::Window,
            3 => Self::Bitmap,
            _ => panic!(),
        }
    }
}

impl Into<u16> for ObjMode {
    fn into(self) -> u16 {
        match self {
            ObjMode::Normal => 0,
            ObjMode::Translucent => 1,
            ObjMode::Window => 2,
            ObjMode::Bitmap => 3,
        }
    }
}
