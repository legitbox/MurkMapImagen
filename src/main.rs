use std::fs;
use bincode;
use std::collections::HashMap;
use flate2::read::GzDecoder;
use image::{Rgba, RgbaImage, imageops, GenericImage, Pixel, GenericImageView, DynamicImage};
use std::fs::File;
use std::io::{self, BufReader, Read};
use anyhow::Context;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use regex::Regex;
use serde::{ Deserialize};

type BlockColors = HashMap<String, [u8; 4]>;

fn load_color_codes() -> std::io::Result<BlockColors> {
    let file = File::open("./imagen_color_codes.json");
    let reader = BufReader::new(file?);
    let color_data: BlockColors = serde_json::from_reader(reader).expect("Invalid Color Data file");
    Ok(color_data)
}

fn is_all_zeros(image: &RgbaImage) -> bool {
    !image.pixels().any(|pixel| pixel.channels() != [0, 0, 0, 0])
}


fn get_filenames_in_dir(dir_path: &str) -> Vec<String> {
    let mut filenames = Vec::new();

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.path().file_name() {
                if let Some(filename_str) = filename.to_str() {
                    filenames.push(filename_str.to_string());
                }
            }
        }
    }

    filenames
}

fn create_chunk_image(chunk_scan: Vec<(i32,i32,i32,String)>, chunk_coordinates: (i32,i32), color_data: &BlockColors) {
    let normalised_coordinates:(u32,u32) = ((chunk_coordinates.0 * 16) as u32, (chunk_coordinates.1*16) as u32);
    let mut chunk_image_vector: Vec<RgbaImage> = Vec::new();
    let mut final_chunk_image = RgbaImage::new(16, 16);
    for y in -64..320 {
        let mut image = RgbaImage::new(16,16);
        for block in chunk_scan.iter() {
            if block.1 == y { {
                    if let Some(color) = color_data.get(&block.3) {
                        image.put_pixel(
                            (block.0 as u32) - normalised_coordinates.0,
                            (block.2 as u32) - normalised_coordinates.1,
                            Rgba(*color),
                        );
                    }
                }
            }
        }
        chunk_image_vector.push(image);
    }
    for layer_index in 0..chunk_image_vector.len() {
        let mut ditherer_switch_x = false;
        let mut ditherer_switch_z = false;
        if is_all_zeros(&chunk_image_vector[layer_index]) {
            continue;
        }
        for z in 0..16{
            for x in (0..16).rev(){
                let mut highlight_layer = chunk_image_vector[layer_index].clone();
                let mut next_layer = chunk_image_vector[layer_index+1].clone();
                let mut pixel = chunk_image_vector[layer_index].get_pixel_mut(x,z);
                if x < 16 {
                    if x != 15 {
                        let forward_pixel = highlight_layer.get_pixel_mut(x + 1, z);
                        if forward_pixel.channels() == [0, 0, 0, 0] {
                            if pixel.channels() != [0, 0, 0, 0] {
                                pixel[0] = pixel[0].saturating_sub(10);
                                pixel[1] = pixel[1].saturating_sub(10);
                                pixel[2] = pixel[2].saturating_sub(10);
                            }
                        }
                    } else {
                        if ditherer_switch_x {
                            if pixel.channels() != [0, 0, 0, 0] {
                                pixel[0] = pixel[0].saturating_sub(5);
                                pixel[1] = pixel[1].saturating_sub(5);
                                pixel[2] = pixel[2].saturating_sub(5);
                            }
                            ditherer_switch_x = false;
                        } else {
                            ditherer_switch_x = true;
                        }
                    }
                    if z == 15 {
                        if ditherer_switch_z {
                            if pixel.channels() != [0, 0, 0, 0] {
                                pixel[0] = pixel[0].saturating_sub(3);
                                pixel[1] = pixel[1].saturating_sub(3);
                                pixel[2] = pixel[2].saturating_sub(3);
                            }
                            ditherer_switch_z = false;
                        } else {
                            ditherer_switch_z = true;
                        }
                    }
                    if x != 0 {
                        let forward_pixel = highlight_layer.get_pixel_mut(x - 1, z);
                        if forward_pixel.channels() == [0, 0, 0, 0] {
                            if pixel.channels() != [0, 0, 0, 0] {
                                pixel[0] = pixel[0].saturating_add(10);
                                pixel[1] = pixel[1].saturating_add(10);
                                pixel[2] = pixel[2].saturating_add(10);
                            }
                        }
                    }
                }
            }
        }
        imageops::overlay(&mut final_chunk_image, &chunk_image_vector[layer_index], 0, 0);
    }
    fs::create_dir_all("./img_output").unwrap();
    final_chunk_image.save(format!("./img_output/{}_{}.png", chunk_coordinates.0, chunk_coordinates.1)).unwrap();
}

fn extract_numbers(filename: &str) -> Vec<i32> {
    let re = Regex::new(r"-?\d+").unwrap();
    re.find_iter(filename)
        .filter_map(|m| m.as_str().parse::<i32>().ok())
        .collect()
}

fn deconstruct_binary_file(filename: String, coordinates: (i32,i32), color_codes: &BlockColors) -> std::io::Result<()> {
    let file = File::open(format!("./chunk_binary_data/{filename}"))?;
    let mut decoder = GzDecoder::new(file);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    let chunk_scan: Vec<(i32, i32, i32, String)> = bincode::deserialize(&decompressed_data).unwrap();
    create_chunk_image(chunk_scan, (coordinates.0,coordinates.1), &color_codes);
    Ok(())
}


fn main() -> io::Result<()> {
    let files = get_filenames_in_dir("./chunk_binary_data");
    println!("Warning! Running the Imagen, get a fire extinguisher!");

    let color_data: BlockColors = load_color_codes()?;

    let pool = ThreadPoolBuilder::new().num_threads(20).build().unwrap();
    pool.install(|| {
        files.par_iter().try_for_each(|file: &String| {
            let numbers = extract_numbers(file);
            let coordinates = (numbers[0], numbers[1]);
            deconstruct_binary_file(file.clone(), coordinates, &color_data)
        }).unwrap();
    });

    let img_dir = "./img_output";
    let output_path = "stitched.png";

    // Collect all chunk files and parse coordinates
    let mut chunks = Vec::new();
    for entry in fs::read_dir(img_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("png") {
            let file_name = path.file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid filename").unwrap();
            let parts: Vec<_> = file_name.split('_').collect();
            if parts.len() != 2 {
                eprintln!("Skipping file with invalid name: {:?}", path);
                continue;
            }
            let x = parts[0].parse::<i32>().unwrap();
            let z = parts[1].parse::<i32>().unwrap();
            chunks.push((x, z, path));
        }
    }

    // Find min and max coordinates
    let min_x = chunks.iter().map(|&(x, _, _)| x).min().unwrap();
    let max_x = chunks.iter().map(|&(x, _, _)| x).max().unwrap();
    let min_z = chunks.iter().map(|&(_, z, _)| z).min().unwrap();
    let max_z = chunks.iter().map(|&(_, z, _)| z).max().unwrap();

    // Calculate dimensions of the stitched image
    let width = (max_x - min_x + 1) as u32 * 16;
    let height = (max_z - min_z + 1) as u32 * 16;

    // Create a new image buffer
    let mut stitched = DynamicImage::new_rgba8(width, height);

    // Process each chunk
    for (x, z, path) in chunks {
        let img = image::open(&path).unwrap();

        // Corrected Z-axis calculation
        let x_pos = (x - min_x) as u32 * 16;
        let z_pos = (z - min_z) as u32 * 16; // Northern chunks at the top

        stitched.copy_from(&img, x_pos, z_pos).unwrap();
    }

    // Save the result
    stitched.save(output_path).unwrap();
    println!("Saved stitched image to {}", output_path);
    Ok(())
}