use std::{fs, io, path::Path};

use sha2::{Digest, Sha256};

pub fn extract_fname(path: &str) -> String {
    return Path::new(&path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
}

pub fn calculate_file_hash(path_and_fname: &str) -> Option<String> {
    if Path::new(path_and_fname).is_dir() {
        return None;
    }
    let mut file = fs::File::open(&path_and_fname).unwrap();
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let hash = hasher.finalize();
    let result = hex::encode(hash);
    return Some(result);
}
