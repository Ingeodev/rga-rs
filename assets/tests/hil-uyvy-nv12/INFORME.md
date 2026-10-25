# INFORME — HIL: UYVY→NV12 con dma-buf reales (rga-rs)

**Fecha:** 2026-10-25 · **Placa:** Orange Pi 5 Pro (RK3588S), Armbian 13 aarch64
**Objeto:** cerrar el **gap 3** del §4.1 del PLAN de `afora-live-svc-rs`
(gate de entrada de F6a): evidenciar la conversión `UYVY_422 → NV12` de
`rga-rs` con **dma-buf reales de la cámara** (`import_dma_fd`), el camino
exacto que usará F6a.
**Código:** `spikes/hil-uyvy-nv12/` (prototipo desechable). **Log:** `run-2026-10-25.log`.

## Procedimiento

1. V4L2 mplane UYVY 1920×1080 sobre rkcif (`/dev/video0`): `S_FMT` aplicado
   (stride=3840, sizeimage=4147200) → `REQBUFS(MMAP,2)` → `VIDIOC_EXPBUF`
   (plane 0) → **2 fds dma-buf, ambos `exp_name: videobuf2_cma_sg`**.
   El slot 0 se encola (frame real); el slot 1 queda **sin encolar** (memoria
   propia usada como dst de RGA).
2. `DQBUF` de un frame real (seq=0). Coherencia CPU↔dma-buf por mmap: ya
   verificada en el spike F5a del adaptador (checksum MATCH).
3. **Conversión A (src fd → dst virtual):** `RgaBuffer::import_dma_fd`
   (UYVY) + dst `import_virtual` (NV12) + `convert_color_with_mode`
   (`YuvBt601LimitRange`) → verificación contra **referencia CPU**
   (conversión UYVY→NV12 BT601 limit escrita en el spike) sobre parche 64×64.
4. **Conversión B (src fd → dst dma-buf):** mismo src, dst =
   `import_dma_fd` del slot 1 → el camino **fd→fd** exacto de F6a (RGA
   escribe el dma-buf que leerá MPP). Determinismo: checksum de la región
   escrita (wstride·hstride·1.5) igual que A.

## Resultados (ver log adjunto)

| Verificación | Resultado |
|---|---|
| `import_dma_fd` (src UYVY, dma-buf `videobuf2_cma_sg`) | ✅ OK |
| Conversión fd→virtual | ✅ OK, planos Y/UV no-triviales |
| Referencia CPU BT601 limit (parche 64×64) | **Y max\|Δ\|=0 · UV filas pares max\|Δ\|=0** (coincidencia exacta) |
| UV filas impares (croma decimado por RGA) | max\|Δ\|=2 — RGA hace **resampling real 4:2:2→4:2:0** (no simple decimado); documentado |
| Conversión fd→fd (dst dma-buf) | ✅ OK |
| Determinismo fd→fd vs fd→virtual | ✅ checksums idénticos |
| Limpieza (STREAMOFF + REQBUFS 0 + munmap) | ✅ sin errores |

**VEREDICTO: PASS — gap 3 CERRADO.** La API de rga-rs
(`import_dma_fd` + `convert_color_with_mode`) es correcta con dma-buf de la
cámara, en ambos modos de dst (virtual y dma-buf). Dato útil para F6a: la
decimación de croma que aplica RGA en filas impares no es copia literal, por
lo que los checksums de F6 deben comparar contra la **misma salida RGA**
(no contra una referencia CPU, que solo coincide en filas pares).

## Requisitos del entorno

- Usuario en grupo `video` (`/dev/video0`, `/dev/rga`) — sin sudo.
- Adapter de cámara **detenido** (nodo de captura libre).
- Build: `cargo build -j 2` (regla de placa, ver README de la raíz).
