use rga::{RgaBuffer, ImageLayout, PixelFormat};

fn main() -> anyhow::Result<()> {
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sample.jpg");
    let img = image::open(asset_path)?.to_rgb8();
    let (w, h) = img.dimensions();
    let src_bytes = img.into_raw();

    let layout = ImageLayout::tight(w, h).aligned(PixelFormat::Rgb888);
    let src = RgaBuffer::import_virtual(&src_bytes, layout, PixelFormat::Rgb888)?;

    let dst_layout = ImageLayout::tight(w, h).aligned(PixelFormat::Bgr888);
    let buf_size = (dst_layout.hstride * dst_layout.wstride * 3) as usize;
    let mut dst_bytes = vec![0u8; buf_size];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, PixelFormat::Bgr888)?;

    rga::convert_color(&src, &dst, PixelFormat::Rgb888, PixelFormat::Bgr888)?;
    let tight = dst.tight_data();
    drop(dst);

    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/output_convert_color.png");
    image::save_buffer(output_path, &tight, w, h, image::ColorType::Rgb8)?;
    println!("OK: assets/output_convert_color.png — colores deberían verse invertidos vs sample.jpg");
    Ok(())
}