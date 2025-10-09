use js_sys;
use std::io::Cursor;
use zune_core::colorspace::ColorSpace;
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

use image::ImageReader;

use wasm_bindgen::prelude::*;
use web_sys::ImageData;

struct RsImg {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

fn decode_jpeg_rs(encoded: &Vec<u8>) -> Result<RsImg, String> {
    let options: DecoderOptions =
        DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(encoded, options);
    let bytes = decoder
        .decode()
        .map_err(|err| format!("JPEG Decode error: {err:?}"))?;
    let dims: (usize, usize) = decoder
        .dimensions()
        .ok_or(format!("JPEG dimension Decode error"))?;
    Ok(RsImg {
        data: bytes,
        width: dims.0 as u32,
        height: dims.1 as u32,
    })
}

fn decode_img_generic_rs(encoded: &Vec<u8>) -> Result<RsImg, String> {
    let decoded = ImageReader::new(Cursor::new(encoded))
        .with_guessed_format()
        .map_err(|err| format!("Error autodetecting image type: {err:?}"))?
        .decode()
        .map_err(|err| format!("Error decoding image: {err:?}"))?;

    let img2: Vec<u8> = decoded.to_rgba8().into_raw();
    Ok(RsImg {
        data: img2,
        width: decoded.width(),
        height: decoded.height(),
    })
}
fn decode_img_rs(encoded: &Vec<u8>, use_fast_jpeg_path: bool) -> Result<RsImg, String> {
    if use_fast_jpeg_path && encoded.len() > 2 && encoded[0] == 0xff && encoded[1] == 0xD8 {
        decode_jpeg_rs(encoded)
    } else {
        decode_img_generic_rs(encoded)
    }
}
fn pack_imgs_rs(decodedImgs: &[RsImg], num_imgs_col: usize) -> Result<RsImg, String> {
    let num_imgs = decodedImgs.len();
    if num_imgs == 0 {
        return Err(format!("decode_packed_imgs_rs needs at least one image"));
    }
    if num_imgs_col <= 0 || num_imgs % num_imgs_col != 0 {
        return Err(format!("Invalid num_imgs_col argument value: {num_imgs_col:?}, not divisible by num_imgs value {num_imgs:?}"));
    }
    let num_rows = num_imgs / num_imgs_col;
    let combinedImageSize: usize = decodedImgs
        .iter()
        .map(|img| (img.width * img.height * 4) as usize)
        .sum();
    let mut combinedImg: Vec<u8> = vec![0 as u8; combinedImageSize];
    let firstImg = decodedImgs.first().unwrap();
    let lastImg = decodedImgs.last().unwrap();
    let mainWidth = firstImg.width as usize;
    let mainHeight = firstImg.height as usize;
    let lastWidth = lastImg.width as usize;
    let lastHeight = lastImg.height as usize;
    let xTotSize = mainWidth * (num_imgs_col - 1) + lastWidth;
    let yTotSize = mainHeight * (num_rows - 1) + lastHeight;
    for yidx in 0..num_rows {
        for xidx in 0..num_imgs_col {
            let img = &decodedImgs[yidx * num_imgs_col + xidx];
            let img_width = img.width as usize;
            let img_height = img.height as usize;
            if ((xidx == num_imgs_col - 1 && img_width != lastWidth)
                || (yidx == num_rows - 1 && img_height != lastHeight)
                || (xidx != num_imgs_col - 1 && img_width != mainWidth)
                || (yidx != num_rows - 1 && img_height != mainHeight))
            {
                return Err(format!("Invalid grid. Images of inconsistent size. num_rows:{num_rows:?}, num_imgs_col:{num_imgs_col:?} xidx: {xidx:?}, yidx: {yidx:?}, img_width: {img_width:?}, img_height: {img_height:?}, mainWidth: {mainWidth:?}, mainHeight: {mainHeight:?}, lastWidth: {lastWidth:?}, lastHeight: {lastHeight:?}"));
            }
            // number of channels
            const nC: usize = 4;
            let offset: usize = yidx * mainHeight * xTotSize * nC + xidx * mainWidth * nC;
            for y in 0..img_height {
                let y_src_offset = offset + y * xTotSize * nC;
                let y_dest_offset = y * img_width * nC;
                combinedImg[y_src_offset..y_src_offset + img_width * nC]
                    .copy_from_slice(&img.data[y_dest_offset..y_dest_offset + img_width * nC]);
            }
        }
    }
    Ok(RsImg {
        data: combinedImg,
        width: xTotSize as u32,
        height: yTotSize as u32,
    })
}

fn to_js_img(rs_img: RsImg) -> Result<ImageData, String> {
    let js_arr: js_sys::Uint8ClampedArray =
        js_sys::Uint8ClampedArray::new_with_length(rs_img.data.len() as u32);
    js_arr.copy_from(&rs_img.data);
    let img_data =
        ImageData::new_with_js_u8_clamped_array_and_sh(&js_arr, rs_img.width, rs_img.height)
            .map_err(|err: JsValue| format!("Error creating JS imagedata: {err:?}"))?;
    Ok(img_data)
}
#[wasm_bindgen]
pub fn decode_jpeg(encoded_js: &js_sys::Uint8Array) -> Result<ImageData, String> {
    let encoded: Vec<u8> = encoded_js.to_vec();
    let rs_img = decode_jpeg_rs(&encoded)?;
    let js_img = to_js_img(rs_img)?;
    Ok(js_img)
}
#[wasm_bindgen]
pub fn decode_img(
    encoded_js: &js_sys::Uint8Array,
    use_fast_jpeg: &js_sys::Boolean,
) -> Result<ImageData, String> {
    let encoded: Vec<u8> = encoded_js.to_vec();
    let use_fast_jpeg_rs = use_fast_jpeg.as_bool().or(Some(false)).unwrap();
    let rs_img = decode_img_rs(&encoded, use_fast_jpeg_rs)?;
    let js_img = to_js_img(rs_img)?;
    Ok(js_img)
}
#[wasm_bindgen]
pub fn decode_pack_imgs(
    encoded_js: &js_sys::Array,
    num_imgs_col: &js_sys::Number,
    use_fast_jpeg: &js_sys::Boolean,
) -> Result<ImageData, String> {
    let use_fast_jpeg_rs = use_fast_jpeg.as_bool().or(Some(false)).unwrap();
    let num_imgs_col_rs = num_imgs_col
        .as_f64()
        .ok_or("Could not decode num_imgs_col into number")? as usize;
    let mut decodedImgs: Vec<RsImg> = Vec::new();
    for js_obj in encoded_js.iter() {
        let js_arr = js_obj.dyn_into::<js_sys::Uint8Array>().map_err(|err| {
            format!("decode_pack_imgs's first argument expects an Array of Uint8Array: {err:?}")
        })?;
        let rs_vec = js_arr.to_vec();
        decodedImgs.push(decode_img_rs(&rs_vec, use_fast_jpeg_rs)?);
    }
    let combined_img = pack_imgs_rs(&decodedImgs, num_imgs_col_rs)?;
    to_js_img(combined_img)
}
