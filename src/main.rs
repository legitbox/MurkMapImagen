use bincode;
use flate2::read::GzDecoder;
use image::{Rgba, RgbaImage, imageops, GenericImage, Pixel};
use std::fs::File;
use std::io::{self, Read};

fn is_all_zeros(image: &RgbaImage) -> bool {
    !image.pixels().any(|pixel| pixel.channels() != [0, 0, 0, 0])
}

fn create_chunk_image(chunk_scan: Vec<(i32,i32,i32,String)>, chunk_coordinates: (i32,i32)) {
    let normalised_coordinates:(u32,u32) = ((chunk_coordinates.0 * 16) as u32, (chunk_coordinates.1*16) as u32);
    let mut chunk_image_vector: Vec<RgbaImage> = Vec::new();
    let mut final_chunk_image = RgbaImage::new(16, 16);
    for y in -64..320 {
        let mut image = RgbaImage::new(16,16);
        for block in chunk_scan.iter() {
            if block.1 == y {
                if block.3 == "minecraft:grass_block"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([112, 137, 57, 255]));
                }
                if block.3 == "minecraft:water"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([70, 148, 168, 120]));
                }
                if block.3 == "minecraft:oak_leaves"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([78, 92, 44, 200]));
                }
                if block.3 == "minecraft:oak_log"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([148, 72, 64, 255]));
                }
                if block.3 == "minecraft:dirt"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([182, 106, 77, 255]));
                }
                if block.3 == "minecraft:stone"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([182, 106, 77, 255]));
                }
                if block.3 == "minecraft:gravel"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([220, 222, 209, 255]));
                }
                if block.3 == "minecraft:clay"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([165, 176, 182, 255]));
                }
                if block.3 == "minecraft:short_grass"{
                    image.put_pixel(block.0 as u32-normalised_coordinates.0, block.2 as u32-normalised_coordinates.1, Rgba([78, 92, 44, 100]));
                }
            }
        }
        chunk_image_vector.push(image);
    }
    let mut brightness_index = 0;
    for layer in &mut chunk_image_vector {
        if is_all_zeros(&layer){
            continue;
        }
        imageops::overlay(&mut final_chunk_image, layer, 0, 0);
        brightness_index += 1;
    }
    final_chunk_image.save("./imgs/result.png".to_string()).unwrap();
}

fn main() -> io::Result<()> {
    let file = File::open("./chunk_binary_data/0_0.mmbf")?;
    let mut decoder = GzDecoder::new(file);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    let chunk_scan: Vec<(i32, i32, i32, String)> = bincode::deserialize(&decompressed_data).unwrap();

    create_chunk_image(chunk_scan, (0,0));
    Ok(())
}