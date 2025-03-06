mod merge;
mod read;
mod stats;
mod write;

use crate::merge::merge_traces;
use crate::read::{get_all_gpx_files_in_dir, read_gpx};
use crate::stats::print_stats;
use crate::write::write_gpx;
use clap::Parser;
use gpx::Gpx;

#[derive(Parser, Debug)]
struct ClapHandler {
    #[clap(short, long)]
    pub source_file: Vec<String>,
    #[clap(long)]
    pub source_directory: Vec<String>,
    #[clap(short, long)]
    pub merge: bool,
    #[clap(short, long)]
    pub print_info: bool,
    #[clap(short, long, default_value = "default_output.gpx")]
    pub destination_file: String,
}

fn main() -> std::io::Result<()> {
    let handler = ClapHandler::parse();
    println!("{:#?}", handler);

    let mut merged = handler.source_file.clone();
    let mut files_in_all_dir: Vec<String> = handler
        .source_directory
        .iter()
        .flat_map(get_all_gpx_files_in_dir)
        .collect();

    merged.append(&mut files_in_all_dir);

    let gpx_extract: Vec<Gpx> = merged
        .iter()
        .map(|_gpx_path| read_gpx(_gpx_path).expect("error while reading source file"))
        .collect::<Vec<_>>();

    println!("Loaded {} gpx files", gpx_extract.len());
    print_stats(&gpx_extract);

    let merged = merge_traces(&gpx_extract);
    write_gpx(handler.destination_file, &merged)?;

    Ok(())
}
