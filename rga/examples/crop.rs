use rga::{RgaBuffer, ImageLayout, PixelFormat};

fn main() -> anyhow::Result<()> {
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sample.jpg");
    let img = image::open(asset_path)?.to_rgb8();
    let (src_w, src_h) = img.dimensions();
    let src_bytes = img.into_raw();

    let fmt = PixelFormat::Rgb888;
    let src_layout = ImageLayout::tight(src_w, src_h).aligned(fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, src_layout, fmt)?;

    let crop_w = 200.min(src_w);
    let crop_h = 200.min(src_h);
    let x = (src_w - crop_w) as i32 / 2;
    let y = (src_h - crop_h) as i32 / 2;

    let dst_layout = ImageLayout::tight(crop_w, crop_h).aligned(fmt);
    let buf_size = (dst_layout.hstride * dst_layout.wstride * 3) as usize;
    let mut dst_bytes = vec![0u8; buf_size];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, fmt)?;
    

    rga::crop(&src, &dst, x, y, crop_w as i32, crop_h as i32)?;
    let tight = dst.tight_data();
    drop(dst);

    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/output_crop.png");
    image::save_buffer(output_path, &tight, crop_w, crop_h, image::ColorType::Rgb8)?;
    println!("OK: assets/output_crop.png ({crop_w}x{crop_h})");
    Ok(())
}