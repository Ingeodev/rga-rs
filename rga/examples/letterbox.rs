use rga::{RgaBuffer, ImageLayout, PixelFormat};

fn main() -> anyhow::Result<()> {
    let asset_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sample.jpg");
    let img = image::open(asset_path)?.to_rgb8();
    let (src_w, src_h) = img.dimensions();
    let src_bytes = img.into_raw();
    let fmt = PixelFormat::Rgb888;

    let src_layout = ImageLayout::tight(src_w, src_h).aligned(fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, src_layout, fmt)?;

    // tamaño del canvas destino (lo que espera el modelo)
    let canvas_w: u32 = 640;
    let canvas_h: u32 = 640;

    // escala que preserva aspect ratio: el lado más restrictivo manda
    let scale = (canvas_w as f64 / src_w as f64).min(canvas_h as f64 / src_h as f64);
    let resized_w = (src_w as f64 * scale).round() as i32;
    let resized_h = (src_h as f64 * scale).round() as i32;

    // centrado dentro del canvas
    let offset_x = (canvas_w as i32 - resized_w) / 2;
    let offset_y = (canvas_h as i32 - resized_h) / 2;

    let dst_layout = ImageLayout::tight(canvas_w, canvas_h).aligned(fmt);
    let buf_size = (dst_layout.hstride * dst_layout.wstride * 3) as usize;
    let mut dst_bytes = vec![0u8; buf_size];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, fmt)?;

    // paso 1: rellenar todo el canvas con gris (114,114,114), estándar YOLO
    let gray: u32 = 0x00727272;
    rga::fill(&dst, (0, 0, canvas_w as i32, canvas_h as i32), gray)?;

    // paso 2: escribir la imagen redimensionada solo en su ventana centrada
    rga::resize_into_rect(&src, &dst, (offset_x, offset_y, resized_w, resized_h))?;

    let tight = dst.tight_data();
    drop(dst);

    let output_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/output_letterbox.png");
    image::save_buffer(output_path, &tight, canvas_w, canvas_h, image::ColorType::Rgb8)?;
    println!(
        "OK: assets/output_letterbox.png ({canvas_w}x{canvas_h}), imagen real {resized_w}x{resized_h} en offset ({offset_x},{offset_y}), scale={scale:.4}"
    );

    Ok(())
}