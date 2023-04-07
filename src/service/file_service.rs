use std::{fs::File, io::Read, path::Path};

pub fn read_file(filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    const BUFFER_LEN: usize = 512;
    let mut buffer = [0u8; BUFFER_LEN];
    let mut file = File::open(filepath)?;

    loop {
        let read_count = file.read(&mut buffer)?;
        println!("{:?}", &buffer[..read_count]);

        if read_count != BUFFER_LEN {
            break;
        }
    }
    Ok(())
}

pub fn extract_fname(path: &str) -> String {
    return Path::new(&path)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
}
