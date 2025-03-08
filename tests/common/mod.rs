use std::{
    hash::{DefaultHasher, Hasher},
    path::PathBuf,
};

pub fn hash_file<P>(path: P) -> std::io::Result<u64>
where
    P: AsRef<std::path::Path>,
{
    let mut hasher = DefaultHasher::new();
    hasher.write(&std::fs::read(path)?);
    Ok(hasher.finish())
}

pub fn assert_same_hash<P, Q>(path1: P, path2: Q)
where
    P: AsRef<std::path::Path>,
    Q: AsRef<std::path::Path>,
{
    let expected_hash = hash_file(&path1).unwrap();
    let actual_hash = hash_file(&path2).unwrap();
    assert!(
        expected_hash == actual_hash,
        "{:?} != {:?}",
        path1.as_ref(),
        path2.as_ref()
    );
}

pub fn file_has_extension(file_path: &PathBuf, extension: &str) -> bool {
    file_path
        .extension()
        .is_some_and(|ext| ext.to_ascii_lowercase() == extension)
}
