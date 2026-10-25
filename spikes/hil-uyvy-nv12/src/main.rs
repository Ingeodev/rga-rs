//! Spike HIL — gap 3 del PLAN de `afora-live-svc-rs` (§4.1): validar que la
//! conversión `UYVY → NV12` de rga-rs funciona con **dma-buf reales de la
//! cámara IMX662** (el camino exacto de F6a), evidenciado en placa.
//!
//! Pasos:
//!   1. V4L2 mplane UYVY 1080p sobre rkcif: REQBUFS(MMAP,2) + EXPBUF → 2 fds
//!      dma-buf (`videobuf2_cma_sg`). El slot 0 se encola (frame real); el
//!      slot 1 queda SIN encolar (memoria propia, dst de RGA).
//!   2. DQBUF de un frame real. `import_dma_fd` (slot 0, UYVY) por rga-rs.
//!   3. Conversión A: src dma-buf → dst virtual (CPU-backed): verificación
//!      contra una referencia CPU UYVY→NV12 (BT601 limit) por parche.
//!   4. Conversión B: src dma-buf → dst dma-buf (slot 1): el camino fd→fd
//!      exacto de F6a (RGA escribe en el dma-buf que leerá MPP).
//!   5. Veredicto + checksums.
//!
//! Uso: hil-uyvy-nv12 [width] [height]  (default 1920x1080)

use std::os::fd::AsRawFd;
use std::path::Path;
use std::ptr::null_mut;
use std::time::Duration;

use edgefirst_v4l2::device::Device;
use edgefirst_v4l2::queue::{BufType, Memory, Plane, Queue};
use edgefirst_v4l2::uapi::{
    V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE, V4L2_COLORSPACE_REC709, V4L2_FIELD_NONE,
    V4L2_PIX_FMT_UYVY, V4L2_QUANTIZATION_LIM_RANGE, V4L2_XFER_FUNC_709, V4L2_YCBCR_ENC_709,
    v4l2_format,
};
use rga::{ColorSpaceMode, ImageLayout, PixelFormat, RgaBuffer};

fn main() {
    println!("== HIL UYVY→NV12 (rga-rs import_dma_fd) — cámara IMX662/rkcif ==");

    let width: u32 = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(1920);
    let height: u32 = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(1080);

    // ---------------------------------------------------------------- 1) V4L2
    let setup = open_and_format(width, height).expect("S_FMT UYVY mplane");
    println!(
        "[1] S_FMT: {}x{} stride={} sizeimage={}",
        setup.width, setup.height, setup.stride, setup.sizeimage
    );
    assert_eq!(setup.width, width, "el driver no aceptó la resolución pedida");

    let mut queue = Queue::new(&setup.device, BufType::VideoCaptureMplane)
        .expect("cola mplane");
    let granted = queue.request(Memory::Mmap, 2).expect("REQBUFS(MMAP,2)");
    assert_eq!(granted, 2, "REQBUFS concedió {granted} de 2");

    let src_fd = queue.export(0, 0).expect("EXPBUF slot 0 (src UYVY)");
    let dst_fd = queue.export(1, 0).expect("EXPBUF slot 1 (dst NV12)");
    println!(
        "[1] EXPBUF: src fd={} dst fd={} (exp_name videobuf2_cma_sg esperado)",
        src_fd.as_raw_fd(),
        dst_fd.as_raw_fd()
    );
    for (name, fd) in [("src", src_fd.as_raw_fd()), ("dst", dst_fd.as_raw_fd())] {
        let fdinfo = std::fs::read_to_string(format!("/proc/self/fdinfo/{fd}")).unwrap_or_default();
        let exp = fdinfo.lines().find(|l| l.starts_with("exp_name:")).unwrap_or("exp_name: ?").to_owned();
        println!("    {name}: {exp}");
    }

    // Solo el slot 0 entra en la cola del driver: el slot 1 (dst) es memoria
    // propia mientras no se encole (el driver no lo toca).
    queue
        .enqueue(0, &[Plane::Mmap { bytesused: 0 }], None)
        .expect("QBUF slot 0");
    queue.stream_on().expect("STREAMON");
    println!("[1] STREAMON con 1 buffer (slot 1 fuera de la cola = dst propio)");

    // ------------------------------------------------------------ 2) frame real
    queue.wait(Some(Duration::from_secs(3))).expect("poll");
    let frame = queue.dequeue().expect("DQBUF").expect("sin frame dequeued");
    println!(
        "[2] frame real: slot={} seq={}",
        frame.index, frame.sequence
    );
    assert_eq!(frame.index, 0, "dequeue inesperado del slot {}", frame.index);

    // mmap del src (solo lectura): coherencia CPU→dma-buf verificada por el
    // spike F5a del adaptador (checksum MATCH productor↔consumidor).
    let src_cpu = unsafe {
        libc::mmap(
            null_mut(),
            setup.sizeimage as usize,
            libc::PROT_READ,
            libc::MAP_SHARED,
            src_fd.as_raw_fd(),
            0,
        )
    };
    assert_ne!(src_cpu, libc::MAP_FAILED, "mmap del src dma-buf falló");
    let src_bytes =
        unsafe { std::slice::from_raw_parts(src_cpu as *const u8, setup.sizeimage as usize) };
    println!("[2] fingerprint del src (mmap CPU): FNV-1a = {:016x}", fnv1a(src_bytes));

    // ------------------------------------------- 3) Conversión A: fd → virtual
    let src_layout = ImageLayout::tight(setup.width, setup.height).aligned(PixelFormat::Uyvy422);
    let src = RgaBuffer::import_dma_fd(src_fd.as_raw_fd(), src_layout, PixelFormat::Uyvy422)
        .expect("import_dma_fd (src UYVY) falló — GAP 3 FALLIDO");
    println!("[3] rga-rs import_dma_fd(src UYVY): OK");

    let dst_layout = ImageLayout::tight(setup.width, setup.height).aligned(PixelFormat::Nv12);
    let mut dst_bytes =
        vec![0u8; dst_layout.hstride as usize * dst_layout.wstride as usize * 3 / 2];
    {
        let dst =
            RgaBuffer::import_virtual_mut(&mut dst_bytes, dst_layout, PixelFormat::Nv12)
                .expect("import_virtual (dst NV12) falló");
        rga::convert_color_with_mode(
            &src, &dst, PixelFormat::Uyvy422, PixelFormat::Nv12,
            ColorSpaceMode::YuvBt601LimitRange,
        )
        .expect("convert_color UYVY→NV12 falló — GAP 3 FALLIDO");
    }
    println!("[3] conversión A (src fd → dst virtual): OK");

    let (ys, uvs) = split_nv12(&dst_bytes, &dst_layout);
    println!(
        "[3] NV12 virtual: plano Y no-plano={} plano UV no-neutro={} FNV-1a={:016x}",
        !is_all(ys, |b| b == 8 || b == 0),
        !is_all(uvs, |b| b == 0x80 || b == 0),
        fnv1a(&dst_bytes)
    );

    // Referencia CPU (BT601 limit): Y se conserva (4:2:2→4:2:0 mismo rango) y
    // el croma se decima 2x2. Parche 64x64 alineado al croma.
    let report = compare_with_cpu_reference(src_bytes, &dst_bytes, &src_layout, &dst_layout);
    println!(
        "[3] referencia CPU parche 64x64: Y max|Δ|={} UV max|Δ|={} (filas pares) · UV filas impares max|Δ|={}",
        report.y_max, report.uv_even_max, report.uv_odd_max
    );

    // ------------------------------------------------ 4) Conversión B: fd → fd
    let dst_dma = RgaBuffer::import_dma_fd(dst_fd.as_raw_fd(), dst_layout, PixelFormat::Nv12)
        .expect("import_dma_fd (dst NV12) falló");
    rga::convert_color_with_mode(
        &src, &dst_dma, PixelFormat::Uyvy422, PixelFormat::Nv12,
        ColorSpaceMode::YuvBt601LimitRange,
    )
    .expect("convert_color fd→fd falló");
    println!("[4] conversión B (src fd → dst dma-buf): OK (RGA escribió el dma-buf que en F6a lee MPP)");

    let dst_cpu = unsafe {
        libc::mmap(
            null_mut(),
            setup.sizeimage as usize,
            libc::PROT_READ,
            libc::MAP_SHARED,
            dst_fd.as_raw_fd(),
            0,
        )
    };
    assert_ne!(dst_cpu, libc::MAP_FAILED, "mmap del dst dma-buf falló");
    let dst_dma_bytes =
        unsafe { std::slice::from_raw_parts(dst_cpu as *const u8, setup.sizeimage as usize) };
    let (ys_fd, uvs_fd) = split_nv12(dst_dma_bytes, &dst_layout);
    println!(
        "[4] NV12 dma-buf: plano Y no-plano={} plano UV no-neutro={} FNV-1a={:016x}",
        !is_all(ys_fd, |b| b == 8 || b == 0),
        !is_all(uvs_fd, |b| b == 0x80 || b == 0),
        fnv1a(dst_dma_bytes)
    );

    // Determinismo fd→fd vs fd→virtual: solo la región escrita por RGA
    // (wstride*hstride*1.5); el resto del slot es memoria sin inicializar.
    let written = dst_layout.hstride as usize * dst_layout.wstride as usize * 3 / 2;
    let determinism = fnv1a(&dst_bytes[..written]) == fnv1a(&dst_dma_bytes[..written]);
    println!("[4] determinismo fd→fd vs fd→virtual (checksum): {determinism}");

    // ------------------------------------------------------------ 5) veredicto
    let pass = !is_all(ys, |b| b == 8 || b == 0)
        && !is_all(uvs, |b| b == 0x80 || b == 0)
        && report.y_max <= 3
        && report.uv_even_max <= 3;
    println!("VEREDICTO HIL: {}", if pass { "PASS — gap 3 CERRADO" } else { "FAIL" });
    if !pass {
        std::process::exit(1);
    }

    // ------------------------------------------------------------- limpieza
    queue.stream_off().expect("STREAMOFF");
    queue.request(Memory::Mmap, 0).ok(); // REQBUFS(0)
    unsafe { libc::munmap(src_cpu, setup.sizeimage as usize) };
    unsafe { libc::munmap(dst_cpu, setup.sizeimage as usize) };
    println!("limpieza: STREAMOFF + REQBUFS(0) + munmap — OK");
}

// ---------------------------------------------------------------------------
// Captura V4L2 (copia mínima del spike F5a del adaptador — prototipo)
// ---------------------------------------------------------------------------

struct CaptureSetup {
    device: Device,
    width: u32,
    height: u32,
    stride: u32,
    sizeimage: u32,
}

fn open_and_format(width: u32, height: u32) -> std::io::Result<CaptureSetup> {
    let device = Device::open(Path::new("/dev/video0"))
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    let mut format = v4l2_format {
        type_: V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE,
        ..v4l2_format::default()
    };
    // SAFETY: tipo mplane seleccionado; pix_mp es la vista válida de la unión.
    let pixels = unsafe { format.pix_mp() };
    pixels.width = width;
    pixels.height = height;
    pixels.pixelformat = V4L2_PIX_FMT_UYVY;
    pixels.field = V4L2_FIELD_NONE;
    pixels.colorspace = V4L2_COLORSPACE_REC709;
    pixels.ycbcr_enc = V4L2_YCBCR_ENC_709;
    pixels.quantization = V4L2_QUANTIZATION_LIM_RANGE;
    pixels.xfer_func = V4L2_XFER_FUNC_709;
    pixels.num_planes = 1;
    pixels.plane_fmt[0].bytesperline = width.saturating_mul(2);
    pixels.plane_fmt[0].sizeimage = width.saturating_mul(height).saturating_mul(2);

    device
        .set_format(&mut format)
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    // SAFETY: idem — S_FMT devuelve el formato realmente aplicado.
    let applied = unsafe { format.pix_mp() };
    Ok(CaptureSetup {
        device,
        width: applied.width,
        height: applied.height,
        stride: applied.plane_fmt[0].bytesperline,
        sizeimage: applied.plane_fmt[0].sizeimage,
    })
}

// ---------------------------------------------------------------------------
// Verificación NV12
// ---------------------------------------------------------------------------

struct RefReport {
    y_max: i32,
    uv_even_max: i32,
    uv_odd_max: i32,
}

/// Separa los planos Y / UV de un buffer NV12 según su layout.
fn split_nv12<'a>(data: &'a [u8], layout: &ImageLayout) -> (&'a [u8], &'a [u8]) {
    let plane = layout.hstride as usize * layout.wstride as usize;
    data.split_at(plane)
}

fn is_all(slice: &[u8], predicate: impl Fn(u8) -> bool) -> bool {
    slice.iter().all(|&byte| predicate(byte))
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Convierte un parche 64x64 UYVY→NV12 en Rust (BT601 limit: luma se copia,
/// croma se decima 2x2 tomando las filas pares) y lo compara contra la salida
/// de RGA. Filas cromáticas impares se reportan aparte (si RGA filtra, Δ > 0).
fn compare_with_cpu_reference(
    src: &[u8],
    dst: &[u8],
    src_layout: &ImageLayout,
    dst_layout: &ImageLayout,
) -> RefReport {
    const PATCH: u32 = 64;
    let (w, h) = (src_layout.wstride, src_layout.hstride.min(PATCH + 1).min(dst_layout.hstride));
    assert!(w >= PATCH && h >= PATCH + 1, "layout demasiado pequeño para el parche");

    let src_stride = src_layout.wstride as usize; // UYVY: 2 B/px → wstride*2 bytes
    let dst_ws = dst_layout.wstride as usize;
    let dst_hs = dst_layout.hstride as usize;
    let uv_offset = dst_hs * dst_ws;

    let (mut y_max, mut uv_even_max, mut uv_odd_max) = (0i32, 0i32, 0i32);

    for row in 0..h.min(PATCH + 1) {
        for col in (0..PATCH).step_by(2) {
            // Luma de 2 píxeles UYVY: [U, Y0, V, Y1] (Y se conserva 422→420).
            let base = row as usize * src_stride * 2 + col as usize * 2;
            for (dy, sy) in [(0usize, 1usize), (0usize, 3usize)] {
                let got = dst[row as usize * dst_ws + col as usize + dy] as i32;
                let want = src[base + sy] as i32;
                y_max = y_max.max((got - want).abs());
            }
            // Croma solo en filas pares (decimado 2x2 → NV12 uv intercalado).
            if row % 2 == 0 {
                let uv_row = (row / 2) as usize;
                let uv_base = uv_offset + uv_row * dst_ws + col as usize;
                let u_out = dst[uv_base] as i32;
                let v_out = dst[uv_base + 1] as i32;
                let u_in = src[base] as i32;
                let v_in = src[base + 2] as i32;
                let delta = (u_out - u_in).abs().max((v_out - v_in).abs());
                uv_even_max = uv_even_max.max(delta);
            } else {
                // Fila impar: si RGA re-samplea croma, aquí puede variar.
                let uv_row = (row / 2) as usize;
                let uv_base = uv_offset + uv_row * dst_ws + col as usize;
                let u_out = dst[uv_base] as i32;
                let v_out = dst[uv_base + 1] as i32;
                let u_in = src[base] as i32;
                let v_in = src[base + 2] as i32;
                let delta = (u_out - u_in).abs().max((v_out - v_in).abs());
                uv_odd_max = uv_odd_max.max(delta);
            }
        }
    }
    RefReport { y_max, uv_even_max, uv_odd_max }
}
