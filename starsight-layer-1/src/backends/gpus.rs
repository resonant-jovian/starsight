//! GPU backends: native (`wgpu`) and web (WebGPU via `wasm-bindgen`).
//!
//! Status: stub. Native lands in 0.6.0 with the interactive feature; web
//! WebGPU lands in 0.10.0 with the WASM target. Feature flag: `gpu`.

// ── WgpuBackend ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.6.0): pub struct WgpuBackend {
//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     surface: wgpu::Surface<'static>,
//     config: wgpu::SurfaceConfiguration,
// }

// ── DrawBackend impl ─────────────────────────────────────────────────────────────────────────────
// TODO(0.6.0): impl DrawBackend for WgpuBackend { ... }

// ── WebWgpuBackend ───────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub struct WebWgpuBackend { ... }   // for WASM targets
