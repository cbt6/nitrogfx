mod common;

use std::fs::{create_dir_all, read_dir, remove_file};

use rayon::iter::{ParallelBridge, ParallelIterator};

use common::{assert_same_hash, file_has_extension};
use nitrogfx::{FileFormat, Jasc, Nclr};

#[test]
fn nclr_to_jasc_to_nclr() {
    let file_ext = "nclr";
    let intermediate_ext = "jasc";

    create_dir_all("tests/assets/temp/").unwrap();
    read_dir(format!("tests/assets/{}", file_ext))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .par_bridge()
        .for_each(|entry| {
            let original_file_path = entry.path();
            if original_file_path.is_dir()
                || original_file_path.metadata().unwrap().len() == 0
                || !file_has_extension(&original_file_path, file_ext)
            {
                return;
            }
            let file_stem = original_file_path.file_stem().unwrap().to_str().unwrap();
            let temp_file_stem = &format!("tests/assets/temp/{}", file_stem);
            let intermediate_path = &format!("{}.{}", temp_file_stem, intermediate_ext);
            let created_file_path = &format!("{}.{}", temp_file_stem, file_ext.to_uppercase());
            let original_file = Nclr::read_from_file(&original_file_path).unwrap();
            Jasc::from_palette(original_file.to_palette())
                .write_to_file(intermediate_path)
                .unwrap();

            let created_file = Nclr::from_palette(
                Jasc::read_from_file(intermediate_path)
                    .unwrap()
                    .to_palette(),
                original_file.metadata(),
            );
            created_file.write_to_file(created_file_path).unwrap();

            assert_same_hash(original_file_path, created_file_path);
            remove_file(intermediate_path).unwrap();
            remove_file(created_file_path).unwrap();
        });
}
