//! Streaming data: append new points and re-render efficiently.
//!
//! Status: stub. Implementation lands in 0.6.0 behind the `interactive` feature.

// ── StreamingData ────────────────────────────────────────────────────────────────────────────────
// TODO(0.6.0): pub struct StreamingData<T> { buffer: VecDeque<T>, capacity: usize, channel: DataChannel<T> }

// ── DataChannel ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.6.0): pub struct DataChannel<T> { sender: Sender<T>, receiver: Receiver<T> }
