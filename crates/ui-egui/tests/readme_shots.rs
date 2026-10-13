//! Renders the screenshots of the README and the project page from the sample projects
//! (`apps/reelwright-web/web/samples`), with the UI exactly as the app draws it: the Edit
//! Statistics and Caption Readability reports, and the demo cut.
//!
//! Off unless `REELWRIGHT_UI_SNAPSHOT_DIR` names a folder (a normal test run only checks that the
//! samples open and the reports fill in):
//!
//! ```sh
//! REELWRIGHT_UI_SNAPSHOT_DIR=target/shots cargo test -p reelwright-ui-egui --test readme_shots -- --nocapture
//! ```

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};

use egui_kittest::Harness;
use reelwright_engine::Session;
use reelwright_ui_egui::ReelwrightApp;
use reelwright_ui_egui::control::ControlRequest;
use serde_json::{Value, json};

struct Driver {
    harness: Harness<'static, ReelwrightApp>,
    tx: Sender<ControlRequest>,
}

impl Driver {
    fn new(sample: &str, seconds: f64, render: bool) -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/reelwright-web/web/samples").join(sample);
        let mut s = Session::default();
        s.execute("file.open", json!({"path": path.to_string_lossy()})).unwrap();
        s.execute("playhead.set", json!({"seconds": seconds})).unwrap();
        let (tx, rx) = channel();
        let app = ReelwrightApp::new(s).with_control(rx);
        let mut b = Harness::builder().with_size(egui::vec2(1600.0, 1000.0)).with_max_steps(10_000);
        if render {
            b = b.wgpu();
        }
        let mut d = Driver { harness: b.build_eframe(move |_cc| app), tx };
        d.frames(6);
        d
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let ctx = self.harness.ctx.clone();
            let mut raw = std::mem::take(self.harness.input_mut());
            eframe::App::raw_input_hook(self.harness.state_mut(), &ctx, &mut raw);
            *self.harness.input_mut() = raw;
            self.harness.step();
        }
    }

    fn menu(&mut self, id: &str) -> Value {
        let (req, reply) = ControlRequest::new("ui.menu.invoke", json!({"id": id}));
        self.tx.send(req).unwrap();
        for _ in 0..600 {
            self.frames(1);
            if let Ok(v) = reply.try_recv() {
                assert_eq!(v["ok"], json!(true), "{id}: {v}");
                self.frames(6);
                return v["result"].clone();
            }
        }
        panic!("no reply to {id}");
    }

    fn save(&mut self, dir: &Path, name: &str) {
        self.frames(4);
        let img = self.harness.render().expect("render");
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join(format!("{name}.png"));
        img.save(&path).unwrap();
        eprintln!("snapshot: {}", path.display());
    }
}

fn out_dir() -> Option<PathBuf> {
    std::env::var_os("REELWRIGHT_UI_SNAPSHOT_DIR").map(PathBuf::from)
}

#[test]
fn edit_statistics_report_of_the_pace_study() {
    let dir = out_dir();
    let mut d = Driver::new("pace-study.fcproj", 7.0, dir.is_some());
    let r = d.menu("sequence.editStats");
    assert_eq!(r["dialog"], "editStats");
    assert!(r["report"]["shots"].as_u64().unwrap() >= 10, "{r}");
    if let Some(dir) = dir {
        d.save(&dir, "reelwright-edit-stats");
    }
}

#[test]
fn caption_readability_report_of_the_caption_sample() {
    let dir = out_dir();
    let mut d = Driver::new("caption-check.fcproj", 7.0, dir.is_some());
    let r = d.menu("captions.checkReadability");
    assert_eq!(r["dialog"], "captionCheck");
    assert_eq!(r["report"]["flagged"], 6, "{r}");
    if let Some(dir) = dir {
        d.save(&dir, "reelwright-caption-check");
    }
}

#[test]
fn the_demo_cut_and_the_chapters_sample_open() {
    let dir = out_dir();
    let mut d = Driver::new("demo-cut.fcproj", 6.0, dir.is_some());
    if let Some(dir) = dir {
        d.save(&dir, "reelwright-editor");
    }
    let c = Driver::new("chapters.fcproj", 0.0, false);
    let chapters =
        c.harness.state().session.active_sequence().map(|q| q.markers.iter().filter(|m| m.kind == reelwright_engine::project::MarkerKind::Chapter).count());
    assert_eq!(chapters, Some(3));
}
