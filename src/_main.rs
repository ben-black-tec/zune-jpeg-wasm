use std::env;
use std::fs::File;
use std::fs;
use std::io::Read;
use zune_jpeg::JpegDecoder;
use zune_core::options::DecoderOptions;
use zune_core::colorspace::ColorSpace;
use std::time::Instant;

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
fn decode_img(encoded: &Vec<u8> ) -> Vec<u8>  {
    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(encoded, options);
    let bytes = decoder.decode().expect("could not decode jpeg");
    return bytes
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let fname = args.get(1).expect("needs 1 argument, the filename to decode");
    let encoded_image =  get_file_as_byte_vec(fname);
    let now = Instant::now();
    let decoded = decode_img(&encoded_image);
    let _decoded1 = decode_img(&encoded_image);
    let _d_ecoded2 = decode_img(&encoded_image);
    let _decoded3 = decode_img(&encoded_image);
    let _decoded4 = decode_img(&encoded_image);
    let elapsed = now.elapsed();
    println!("{} bytes, expected {}, decoded in {}ms",decoded.len(),1536*1536*4,elapsed.as_millis()/5);
}
