use std::{collections::VecDeque, io::Write};

use serde::{Deserialize, Serialize};

use crate::{
    enums::{NtrFileVersion, OamSize, ObjMode},
    ntr::{NtrFile, NtrFileBlock, NtrFormat},
    read_write_ext::{ReadExt, WriteExt},
    FileFormat, NtrMappingType,
};

#[derive(Debug, Deserialize, Serialize)]
struct BoundingRectangle {
    max_x: i16,
    max_y: i16,
    min_x: i16,
    min_y: i16,
}

#[derive(Debug, Deserialize, Serialize)]
struct OamData {
    y: i8,
    x: i16,

    affine: bool,

    disable: bool,
    h_flip: bool,
    v_flip: bool,

    mode: ObjMode,
    mosaic: bool,

    color_mode: u8,

    oam_size: OamSize,
    tile_number: u16,
    priority: u8,
    palette_number: u8,
}

impl Into<(u16, u16, u16)> for &OamData {
    fn into(self) -> (u16, u16, u16) {
        let (shape, size) = self.oam_size.into();

        let y = (self.y as u8) as u16;
        let affine = if self.affine { 1 } else { 0 };
        let disable = if self.disable { 1 } else { 0 };
        let mode: u16 = self.mode.into();
        let mosaic = if self.mosaic { 1 } else { 0 };
        let color_mode = u16::from(self.color_mode);
        let shape = u16::from(shape);

        let attr0 = y
            | (affine << 0x8)
            | (disable << 0x9)
            | (mode << 0xa)
            | (mosaic << 0xc)
            | (color_mode << 0xd)
            | (shape << 0xe);

        let x = if self.x >= 0 { self.x } else { 512 + self.x } as u16;
        let h_flip = if self.h_flip { 1 } else { 0 };
        let v_flip = if self.v_flip { 1 } else { 0 };
        let size = u16::from(size);

        let attr1 = x | (h_flip << 0xc) | (v_flip << 0xd) | (size << 0xe);

        let tile_number = self.tile_number;
        let priority = u16::from(self.priority);
        let palette_number = u16::from(self.palette_number);

        let attr2 = tile_number | (priority << 0xa) | (palette_number << 0xc);

        (attr0, attr1, attr2)
    }
}

impl From<(u16, u16, u16)> for OamData {
    fn from(value: (u16, u16, u16)) -> Self {
        let (attr0, attr1, attr2) = value;

        let y = ((attr0 >> 0) & ((1 << 8) - 1)) as i8;
        let affine = ((attr0 >> 0x8) & 1) != 0;
        let disable = ((attr0 >> 0x9) & 1) != 0;
        let mode = ((attr0 >> 0xa) & ((1 << 2) - 1)).try_into().unwrap();
        let mosaic = ((attr0 >> 0xc) & 1) != 0;
        let color_mode: u8 = (((attr0 >> 0xd) & 1) != 0).try_into().unwrap();
        let shape = ((attr0 >> 0xe) & ((1 << 2) - 1)).try_into().unwrap();

        let x = (attr1 >> 0) & ((1 << 9) - 1);
        let x = x as i16 - (if x < 256 { 0 } else { 512 });
        assert!(-256 <= x && x <= 255);
        let h_flip = ((attr1 >> 0xc) & 1) != 0;
        let v_flip = ((attr1 >> 0xd) & 1) != 0;
        let size = ((attr1 >> 0xe) & ((1 << 2) - 1)).try_into().unwrap();

        let tile_number = (attr2 >> 0) & ((1 << 0xa) - 1);
        let priority = ((attr2 >> 0xa) & ((1 << 2) - 1)).try_into().unwrap();
        let palette_number = ((attr2 >> 0xc) & ((1 << 4) - 1)).try_into().unwrap();

        let oam_size: OamSize = (shape, size).into();

        OamData {
            y,
            x,
            affine,
            disable,
            h_flip,
            v_flip,
            mode,
            mosaic,
            color_mode,
            oam_size,
            tile_number,
            priority,
            palette_number,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct CellAttribute {
    h_flip: bool,
    v_flip: bool,
    has_bounding_rectangle: bool,
    bounding_sphere_radius: u16,
}

impl From<u16> for CellAttribute {
    fn from(value: u16) -> Self {
        let bounding_sphere_radius = value & 0x3f;
        let h_flip = value & (1 << 8) != 0;
        let v_flip = value & (1 << 9) != 0;
        let h_v_flip = value & (1 << 0xa) != 0;
        assert!(h_v_flip == (h_flip && v_flip));
        let has_bounding_rectangle = value & (1 << 0xb) != 0;
        CellAttribute {
            h_flip,
            v_flip,
            has_bounding_rectangle,
            bounding_sphere_radius,
        }
    }
}

impl Into<u16> for CellAttribute {
    fn into(self) -> u16 {
        let h_v_flip = self.h_flip && self.v_flip;
        (self.bounding_sphere_radius & 0x3f)
            | ((self.h_flip as u16) << 8)
            | ((self.v_flip as u16) << 9)
            | ((h_v_flip as u16) << 0xa)
            | ((self.has_bounding_rectangle as u16) << 0xb)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Cell {
    attribute: CellAttribute,
    oam_data: Vec<OamData>,
    bounding_rectangle: Option<BoundingRectangle>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CellVramTransferData {
    src_offset: u32,
    size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct VramData {
    max_size: u32,
    data: Vec<CellVramTransferData>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Ncer {
    version: NtrFileVersion,
    cells: Vec<Cell>,
    mapping_type: NtrMappingType,
    vram_data: Option<VramData>,
    has_user_extended_attribute_data: bool,
    labels: Vec<String>,
}

impl NtrFormat for Ncer {
    fn read_from_ntr_file(file: &NtrFile) -> std::io::Result<Self> {
        assert!(file.id() == "RECN");

        assert!(file.blocks().len() == 3);

        let cebk_block = &file.blocks()[0];
        let labl_block = &file.blocks()[1];
        let uext_block = &file.blocks()[2];

        let (cells, mapping_type, vram_data, has_user_extended_attribute_data) =
            Self::from_cebk_block(cebk_block)?;
        let labels = NtrFile::read_labl_block(labl_block)?;
        assert!(uext_block.id() == "TXEU" && uext_block.contents() == [0, 0, 0, 0]);

        Ok(Self {
            version: file.version(),
            cells,
            mapping_type,
            vram_data,
            has_user_extended_attribute_data,
            labels,
        })
    }

    fn write_to_ntr_file(&self) -> std::io::Result<NtrFile> {
        Ok(NtrFile::new(
            "RECN",
            self.version,
            vec![
                self.to_cebk_block()?,
                NtrFile::write_labl_block(&self.labels)?,
                self.to_uext_block()?,
            ],
        ))
    }
}

impl Ncer {
    pub fn from_json(json: &str) -> std::io::Result<Self> {
        Ok(serde_json::from_str::<Self>(json).unwrap())
    }

    pub fn to_json(&self) -> std::io::Result<String> {
        Ok(serde_json::to_string_pretty(&self).unwrap())
    }

    fn from_cebk_block(
        block: &NtrFileBlock,
    ) -> std::io::Result<(Vec<Cell>, NtrMappingType, Option<VramData>, bool)> {
        assert!(block.id() == "KBEC");
        let mut cebk = block.contents();
        let num_cells = cebk.read_u16()?;
        assert!(num_cells > 0);
        let cell_bank_attributes = cebk.read_u16()?;
        assert!(cell_bank_attributes == 0 || cell_bank_attributes == 1);
        let has_bounding_rectangle = cell_bank_attributes != 0;
        assert!(cebk.read_u32()? == 0x00000018);
        let mapping_type = NtrMappingType::from_u32_ncer(cebk.read_u32()?);
        let vram_offset = cebk.read_u32()?;
        assert!(cebk.read_u32()? == 0);

        let user_extended_attribute_data_offset = cebk.read_u32()?;
        let has_user_extended_attribute_data = user_extended_attribute_data_offset != 0;

        let mut cells = vec![];
        let mut cell_attributes = VecDeque::new();
        let mut bounding_rects = VecDeque::new();
        let mut list_num_oam_attributes = VecDeque::new();

        for _ in 0..num_cells {
            let num_oam_attributes = cebk.read_u16()?;
            list_num_oam_attributes.push_back(num_oam_attributes);
            let cell_attribute: CellAttribute = cebk.read_u16()?.into();
            assert!(has_bounding_rectangle == cell_attribute.has_bounding_rectangle);
            cell_attributes.push_back(cell_attribute);
            let _oam_attrs_offset = cebk.read_u32()?;
            let bounding_rectangle = if has_bounding_rectangle {
                Some(BoundingRectangle {
                    max_x: cebk.read_i16()?,
                    max_y: cebk.read_i16()?,
                    min_x: cebk.read_i16()?,
                    min_y: cebk.read_i16()?,
                })
            } else {
                None
            };
            bounding_rects.push_back(bounding_rectangle);
        }
        for _ in 0..num_cells {
            let mut oam_data = vec![];
            for _ in 0..list_num_oam_attributes.pop_front().unwrap() {
                let attr0 = cebk.read_u16()?;
                let attr1 = cebk.read_u16()?;
                let attr2 = cebk.read_u16()?;
                oam_data.push((attr0, attr1, attr2).into());
            }
            cells.push(Cell {
                attribute: cell_attributes.pop_front().unwrap(),
                oam_data,
                bounding_rectangle: bounding_rects.pop_front().unwrap(),
            });
        }

        // Given that each oam data entry is 2 bytes, if there are an odd number
        // of oam entries, that means there will be an extra 2 bytes at the end
        // as padding to ensure 4-byte alignment.
        let has_padding = (cells.iter().map(|cell| cell.oam_data.len()).sum::<usize>()) % 2 == 1;
        if has_padding {
            let _ = cebk.read_u16()?;
        }

        let vram_data = if vram_offset == 0 {
            None
        } else {
            let max_size = cebk.read_u32()?;
            assert!(cebk.read_u32()? == 0x00000008);
            let mut transfer_data = vec![];
            for _ in 0..num_cells {
                transfer_data.push(CellVramTransferData {
                    src_offset: cebk.read_u32()?,
                    size: cebk.read_u32()?,
                });
            }
            Some(VramData {
                max_size,
                data: transfer_data,
            })
        };

        if user_extended_attribute_data_offset != 0 {
            assert!(cebk.read_string(4)? == "TACU");
            let user_extended_attribute_data_size = cebk.read_u32()?;
            assert!(cebk.read_u16()? == num_cells);
            assert!(user_extended_attribute_data_size == u32::from(16 + num_cells * 8));
            assert!(cebk.read_u16()? == 0x0001);
            assert!(cebk.read_u32()? == 0x00000008);
            for i in 0..num_cells {
                assert!(cebk.read_u32()? == u32::from(8 + 4 * (num_cells + i)));
            }
            for _ in 0..num_cells {
                assert!(cebk.read_u32()? == 0x00000000);
            }
        };

        Ok((
            cells,
            mapping_type,
            vram_data,
            has_user_extended_attribute_data,
        ))
    }

    fn to_cebk_block(&self) -> std::io::Result<NtrFileBlock> {
        let mut cell_oam_data = vec![];
        let mut oam_attrs_offsets = vec![];
        for cell in &self.cells {
            oam_attrs_offsets.push(cell_oam_data.len());
            for oam_data in &cell.oam_data {
                let (attr0, attr1, attr2) = oam_data.into();
                cell_oam_data.write_u16(attr0)?;
                cell_oam_data.write_u16(attr1)?;
                cell_oam_data.write_u16(attr2)?;
            }
        }

        let mut cell_data = vec![];
        for (i, cell) in self.cells.iter().enumerate() {
            cell_data.write_u16(cell.oam_data.len().try_into().unwrap())?;
            cell_data.write_u16(cell.attribute.into())?;
            cell_data.write_u32(oam_attrs_offsets[i].try_into().unwrap())?;
            if let Some(br) = &cell.bounding_rectangle {
                cell_data.write_i16(br.max_x)?;
                cell_data.write_i16(br.max_y)?;
                cell_data.write_i16(br.min_x)?;
                cell_data.write_i16(br.min_y)?;
            }
        }
        cell_data.write_all(&cell_oam_data)?;
        while cell_data.len() % 4 != 0 {
            cell_data.write_u8(0)?;
        }
        let cell_data_len = u32::try_from(cell_data.len()).unwrap();

        let mut vram_data = vec![];
        if let Some(x) = &self.vram_data {
            vram_data.write_u32(x.max_size)?;
            vram_data.write_u32(0x00000008)?;
            for cell_vram_transfer_data in &x.data {
                vram_data.write_u32(cell_vram_transfer_data.src_offset)?;
                vram_data.write_u32(cell_vram_transfer_data.size)?;
            }
        }
        let vram_data_len = u32::try_from(vram_data.len()).unwrap();

        let mut user_extended_attribute_data = vec![];
        if self.has_user_extended_attribute_data {
            let num_cells: u16 = self.cells.len().try_into().unwrap();
            user_extended_attribute_data.write_string("TACU")?;
            user_extended_attribute_data.write_u32((16 + num_cells * 8).into())?;
            user_extended_attribute_data.write_u16(num_cells)?;
            user_extended_attribute_data.write_u16(0x0001)?;
            user_extended_attribute_data.write_u32(0x00000008)?;
            for i in 0..num_cells {
                user_extended_attribute_data.write_u32(u32::from(8 + 4 * (num_cells + i)))?;
            }
            for _ in 0..num_cells {
                user_extended_attribute_data.write_u32(0x00000000)?;
            }
        }

        let mut cebk = vec![];
        cebk.write_u16(self.cells.len().try_into().unwrap())?;
        let has_bounding_rectangle = self.cells[0].attribute.has_bounding_rectangle;
        cebk.write_u16(has_bounding_rectangle.into())?;
        cebk.write_u32(0x00000018)?;
        cebk.write_u32(self.mapping_type.into_u32_ncer())?;
        let vram_offset = match self.vram_data {
            Some(_) => 0x00000018 + cell_data_len,
            None => 0,
        };
        cebk.write_u32(vram_offset)?;
        cebk.write_u32(0x00000000)?;
        let user_extended_attribute_data_offset = match self.has_user_extended_attribute_data {
            true => 0x00000018 + cell_data_len + vram_data_len,
            false => 0,
        };
        cebk.write_u32(user_extended_attribute_data_offset)?;
        cebk.write_all(&cell_data)?;
        cebk.write_all(&vram_data)?;
        cebk.write_all(&user_extended_attribute_data)?;

        Ok(NtrFileBlock::new("KBEC", cebk))
    }

    fn to_uext_block(&self) -> std::io::Result<NtrFileBlock> {
        Ok(NtrFileBlock::new("TXEU", vec![0, 0, 0, 0]))
    }
}

impl FileFormat for Ncer {
    fn extension() -> String {
        "NCER".to_string()
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        NtrFormat::read_from_data(data)
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        NtrFormat::write_to_data(self)
    }
}
