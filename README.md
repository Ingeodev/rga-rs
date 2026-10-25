# rga-rs

Bindings Rust para el **RGA** (2D raster accelerator) de Rockchip: crates
`rga` (API segura) + `rga-sys` (FFI sobre `librga`). En el ecosistema Afora es
la pieza que convierte **UYVY → NV12 sin copias de CPU** en todos los clientes
del adaptador de cámara.

## El ecosistema Afora

Afora es un sistema de visión vehicular sobre **Orange Pi 5 Pro (RK3588S,
Armbian 13 aarch64)** con cámara **IMX662-VEYE**. El adaptador de cámara
publica frames UYVY en pub/sub zero-copy (ring dma-buf + SCM_RIGHTS); los
clientes usan este crate para importar los dma-buf por fd y hacer la
conversión/rescalado con hardware 2D, sin tocar la CPU:

```
IMX662 ──MIPI──► rkcif/V4L2 ──► afora-imx662-camera-adapter-rs
                (ring dma-buf · pub/sub UYVY · SCM_RIGHTS)
                          │ IPC (bloque E: eventos + fds)
          ┌───────────────┼──────────────────────┐
          ▼               ▼                      ▼
    inferencia      afora-live-svc-rs      storage MP4 (futuro)
    (RKNN, F5a)     usa rga-rs:            H.265
                    UYVY→NV12 (zero-copy)
                          │ RTSP :8554/afora
                          ▼
                    afora-mediamtx (:8888 WHEP)
                          │ WHEP
                          ▼
     afora-vision (Angular :4200) ◄──REST :8080──► afora-settings-server
```

### Paquetes del ecosistema

| Paquete | Función |
|---|---|
| `afora-imx662-camera-adapter-rs` | Productor de frames + administración de cámara (I2C/V4L2, perfiles, motor de aplicación) |
| `afora-live-svc-rs` | Cliente #2: preview de panel — IPC → RGA (UYVY→NV12) → MPP H.264 → RTSP |
| `afora-mediamtx` | Data-plane de vídeo (router MediaMTX: RTSP :8554, WHEP :8888) |
| `afora-settings-server` | Control plane REST (actix-web :8080); **nunca transporta vídeo** |
| `afora-vision` | Panel Angular 22 (WHEP + REST) |
| `afora-vehicle-counter-rs` | Proyecto existente: tracking/analítica vehicular (independiente) |
| `rga-rs` | **Este repo** — bindings RGA (`rga` + `rga-sys`); path-dep de los clientes |
| `veye-camera-config` | Config de cámaras VEYE (proyecto existente) |

## Arquitectura de este paquete

```
rga-sys/   FFI autogenerada (bindgen) sobre los headers de librga (SDK 1.10.5)
rga/       API segura:
├── buffer.rs    RgaBuffer: import_virtual / import_dma_fd (zero-copy dma-buf)
│               · buffer_size() (semi-planarios = wstride·hstride·1.5) · Drop → release
├── format.rs    PixelFormat: Rgba8888, Rgb888, Bgr888, Nv12, Nv21, Uyvy422 (IMX662)
├── lib.rs       ImageLayout: tight() · aligned() (invariante RGA: wstride·bpp % 16 == 0)
├── error.rs     RgaError::check(status)
└── operations/  convert_color (+_with_mode / ColorSpaceMode BT601/709)
                resize · resize_into_rect · crop · flip · rotate · fill · quantize
```

**Uso típico (pipeline F6 de `afora-live-svc-rs`):**

```rust
let src = RgaBuffer::import_dma_fd(fd_del_ring, layout, PixelFormat::Uyvy422)?;
let dst = /* buffer NV12 propio (dma-buf) */;
rga::convert_color_with_mode(&src, &dst,
    PixelFormat::Uyvy422, PixelFormat::Nv12,
    rga::ColorSpaceMode::YuvBt601LimitRange)?;
```

## Documentación

| Documento | Contenido |
|---|---|
| [`TODO.md`](./TODO.md) | Issues conocidos: errores en `rotate`/`crop` (**no afectan** al pipeline UYVY→NV12/resize) |
| [`rga/tests/ops.rs`](./rga/tests/ops.rs) | Tests de operaciones — requieren placa: `cargo test --features hardware-tests -j 2` |
| [`rga/examples/`](./rga/examples/) | Ejemplos por operación |
| Spike F5a del adaptador | Evidencia del uso con dma-buf reales: `afora-imx662-camera-adapter-rs/assets/tests/f5a/` |

**Estado:** gaps para F6 cerrados (commit `bbbbee5`: `Uyvy422`,
`ColorSpaceMode`/`convert_color_with_mode`, fix de `buffer_size()` para
semi-planarios). Tests 6/6 en placa.
> ⚠️ En esta placa (3.8 GB RAM) cargo siempre con `-j 2` — el paralelismo por
> defecto agota la RAM y congela el sistema.
