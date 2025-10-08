use js_sys;
use std::io::Cursor;
use zune_core::colorspace::ColorSpace;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

use image::ImageReader;

use wasm_bindgen::prelude::*;
use web_sys::ImageData;

#[wasm_bindgen]
pub fn decode_jpeg(encoded_js: &js_sys::Uint8Array) -> Result<ImageData, String> {
    let encoded = encoded_js.to_vec();
    let options: DecoderOptions =
        DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(&encoded, options);
    let result = decoder.decode();
    match result {
        Ok(bytes) => {
            let js_arr = js_sys::Uint8ClampedArray::new_with_length(bytes.len() as u32);
            js_arr.copy_from(&bytes);
            let dims = decoder.dimensions().unwrap();
            let img_data = ImageData::new_with_js_u8_clamped_array_and_sh(
                &js_arr,
                dims.0 as u32,
                dims.1 as u32,
            )
            .map_err(|err: JsValue| format!("Error creating JS imagedata: {err:?}"))?;
            Ok(img_data)
        }
        Err(err) => Err(format!("JPEG Decode error: {err:?}")),
    }
}
#[wasm_bindgen]
pub fn decode_img(
    encoded_js: &js_sys::Uint8Array,
    use_fast_jpeg: &js_sys::Boolean,
) -> Result<ImageData, String> {
    if use_fast_jpeg.as_bool().or(Some(false)).unwrap()
        && encoded_js.length() > 2
        && encoded_js.get_index(0) == 0xff
        && encoded_js.get_index(1) == 0xD8
    {
        decode_jpeg(encoded_js)
    } else {
        let data = encoded_js.to_vec();
        let decoded = ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .map_err(|err| format!("Error autodetecting image type: {err:?}"))?
            .decode()
            .map_err(|err| format!("Error decoding image: {err:?}"))?;

        let img2: Vec<u8> = decoded.to_rgba8().into_raw();
        let js_arr = js_sys::Uint8ClampedArray::new_with_length(img2.len() as u32);
        js_arr.copy_from(&img2);
        let img_data = ImageData::new_with_js_u8_clamped_array_and_sh(
            &js_arr,
            decoded.width() as u32,
            decoded.height() as u32,
        )
        .map_err(|err: JsValue| format!("Error creating JS imagedata: {err:?}"))?;
        Ok(img_data)
    }
}
#[wasm_bindgen]
pub fn decode_pack_imgs(
    encoded_js: &js_sys::Array<js_sys::Uint8Array>,
    use_fast_jpeg: &js_sys::Boolean,
) -> Result<ImageData, String> {

}
