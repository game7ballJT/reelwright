//! Settings ▸ Playback ▸ Hardware decoding through the decoder registry and the media stack:
//! Off gives the software decoder (and no hardware frames), Auto the hardware one where the
//! system has it. Its own test binary: the switch is process-wide.

mod common;

use common::*;
use reelwright_media::FrameRequest;

#[test]
fn off_uses_the_software_decoder() {
    let ff = reelwright_testkit::require_ffmpeg!();
    let Some(path) = named(&ff, "hevc_main.mp4") else { return };
    let s = read_stream(&path);
    let available = reelwright_platform::register();
    assert_eq!(reelwright_platform::register(), available, "registering twice is harmless");

    reelwright_codecs::hw::set_hardware_decoding(false);
    let d = reelwright_codecs::make_video_decoder(&s.entry).unwrap();
    assert_eq!(d.name(), "Reelwright HEVC");
    let bytes: std::sync::Arc<[u8]> = std::fs::read(&path).unwrap().into();
    let (g0, h0) = (reelwright_codecs::gop_stats(), reelwright_codecs::hw::hw_stats());
    let src = reelwright_codecs::open_bytes("hevc_main.mp4", bytes.clone()).unwrap();
    let rate = src.info().frame_rate();
    for i in 0..30 {
        src.video_frame(FrameRequest::full(rate.tick_of(i))).unwrap();
    }
    assert!(reelwright_codecs::gop_stats().frames > g0.frames, "frames counted");
    assert_eq!(reelwright_codecs::hw::hw_stats().frames, h0.frames, "Off: no hardware frames");

    reelwright_codecs::hw::set_hardware_decoding(true);
    let d = reelwright_codecs::make_video_decoder(&s.entry).unwrap();
    let hw = reelwright_platform::hardware_decoder_for(&s.entry);
    if !hw {
        eprintln!("SKIPPED (Auto half): no hardware decoder ({available:?})");
        assert_eq!(d.name(), "Reelwright HEVC");
        return;
    }
    let (backend, hw_name) = if cfg!(target_os = "macos") { ("VideoToolbox", "VideoToolbox HEVC") } else { ("Media Foundation", "Media Foundation HEVC") };
    assert_eq!(d.name(), hw_name);
    assert_eq!(reelwright_codecs::hw::hw_backend(), Some(backend), "perf.stats reports the backend");
    assert!(reelwright_platform::registered(), "register() put the factory in the registry");
    let h1 = reelwright_codecs::hw::hw_stats();
    let src = reelwright_codecs::open_bytes("hevc_main.mp4", bytes).unwrap();
    for i in 0..30 {
        src.video_frame(FrameRequest::full(rate.tick_of(i))).unwrap();
    }
    let h2 = reelwright_codecs::hw::hw_stats();
    assert!(h2.frames > h1.frames && h2.sessions > h1.sessions, "Auto: hardware frames {h1:?} → {h2:?}");
}
