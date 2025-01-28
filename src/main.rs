use bincode;
use flate2::read::GzDecoder;
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::{self, Read};

fn create_chunk_image(chunk_scan: Vec<(i32,i32,i32,String)>) {
    let x_size:u8 = 16;
    let z_size:u8 = 16;
    let layer_count:u16 = 320;

}

fn main() -> io::Result<()> {
    // Open the compressed binary file
    let file = File::open("./chunk_binary_data/0_0.mmbf")?;

    // Create a GzDecoder to decompress the data
    let mut decoder = GzDecoder::new(file);

    // Read the decompressed data into a Vec<u8>
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;

    // Deserialize the decompressed data back into the original structure
    let chunk_scan: Vec<(i32, i32, i32, String)> = bincode::deserialize(&decompressed_data).unwrap();

    create_chunk_image(chunk_scan);
    Ok(())
}