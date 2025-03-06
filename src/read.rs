use gpx::{Gpx, read};
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufReader, ErrorKind};

pub fn read_gpx(path: &String) -> std::io::Result<Gpx> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let gpx: Gpx =
        read(reader).map_err(|e| std::io::Error::new(ErrorKind::Unsupported, e.to_string()))?;

    Ok(gpx)
}

pub fn get_all_gpx_files_in_dir(root: &String) -> Vec<String> {
    match fs::read_dir(root) {
        Ok(read_dir) => {
            let filtered: Vec<DirEntry> = read_dir.into_iter().filter_map(Result::ok).collect();

            let filtered_gpx = filtered
                .iter()
                .filter(|fi| {
                    let binding = fi.file_name();
                    let file_name = binding.to_str().unwrap_or_default();
                    file_name.ends_with(".gpx")
                })
                .collect::<Vec<&DirEntry>>();

            let file_names = filtered_gpx
                .iter()
                .map(|fi| fi.path().to_str().unwrap_or_default().to_string())
                .collect::<Vec<String>>();
            file_names
        }
        _ => vec![],
    }
}
