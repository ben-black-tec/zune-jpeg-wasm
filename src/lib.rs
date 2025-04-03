use zune_jpeg::JpegDecoder;
use zune_core::options::DecoderOptions;
use zune_core::colorspace::ColorSpace;
use js_sys;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
pub fn decode_img(encoded_js: &js_sys::Uint8Array ) -> Result<js_sys::Uint8Array,String>  {
    let encoded = encoded_js.to_vec();
    let options: DecoderOptions = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(encoded, options);
    let result = decoder.decode();
    match result{
        Ok(bytes)=>{
            let js_arr = js_sys::Uint8Array::new_with_length(bytes.len()as u32);
            js_arr.copy_from(&bytes);
            Ok(js_arr)
        },
        Err(err)=>Err(format!("Decode error: {err:?}"))
    }
}