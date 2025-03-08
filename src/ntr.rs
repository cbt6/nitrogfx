use std::io::Write;

use crate::{
    enums::NtrFileVersion,
    read_write_ext::{ReadExt, WriteExt},
};

pub struct NtrFileBlock {
    id: String,
    contents: Vec<u8>,
}

impl NtrFileBlock {
    pub fn new(id: &str, contents: Vec<u8>) -> Self {
        Self {
            id: id.to_string(),
            contents,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn contents(&self) -> &[u8] {
        &self.contents.as_slice()
    }
}

pub struct NtrMetadata {
    pub version: NtrFileVersion,
}

pub struct NtrFile {
    id: String,
    version: NtrFileVersion,
    blocks: Vec<NtrFileBlock>,
}

impl NtrFile {
    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        assert!(data.len() > 0);
        let mut data = data;
        let file_id = data.read_string(4)?;
        if data.read_u16()? != 0xFEFF {
            unimplemented!();
        }
        let version: NtrFileVersion = data.read_u16()?.into();
        let _file_size = data.read_u32()?;
        assert!(data.read_u16()? == 16);
        let num_blocks = data.read_u16()?;

        let mut blocks = vec![];
        for _ in 0..num_blocks {
            let block_id = data.read_string(4)?;
            let block_size = data.read_u32()?;
            let contents = data.read_sized((block_size - 8).try_into().unwrap())?;
            blocks.push(NtrFileBlock {
                id: block_id,
                contents,
            });
        }

        Ok(NtrFile {
            id: file_id,
            version,
            blocks,
        })
    }

    pub fn new(id: &str, version: NtrFileVersion, blocks: Vec<NtrFileBlock>) -> Self {
        Self {
            id: id.to_string(),
            version,
            blocks,
        }
    }

    pub fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        let mut data = vec![];
        data.write_string(self.id())?;
        data.write_u16(0xFEFF)?;
        data.write_u16(self.version().into())?;

        let file_size = 16
            + self
                .blocks()
                .iter()
                .map(|block| block.contents().len() + 8)
                .sum::<usize>();
        data.write_u32(file_size.try_into().unwrap())?;

        data.write_u16(0x0010)?;
        data.write_u16(self.blocks().len().try_into().unwrap())?;

        for block in self.blocks() {
            data.write_string(block.id())?;
            data.write_u32(u32::try_from(block.contents().len()).unwrap() + 8)?;
            data.write_all(block.contents())?;
        }

        Ok(data)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> NtrFileVersion {
        self.version
    }

    pub fn blocks(&self) -> &Vec<NtrFileBlock> {
        &self.blocks
    }

    pub fn read_labl_block(block: &NtrFileBlock) -> std::io::Result<Vec<String>> {
        fn peek_u32(array: &[u8]) -> u32 {
            u32::from_le_bytes(array[0..4].try_into().unwrap())
        }
        assert!(block.id() == "LBAL");
        // We can never be fully sure how many labels are present in the block,
        // so this is just an estimation at best. An example of when this would
        // fail would be when the first 4 labels are empty strings.
        let mut offsets = vec![];
        let mut labl = block.contents();
        loop {
            let possible_offset = peek_u32(labl);
            if possible_offset > u32::try_from(labl.len()).unwrap() {
                break;
            }
            if !offsets.is_empty() && possible_offset <= *offsets.last().unwrap() {
                break;
            }
            offsets.push(labl.read_u32()?);
        }
        let mut labels = vec![];
        for _ in offsets {
            let mut label = String::new();
            loop {
                let value = labl.read_u8()?;
                assert!(value < 127);
                let ch = value as char;
                if ch == '\0' {
                    break;
                }
                label.push(ch);
            }
            labels.push(label);
        }
        Ok(labels)
    }

    pub fn write_labl_block(labels: &[String]) -> std::io::Result<NtrFileBlock> {
        let mut labl = vec![];
        let mut offset = 0;
        for label in labels {
            labl.write_u32(offset)?;
            offset += u32::try_from(label.len()).unwrap() + 1;
        }
        for label in labels {
            labl.write_string(label)?;
            labl.write_string("\0")?;
        }
        Ok(NtrFileBlock::new("LBAL", labl))
    }
}

pub trait NtrFormat
where
    Self: Sized,
{
    fn read_from_ntr_file(file: &NtrFile) -> std::io::Result<Self>;

    fn write_to_ntr_file(&self) -> std::io::Result<NtrFile>;

    fn read_from_data(data: &[u8]) -> std::io::Result<Self> {
        Self::read_from_ntr_file(&NtrFile::read_from_data(data)?)
    }

    fn write_to_data(&self) -> std::io::Result<Vec<u8>> {
        self.write_to_ntr_file()?.write_to_data()
    }
}
