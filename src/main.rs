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

fn encode(input: &String, output: &String) {
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

fn decode(input: &String, output: &String) {
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