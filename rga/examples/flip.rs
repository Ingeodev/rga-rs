use rga::{RgaBuffer, ImageLayout, PixelFormat};
use rga::flip::FlipMode;

fn main() -> anyhow::Result<()> {
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sample.jpg");
    let img = image::open(asset_path)?.to_rgb8();
    let (w, h) = img.dimensions();
    let src_bytes = img.into_raw();

    let fmt = PixelFormat::Rgb888;
    let layout = ImageLayout::tight(w, h).aligned(fmt);
    let buf_size = (layout.hstride * layout.wstride * 3) as usize;
    let src = RgaBuffer::import_virtual(&src_bytes, layout, fmt)?;

    let mut dst_bytes = vec![0u8; buf_size];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, layout, fmt)?;

    rga::flip(&src, &dst, FlipMode::Horizontal)?;
    let tight = dst.tight_data();
    drop(dst);

    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/output_flip.png");
    image::save_buffer(output_path, &tight, w, h, image::ColorType::Rgb8)?;
    println!("OK: assets/output_flip.png ({w}x{h})");
    Ok(())
}