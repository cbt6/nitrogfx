use std::{fs::File, io::Read};

pub trait FileFormat
where
    Self: Sized,
{
    fn extension() -> String;

    fn read_from_file<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let mut data = vec![];
        File::open(path)?.read_to_end(&mut data)?;
        Self::read_from_data(&data)
    }

    fn write_to_file<P>(&self, path: P) -> std::io::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        std::fs::write(path, self.write_to_data()?)
    }

    fn read_from_data(data: &[u8]) -> std::io::Result<Self>;

    fn write_to_data(&self) -> std::io::Result<Vec<u8>>;
}
