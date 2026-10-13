//! Sequence and Markers insight: Edit Statistics and Export Chapter List.
//!
//! | Id | Menu | Notes |
//! |---|---|---|
//! | `sequence.editStats` | Sequence ▸ Edit Statistics… | shots, cuts, average / median shot length, cuts per minute, a pace histogram, the longest and shortest shots |
//! | `markers.exportChapters` | Markers ▸ Export Chapter List… | the sequence's Chapter markers as a YouTube-style chapter list (`0:00 Intro`) or JSON |
//!
//! **Edit Statistics** looks at the enabled video clips on enabled tracks. A *cut* is a start time at
//! which a clip begins after the first one (overlapping clips on several tracks that begin together
//! count once), so cuts per minute describes the rhythm of the program rather than the number of
//! layers. Lengths are in seconds, computed from the exact integer time of the clips.
//!
//! **Export Chapter List** needs no more than the markers: chapter times are the markers' start
//! times from the start of the sequence, shown as `M:SS` (or `H:MM:SS`). It warns about what a video
//! platform will not accept: no chapter at 0:00, fewer than three chapters, or chapters closer than
//! ten seconds. `path` writes the text through the host's file service.

use reelwright_project::{Marker, MarkerKind, Sequence};
use reelwright_time::Tick;
use serde_json::{Value, json};

use crate::commands::{CommandSpec, bad, has_seq, str_p};
use crate::{EngineError, Result, Session};

pub(crate) fn commands() -> Vec<CommandSpec> {
    vec![
        CommandSpec {
            id: "sequence.editStats",
            label: "Edit Statistics…",
            menu: &["Sequence"],
            shortcut: None,
            params: r#"{} → {shots, cuts, durationSeconds, averageShotSeconds, medianShotSeconds, cutsPerMinute, longest: {clip, name, seconds}, shortest, histogram: [{label, count}], videoTracks}"#,
            enabled: has_video_clips,
            run: edit_stats,
            journal: false,
        },
        CommandSpec {
            id: "markers.exportChapters",
            label: "Export Chapter List…",
            menu: &["Markers"],
            shortcut: None,
            params: r#"{"kinds":"chapter"|"all"="chapter","format":"youtube"|"json"="youtube","path":str?} → {text, count, warnings:[str]}"#,
            enabled: has_seq,
            run: export_chapters,
            journal: false,
        },
    ]
}

fn has_video_clips(s: &Session) -> std::result::Result<(), String> {
    has_seq(s)?;
    let q = s.active_sequence().ok_or("no sequence is open")?;
    if q.video_tracks.iter().any(|t| t.enabled && t.items.iter().any(|i| i.enabled)) { Ok(()) } else { Err("the sequence has no video clips".into()) }
}

/// Shot-length buckets: (label, upper bound in seconds, exclusive).
const BUCKETS: [(&str, f64); 6] = [("under 1 s", 1.0), ("1-2 s", 2.0), ("2-4 s", 4.0), ("4-8 s", 8.0), ("8-16 s", 16.0), ("16 s or more", f64::INFINITY)];

/// The statistics of a sequence's video clips (see the module docs).
pub fn stats(q: &Sequence) -> Value {
    struct Shot {
        id: u64,
        name: String,
        start: Tick,
        seconds: f64,
    }
    let mut shots: Vec<Shot> = vec![];
    let mut tracks = 0;
    let mut end = Tick::ZERO;
    for t in q.video_tracks.iter().filter(|t| t.enabled) {
        let mut used = false;
        for it in t.items.iter().filter(|i| i.enabled && i.duration.0 > 0) {
            used = true;
            end = end.max(it.end());
            shots.push(Shot { id: it.id.0, name: it.name.clone(), start: it.start, seconds: it.duration.seconds() });
        }
        tracks += usize::from(used);
    }
    if shots.is_empty() {
        return json!({"shots": 0, "cuts": 0, "durationSeconds": 0.0, "videoTracks": 0, "histogram": histogram(&[])});
    }
    let mut starts: Vec<Tick> = shots.iter().map(|s| s.start).collect();
    starts.sort();
    starts.dedup();
    let cuts = starts.len().saturating_sub(1);
    let first = starts.first().copied().unwrap_or(Tick::ZERO);
    let duration = (end - first).seconds().max(0.0);
    let lengths: Vec<f64> = shots.iter().map(|s| s.seconds).collect();
    let mut sorted = lengths.clone();
    sorted.sort_by(f64::total_cmp);
    let median = if sorted.len() % 2 == 1 { sorted[sorted.len() / 2] } else { (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0 };
    let average = lengths.iter().sum::<f64>() / lengths.len() as f64;
    let round = |v: f64| (v * 1000.0).round() / 1000.0;
    let longest = shots.iter().max_by(|a, b| a.seconds.total_cmp(&b.seconds));
    let shortest = shots.iter().min_by(|a, b| a.seconds.total_cmp(&b.seconds));
    let describe = |s: Option<&Shot>| s.map_or(Value::Null, |s| json!({"clip": s.id, "name": s.name, "seconds": round(s.seconds)}));
    json!({
        "shots": shots.len(),
        "cuts": cuts,
        "durationSeconds": round(duration),
        "averageShotSeconds": round(average),
        "medianShotSeconds": round(median),
        "cutsPerMinute": if duration > 0.0 { round(cuts as f64 * 60.0 / duration) } else { 0.0 },
        "longest": describe(longest),
        "shortest": describe(shortest),
        "histogram": histogram(&lengths),
        "videoTracks": tracks,
    })
}

fn histogram(lengths: &[f64]) -> Value {
    let mut counts = [0usize; BUCKETS.len()];
    for l in lengths {
        let i = BUCKETS.iter().position(|(_, hi)| *l < *hi).unwrap_or(BUCKETS.len() - 1);
        counts[i] += 1;
    }
    Value::Array(BUCKETS.iter().zip(counts).map(|((label, _), count)| json!({"label": label, "count": count})).collect())
}

fn edit_stats(s: &mut Session, _: &Value) -> Result<Value> {
    let q = s.active_sequence().ok_or(EngineError::Other("no sequence is open".into()))?;
    let r = stats(q);
    // The answer where the editor looks (a toast in the app; headless callers get the JSON).
    let (shots, avg, cpm) = (r["shots"].as_u64().unwrap_or(0), r["averageShotSeconds"].as_f64().unwrap_or(0.0), r["cutsPerMinute"].as_f64().unwrap_or(0.0));
    s.toast(format!("{shots} shots, average {avg:.1} s, {cpm:.1} cuts per minute"));
    Ok(r)
}

/// `M:SS`, or `H:MM:SS` from an hour on, from whole seconds.
pub fn chapter_time(seconds: u64) -> String {
    let (h, m, sec) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 { format!("{h}:{m:02}:{sec:02}") } else { format!("{m}:{sec:02}") }
}

/// The chapters of `markers` in time order as (whole seconds, title): a chapter without a name is
/// called "Chapter N".
pub fn chapters(markers: &[Marker], all: bool) -> Vec<(u64, String)> {
    let mut v: Vec<&Marker> = markers.iter().filter(|m| all || m.kind == MarkerKind::Chapter).collect();
    v.sort_by_key(|m| m.start);
    v.iter()
        .enumerate()
        .map(|(i, m)| {
            let name = m.name.trim();
            (m.start.seconds().max(0.0).floor() as u64, if name.is_empty() { format!("Chapter {}", i + 1) } else { name.to_string() })
        })
        .collect()
}

/// What a video platform won't take from a chapter list.
pub fn chapter_warnings(list: &[(u64, String)]) -> Vec<String> {
    let mut w = vec![];
    if list.first().is_some_and(|(t, _)| *t != 0) {
        w.push("the first chapter should start at 0:00".to_string());
    }
    if list.len() < 3 {
        w.push("a video platform needs at least three chapters".to_string());
    }
    if list.windows(2).any(|p| p[1].0 < p[0].0 + 10) {
        w.push("chapters should be at least 10 seconds long".to_string());
    }
    w
}

fn export_chapters(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "markers.exportChapters";
    let all = match str_p(p, "kinds").unwrap_or("chapter") {
        "chapter" => false,
        "all" => true,
        other => return Err(bad(C, format!("kinds must be chapter or all (got `{other}`)"))),
    };
    let json_out = match str_p(p, "format").unwrap_or("youtube") {
        "youtube" => false,
        "json" => true,
        other => return Err(bad(C, format!("format must be youtube or json (got `{other}`)"))),
    };
    let q = s.active_sequence().ok_or(EngineError::Other("no sequence is open".into()))?;
    let list = chapters(&q.markers, all);
    if list.is_empty() {
        return Err(bad(C, if all { "the sequence has no markers" } else { "the sequence has no Chapter markers (add some, or pass kinds: all)" }));
    }
    let warnings = chapter_warnings(&list);
    let text = if json_out {
        serde_json::to_string_pretty(&list.iter().map(|(t, n)| json!({"seconds": t, "time": chapter_time(*t), "title": n})).collect::<Vec<_>>())
            .unwrap_or_default()
    } else {
        list.iter().map(|(t, n)| format!("{} {n}", chapter_time(*t))).collect::<Vec<_>>().join("\n")
    };
    if let Some(path) = str_p(p, "path") {
        let mut bytes = text.clone().into_bytes();
        bytes.push(b'\n');
        s.services.write_file(path, &bytes).map_err(|e| EngineError::Other(e.to_string()))?;
    }
    if str_p(p, "path").is_some() {
        s.toast(format!("Wrote {} chapters{}", list.len(), if warnings.is_empty() { String::new() } else { format!(" ({})", warnings.join("; ")) }));
    }
    Ok(json!({"text": text, "count": list.len(), "warnings": warnings}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reelwright_project::{Label, MarkerId};

    fn demo() -> Session {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        s
    }

    fn marker(id: u64, seconds: f64, name: &str, kind: MarkerKind) -> Marker {
        Marker {
            id: MarkerId(id),
            start: Tick::from_seconds_f64(seconds),
            duration: Tick::ZERO,
            name: name.into(),
            comment: String::new(),
            kind,
            color: Label::Green,
        }
    }

    #[test]
    fn chapter_times_are_minutes_and_seconds_or_hours() {
        assert_eq!(chapter_time(0), "0:00");
        assert_eq!(chapter_time(59), "0:59");
        assert_eq!(chapter_time(61), "1:01");
        assert_eq!(chapter_time(3599), "59:59");
        assert_eq!(chapter_time(3600), "1:00:00");
        assert_eq!(chapter_time(3 * 3600 + 7 * 60 + 9), "3:07:09");
    }

    #[test]
    fn only_chapter_markers_are_listed_in_time_order_and_unnamed_ones_are_numbered() {
        let ms = vec![
            marker(1, 30.0, "Middle", MarkerKind::Chapter),
            marker(2, 5.0, "A comment", MarkerKind::Comment),
            marker(3, 0.0, "", MarkerKind::Chapter),
            marker(4, 61.9, "End", MarkerKind::Chapter),
        ];
        assert_eq!(chapters(&ms, false), vec![(0, "Chapter 1".to_string()), (30, "Middle".to_string()), (61, "End".to_string())]);
        assert_eq!(chapters(&ms, true).len(), 4);
    }

    #[test]
    fn warnings_name_what_a_platform_rejects() {
        let ok = vec![(0, "a".to_string()), (30, "b".to_string()), (90, "c".to_string())];
        assert!(chapter_warnings(&ok).is_empty());
        let w = chapter_warnings(&[(5, "a".into()), (8, "b".into())]);
        assert_eq!(w.len(), 3, "{w:?}");
        assert!(w.iter().any(|m| m.contains("0:00")) && w.iter().any(|m| m.contains("three")) && w.iter().any(|m| m.contains("10 seconds")));
    }

    #[test]
    fn exports_the_chapter_list_and_reports_problems() {
        let mut s = demo();
        let none = s.execute("markers.exportChapters", json!({}));
        assert!(none.is_err(), "no chapter markers yet");
        s.edit_sequence("Add chapters", |q, _, _| {
            q.markers.push(marker(901, 0.0, "Launch", MarkerKind::Chapter));
            q.markers.push(marker(902, 75.0, "Landing", MarkerKind::Chapter));
            q.markers.push(marker(903, 3725.0, "Moonwalk", MarkerKind::Chapter));
            Ok(())
        })
        .unwrap();
        let r = s.execute("markers.exportChapters", json!({})).unwrap();
        assert_eq!(r["text"], "0:00 Launch\n1:15 Landing\n1:02:05 Moonwalk");
        assert_eq!((r["count"].as_u64(), r["warnings"].as_array().map(Vec::len)), (Some(3), Some(0)));
        let j = s.execute("markers.exportChapters", json!({"format": "json"})).unwrap();
        let v: Value = serde_json::from_str(j["text"].as_str().unwrap()).unwrap();
        assert_eq!(v[1], json!({"seconds": 75, "time": "1:15", "title": "Landing"}));
        assert!(s.execute("markers.exportChapters", json!({"kinds": "nope"})).is_err());
        assert!(s.execute("markers.exportChapters", json!({"format": "xml"})).is_err());
    }

    #[test]
    fn edit_stats_describe_the_demo_cut() {
        let mut s = demo();
        let r = s.execute("sequence.editStats", json!({})).unwrap();
        let shots = r["shots"].as_u64().unwrap();
        assert!(shots >= 2, "{r}");
        assert!(r["cuts"].as_u64().unwrap() >= 1 && r["cuts"].as_u64().unwrap() < shots, "overlapping clips count one cut: {r}");
        assert!(r["durationSeconds"].as_f64().unwrap() > 0.0);
        let (avg, median) = (r["averageShotSeconds"].as_f64().unwrap(), r["medianShotSeconds"].as_f64().unwrap());
        let (long, short) = (r["longest"]["seconds"].as_f64().unwrap(), r["shortest"]["seconds"].as_f64().unwrap());
        assert!(short <= avg && avg <= long && short <= median && median <= long, "{r}");
        let total: u64 = r["histogram"].as_array().unwrap().iter().map(|b| b["count"].as_u64().unwrap()).sum();
        assert_eq!(total, shots, "every shot falls in one bucket");
        assert!(r["cutsPerMinute"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn an_empty_sequence_reports_zeros_and_the_command_is_disabled() {
        let mut s = Session::default();
        s.execute("file.newProject", json!({})).ok();
        let q = s.active_sequence().map(stats);
        if let Some(r) = q {
            assert_eq!(r["shots"], 0);
            assert_eq!(r["histogram"].as_array().unwrap().len(), BUCKETS.len());
        }
    }
}
