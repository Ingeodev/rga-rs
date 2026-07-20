use rga::{RgaBuffer, ImageLayout, PixelFormat};

fn main() -> anyhow::Result<()> {
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sample.jpg");
    let img = image::open(asset_path)?.to_rgb8();
    let (src_w, src_h) = img.dimensions();
    let src_bytes = img.into_raw();

    let fmt = PixelFormat::Rgb888;
    let src_layout = ImageLayout::tight(src_w, src_h).aligned(fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, src_layout, fmt)?;

    let dst_w = src_w / 2;
    let dst_h = src_h / 2;
    let dst_layout = ImageLayout::tight(dst_w, dst_h).aligned(fmt);
    let buf_size = (dst_layout.hstride * dst_layout.wstride * 3) as usize;
    let mut dst_bytes = vec![0u8; buf_size];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, fmt)?;

    rga::resize(&src, &dst, 0.5, 0.5)?;
    let tight = dst.tight_data();
    drop(dst);

    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/output_resize.png");
    image::save_buffer(output_path, &tight, dst_w, dst_h, image::ColorType::Rgb8)?;
    println!("OK: examples/output_resize.png ({dst_w}x{dst_h})");
    Ok(())
}