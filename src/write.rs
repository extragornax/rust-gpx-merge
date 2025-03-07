use axum::body::Bytes;
use gpx::Gpx;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn write_gpx(out_path: String, data: &Gpx) -> std::io::Result<()> {
    let gpx_file = File::create(out_path)?;
    let buf = BufWriter::new(gpx_file);
    if let Err(e) = gpx::write(data, buf) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{:?}", e),
        ));
    };

    Ok(())
}

pub fn write_multipart_file(
    file_name: String,
    name: String,
    content_type: String,
    data: Bytes,
) -> std::io::Result<()> {
    let prefix = "traces_import";
    let file_name_bash = blake3::hash(file_name.as_bytes()).to_string();
    println!("Writing traces to file: {} (blake {})", file_name, file_name_bash);

    let full_path = format!("{}/{}", prefix, file_name_bash);
    let path = Path::new(&full_path);
    let mut file = File::create(&path)?;
    let mut writer = BufWriter::new(&mut file);

    writeln!(writer, "Name: {}", name)?;
    writeln!(writer, "Content-Type: {}", content_type)?;
    writer.write_all(data.as_ref())?;

    Ok(())
}
