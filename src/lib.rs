use js_sys;
use std::io::Cursor;
use zune_core::colorspace::ColorSpace;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

use image::ImageReader;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn decode_jpeg(encoded_js: &js_sys::Uint8Array) -> Result<js_sys::Uint8Array, String> {
    let encoded = encoded_js.to_vec();
    let options: DecoderOptions =
        DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(&encoded, options);
    let result = decoder.decode();
    match result {
        Ok(bytes) => {
            let js_arr = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
            js_arr.copy_from(&bytes);
            Ok(js_arr)
        }
        Err(err) => Err(format!("JPEG Decode error: {err:?}")),
    }
}
#[wasm_bindgen]
pub fn decode_img(
    encoded_js: &js_sys::Uint8Array,
    use_fast_jpeg: &js_sys::Boolean,
) -> Result<js_sys::Uint8Array, String> {
    if use_fast_jpeg.as_bool().or(Some(false)).unwrap()
        && encoded_js.length() > 2
        && encoded_js.get_index(0) == 0xff
        && encoded_js.get_index(1) == 0xD8
    {
        decode_jpeg(encoded_js)
    } else {
        let data = encoded_js.to_vec();
        let img2: Vec<u8> = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .map_err(|err| format!("Error autodetecting image type: {err:?}"))?
            .decode()
            .map_err(|err| format!("Error decoding image: {err:?}"))?
            .to_rgba8()
            .into_raw();
        let js_arr = js_sys::Uint8Array::new_with_length(img2.len() as u32);
        js_arr.copy_from(&img2);
        Ok(js_arr)
    }
}
