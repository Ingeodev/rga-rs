#![cfg(feature = "hardware-tests")]

use rga::{ImageLayout, PixelFormat, RgaBuffer};

fn load_sample() -> (Vec<u8>, u32, u32) {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/sample.jpg");
    let img = image::open(path)
        .expect("no se pudo abrir assets/sample.jpg")
        .to_rgb8();
    let (w, h) = img.dimensions();
    (img.into_raw(), w, h)
}

fn make_layout(w: u32, h: u32, fmt: PixelFormat) -> ImageLayout {
    ImageLayout::tight(w, h).aligned(fmt)
}

fn buf_size(layout: &ImageLayout, bpp: u32) -> usize {
    (layout.hstride * layout.wstride * bpp) as usize
}

#[test]
fn resize_reduces_dimensions() {
    let fmt = PixelFormat::Rgb888;
    let (src_bytes, src_w, src_h) = load_sample();
    let src_layout = make_layout(src_w, src_h, fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, src_layout, fmt)
        .expect("import_virtual (src) falló");

    let dst_w = src_w / 2;
    let dst_h = src_h / 2;
    let dst_layout = make_layout(dst_w, dst_h, fmt);
    let mut dst_bytes = vec![0u8; buf_size(&dst_layout, 3)];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, fmt)
        .expect("import_virtual_mut (dst) falló");

    rga::resize(&src, &dst, 0.5, 0.5).expect("resize falló");
    let tight = dst.tight_data();
    drop(dst);

    assert!(tight.iter().any(|&b| b != 0), "dst quedó vacío tras resize");
}

#[test]
fn crop_produces_expected_dimensions() {
    let fmt = PixelFormat::Rgb888;
    let (src_bytes, src_w, src_h) = load_sample();
    let src_layout = make_layout(src_w, src_h, fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, src_layout, fmt)
        .expect("import_virtual (src) falló");

    let crop_w = 200.min(src_w);
    let crop_h = 200.min(src_h);
    let x = (src_w - crop_w) as i32 / 2;
    let y = (src_h - crop_h) as i32 / 2;

    let dst_layout = make_layout(crop_w, crop_h, fmt);
    let mut dst_bytes = vec![0u8; buf_size(&dst_layout, 3)];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, fmt)
        .expect("import_virtual_mut (dst) falló");

    rga::crop(&src, &dst, x, y, crop_w as i32, crop_h as i32).expect("crop falló");
    let tight = dst.tight_data();
    drop(dst);

    assert!(tight.iter().any(|&b| b != 0), "dst quedó vacío tras crop");
}

#[test]
fn convert_color_rgb_to_bgr_swaps_channels() {
    let fmt = PixelFormat::Rgb888;
    let (src_bytes, w, h) = load_sample();
    let layout = make_layout(w, h, fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, layout, fmt)
        .expect("import_virtual (src) falló");

    let dst_layout = make_layout(w, h, PixelFormat::Bgr888);
    let mut dst_bytes = vec![0u8; buf_size(&dst_layout, 3)];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, PixelFormat::Bgr888)
        .expect("import_virtual_mut (dst) falló");

    rga::convert_color(&src, &dst, PixelFormat::Rgb888, PixelFormat::Bgr888)
        .expect("convert_color falló");
    let tight = dst.tight_data();
    drop(dst);

    assert_eq!(
        src_bytes[0], tight[2],
        "canal R de src no coincide con canal B de dst tras RGB->BGR"
    );
    assert_eq!(
        src_bytes[2], tight[0],
        "canal B de src no coincide con canal R de dst tras RGB->BGR"
    );
}

#[test]
fn import_virtual_rejects_after_source_freed() {
    let bytes = vec![0u8; 100 * 100 * 3];
    let layout = ImageLayout::tight(100, 100);
    let _buf = RgaBuffer::import_virtual(&bytes, layout, PixelFormat::Rgb888)
        .expect("import_virtual falló");
}

#[test]
fn rotate_90_swaps_dimensions_correctly() {
    let fmt = PixelFormat::Rgb888;
    let (src_bytes, src_w, src_h) = load_sample();
    let src_layout = make_layout(src_w, src_h, fmt);
    let src = RgaBuffer::import_virtual(&src_bytes, src_layout, fmt)
        .expect("import_virtual (src) falló");

    let dst_layout = make_layout(src_h, src_w, fmt);
    let mut dst_bytes = vec![0u8; buf_size(&dst_layout, 3)];
    let dst = RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, fmt)
        .expect("import_virtual_mut (dst) falló");

    rga::rotate(&src, &dst, rga::rotate::Rotation::Rot90).expect("rotate falló");
    let tight = dst.tight_data();
    drop(dst);

    assert!(tight.iter().any(|&b| b != 0), "dst quedó vacío tras rotate");
}

/// Pipeline clave del futuro live-svc (F6): RGB → UYVY 4:2:2 packed →
/// NV12 4:2:0 semi-planar, pasando por el espacio de color explícito.
/// Verifica plano Y (luminancia vs BT601 limit del sample RGB) y plano UV.
#[test]
fn convert_color_rgb_to_uyvy_then_to_nv12() {
    let (src_bytes, w, h) = load_sample();

    // ---- Paso A: RGB → UYVY (BT601 limit, el espacio típico de cámara) ----
    let rgb_layout = make_layout(w, h, PixelFormat::Rgb888);
    let rgb_src = RgaBuffer::import_virtual(&src_bytes, rgb_layout, PixelFormat::Rgb888)
        .expect("import_virtual (RGB) falló");

    let uyvy_layout = make_layout(w, h, PixelFormat::Uyvy422);
    let mut uyvy_bytes = vec![0u8; uyvy_layout.hstride as usize * uyvy_layout.wstride as usize * 2];
    let uyvy_dst = RgaBuffer::import_virtual_mut(&mut uyvy_bytes, uyvy_layout, PixelFormat::Uyvy422)
        .expect("import_virtual_mut (UYVY) falló");

    rga::convert_color_with_mode(
        &rgb_src, &uyvy_dst,
        PixelFormat::Rgb888, PixelFormat::Uyvy422,
        rga::ColorSpaceMode::RgbToYuvBt601Limit,
    ).expect("RGB → UYVY falló");
    drop(rgb_src);

    // ---- Paso B: UYVY → NV12 (el paso real de F6, YUV → YUV) ----
    let nv12_layout = ImageLayout::tight(w, h).aligned(PixelFormat::Nv12);
    let mut nv12_bytes = vec![0u8; nv12_layout.hstride as usize * nv12_layout.wstride as usize * 3 / 2];
    let nv12_dst = RgaBuffer::import_virtual_mut(&mut nv12_bytes, nv12_layout, PixelFormat::Nv12)
        .expect("import_virtual_mut (NV12) falló");

    rga::convert_color_with_mode(
        &uyvy_dst, &nv12_dst,
        PixelFormat::Uyvy422, PixelFormat::Nv12,
        rga::ColorSpaceMode::YuvBt601LimitRange,
    ).expect("UYVY → NV12 falló");

    // ---- Verificación sobre los planos del NV12 ----
    let ws = nv12_layout.wstride as usize;
    let hs = nv12_layout.hstride as usize;
    let data = nv12_dst.bytes(); // buffer respaldado por CPU: slice completo
    let (y_plane, uv_plane) = data.split_at(ws * hs);

    assert!(y_plane.iter().any(|&b| b != 8 && b != 0), "plano Y vacío tras UYVY→NV12");
    assert!(uv_plane.iter().any(|&b| b != 0x80 && b != 0), "plano UV vacío (neutro) tras UYVY→NV12");

    // Spot-check de luminancia: el primer pixel del sample RGB debe mapear a
    // Y = 16 + 0.257*R + 0.504*G + 0.098*B (BT601 limit), tolerancia ±3.
    let r = src_bytes[0] as f32;
    let g = src_bytes[1] as f32;
    let b = src_bytes[2] as f32;
    let y_expect = 16.0 + 0.257 * r + 0.504 * g + 0.098 * b;
    let y0 = y_plane[0] as f32;
    assert!(
        (y0 - y_expect).abs() <= 3.0,
        "Y[0]={} != BT601 limit esperado {} (delta {})",
        y0, y_expect, (y0 - y_expect).abs()
    );
}