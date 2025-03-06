use gpx::Gpx;
use std::fs::File;
use std::io::BufWriter;

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
