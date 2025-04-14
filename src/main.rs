use std::{env, fs::File, io::{BufReader, BufWriter, Read, Write}, path::Path};
use png::{Encoder, Decoder};

#[derive(Debug, PartialEq, Eq)]
struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

fn encode_png(input: &String, output: &String) {
    let mut f = File::open(Path::new(input)).expect("Unable to open file!");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("Unable to read from file!");

    let mut pixels: Vec<Pixel> = buf.chunks(3).map(|chunk| match chunk.len() {
        1 => Pixel::new(0, chunk[0], 0, 193),
        2 => Pixel::new(chunk[1], chunk[0], 0, 193),
        3 => Pixel::new(chunk[1], chunk[0], chunk[2], 255),
        _ => panic!("Invalid chunk size!"),
    }).collect();

    if pixels.is_empty() {
        println!("No data in pixel buffer.");
        return;
    }

    let total_pixels = pixels.len();
    let width = (total_pixels as f64).sqrt().ceil() as u32;
    let height = (total_pixels as f64 / width as f64).ceil() as u32;

    while pixels.len() < (width * height) as usize {
        pixels.push(Pixel::new(0, 0, 0, 193)); 
    }

    let output_file = File::create(output).expect("Unable to create file!"); 
    let writer = &mut BufWriter::new(output_file);

    let mut enc = Encoder::new(writer, width, height);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);

    let mut w = enc.write_header().expect("Unable to get write header!");
    let data: Vec<u8> = pixels.iter().flat_map(|p| vec![p.r, p.g, p.b, p.a]).collect();
    w.write_image_data(&data).expect("Unable to write image data!");
}

fn encode_tga(input: &String, output: &String) {
    let mut f = File::open(Path::new(input)).expect("Unable to open file!");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).expect("Unable to read from file!");

    let mut pixels: Vec<Pixel> = buf.chunks(3).map(|chunk| match chunk.len() {
        1 => Pixel::new(0, chunk[0], 0, 193),
        2 => Pixel::new(chunk[1], chunk[0], 0, 193),
        3 => Pixel::new(chunk[1], chunk[0], chunk[2], 255),
        _ => panic!("Invalid chunk size!"),
    }).collect();

    if pixels.is_empty() {
        println!("No data in pixel buffer.");
        return;
    }

    let total_pixels = pixels.len();
    let width = (total_pixels as f64).sqrt().ceil() as u16;
    let height = (total_pixels as f64 / width as f64).ceil() as u16;

    while pixels.len() < (width as usize * height as usize) {
        pixels.push(Pixel::new(0, 0, 0, 193)); 
    }

    let file = File::create(output).expect("Unable to create file!");
    let mut writer = BufWriter::new(file);

    let header = [
        0u8,        
        0u8,        
        2u8,        
        0, 0, 0, 0, 0,  
        0, 0,        
        0, 0,        
        (width & 0xFF) as u8, (width >> 8) as u8,
        (height & 0xFF) as u8, (height >> 8) as u8,
        32,          
        0b00100000,  
    ];
    writer.write_all(&header).expect("Failed to write TGA header");

    for pixel in &pixels {
        writer.write_all(&[pixel.b, pixel.g, pixel.r, pixel.a]).expect("Failed to write pixel");
    }

    writer.flush().expect("Failed to flush TGA file");
}

fn decode_png(input: &String, output: &String) {
    let file = File::open(Path::new(input)).expect("Unable to open PNG file!");
    let reader = BufReader::new(file);

    let decoder = Decoder::new(reader);
    let mut reader = decoder.read_info().expect("Failed to read PNG info!");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("Failed to read PNG frame!");

    let rgb_data = &buf[..info.buffer_size()];
    let pixels: Vec<Pixel> = rgb_data.chunks(4)
        .map(|chunk| Pixel { r: chunk[0], g: chunk[1], b: chunk[2], a: chunk[3] })
        .collect();

    let mut original_data = Vec::new();
    for pixel in &pixels {
        if pixel.a == 193 {

            if pixel.r == 0 && pixel.b == 0 {
                original_data.push(pixel.g);
            } else if pixel.b == 0 {
                original_data.push(pixel.g);
                original_data.push(pixel.r);
            }
        } else {

            original_data.push(pixel.g);
            original_data.push(pixel.r);
            original_data.push(pixel.b);
        }
    }

    while let Some(&last) = original_data.last() {
        if last == 0 {
            original_data.pop();
        } else {
            break;
        }
    }

    let mut output_file = File::create(output).expect("Unable to create output file!");
    output_file.write_all(&original_data).expect("Failed to write decoded data!");
}

fn decode_tga(input: &String, output: &String) {
    let mut file = BufReader::new(File::open(Path::new(input)).expect("Unable to open TGA file!"));

    let mut header = [0u8; 18];
    file.read_exact(&mut header).expect("Failed to read TGA header");

    let image_type = header[2];
    if image_type != 2 {
        panic!("Only uncompressed true-color TGA images are supported.");
    }

    let width = u16::from_le_bytes([header[12], header[13]]);
    let height = u16::from_le_bytes([header[14], header[15]]);
    let bpp = header[16];

    let img_desc = header[17];
    let flip_vertically = img_desc & 0b0010_0000 == 0;

    let pixel_count = width as usize * height as usize;
    let bytes_per_pixel = (bpp / 8) as usize;
    let mut raw_pixels = vec![0u8; pixel_count * bytes_per_pixel];
    file.read_exact(&mut raw_pixels).expect("Failed to read TGA pixel data");

    let mut pixels: Vec<Pixel> = Vec::with_capacity(pixel_count);
    for chunk in raw_pixels.chunks_exact(bytes_per_pixel) {
        let (b, g, r, a) = match chunk {
            [b, g, r, a] => (*b, *g, *r, *a),
            [b, g, r] => (*b, *g, *r, 255),
            _ => panic!("Unexpected pixel size"),
        };
        pixels.push(Pixel { r, g, b, a });
    }

    if flip_vertically {
        let row_len = width as usize;
        for y in 0..(height as usize / 2) {
            let top = y * row_len;
            let bottom = (height as usize - 1 - y) * row_len;
            let (top_slice, bottom_slice) = pixels.split_at_mut(bottom);
            top_slice[top..top + row_len].swap_with_slice(&mut bottom_slice[..row_len]);
        }
    }

    let mut original_data = Vec::new();
    for pixel in &pixels {
        if pixel.a == 193 {
            if pixel.r == 0 && pixel.b == 0 {
                original_data.push(pixel.g);
            } else if pixel.b == 0 {
                original_data.push(pixel.g);
                original_data.push(pixel.r);
            }
        } else {
            original_data.push(pixel.g);
            original_data.push(pixel.r);
            original_data.push(pixel.b);
        }
    }

    while let Some(&last) = original_data.last() {
        if last == 0 {
            original_data.pop();
        } else {
            break;
        }
    }

    let mut out = File::create(output).expect("Unable to create output file!");
    out.write_all(&original_data).expect("Failed to write decoded data!");
}

fn encode(input: &String, output: &String) {
    if output.ends_with(".png") {
        encode_png(input, output);
    } else if output.ends_with(".tga") {
        encode_tga(input, output);
    } else {
        panic!("Unsupported file format \"{}\"! Only .png and .tga are supported.", output);
    }
}

fn decode(input: &String, output: &String) {
    if input.ends_with(".png") {
        decode_png(input, output);
    } else if input.ends_with(".tga") {
        decode_tga(input, output);
    } else {
        panic!("Unsupported file format! Only .png and .tga are supported.");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        panic!("Invalid argument count!\nUsage:\n\t{} [input] [output] [encode/decode]", args[0]);
    }

    match args[3].to_lowercase().as_str() {
        "encode" => encode(&args[1], &args[2]),
        "decode" => decode(&args[1], &args[2]),
        x => panic!("Not decoding or encoding! Action \"{}\" is unknown.", x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding() {
        let buf = b"chicken".to_vec();

        let mut pixels: Vec<Pixel> = buf.chunks(3).map(|chunk| match chunk.len() {
            1 => Pixel::new(0, chunk[0], 0, 193),
            2 => Pixel::new(chunk[1], chunk[0], 0, 193),
            3 => Pixel::new(chunk[1], chunk[0], chunk[2], 255),
            _ => panic!("Invalid chunk size!"),
        }).collect();

        if pixels.is_empty() {
            println!("No data in pixel buffer.");
            return;
        }

        let total_pixels = pixels.len();
        let width = (total_pixels as f64).sqrt().ceil() as u32;
        let height = (total_pixels as f64 / width as f64).ceil() as u32;

        while pixels.len() < (width * height) as usize {
            pixels.push(Pixel::new(0, 0, 0, 193));
        }

        assert_eq!(pixels, vec![
            Pixel::new(104, 99, 105, 255),
            Pixel::new(107, 99, 101, 255),
            Pixel::new(0, 110, 0, 193),
            Pixel::new(0, 0, 0, 193)
        ]);
    }

    #[test]
    fn decoding() {
        let pixels: Vec<Pixel> = vec![
            Pixel::new(104, 99, 105, 255),
            Pixel::new(107, 99, 101, 255),
            Pixel::new(0, 110, 0, 193),
            Pixel::new(0, 0, 0, 193)
        ];

        let mut original_data = Vec::new();
        for pixel in &pixels {
            if pixel.a == 193 {
                if pixel.r == 0 && pixel.b == 0 {
                    original_data.push(pixel.g);
                } else if pixel.b == 0 {
                    original_data.push(pixel.g);
                    original_data.push(pixel.r);
                }
            } else {
                original_data.push(pixel.g);
                original_data.push(pixel.r);
                original_data.push(pixel.b);
            }
        }

        while let Some(&last) = original_data.last() {
            if last == 0 {
                original_data.pop();
            } else {
                break;
            }
        }

        assert_eq!(original_data, b"chicken".to_vec());
    }
}