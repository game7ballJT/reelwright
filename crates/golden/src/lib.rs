//! Golden-image tests for Reelwright's renderer (test-only crate; the library is empty).
//!
//! `tests/golden.rs` builds small procedural projects (no media files), renders one frame of each
//! through the CPU reference compositor (`reelwright-render`) and compares it with a committed PNG
//! in `goldens/` using `reelwright_testkit::golden`. The same scenes are composited on the GPU
//! (`reelwright-gpu`) when an adapter is available and compared with the CPU result.
//!
//! Regenerate references with `REELWRIGHT_BLESS=1 cargo test -p reelwright-golden`.
