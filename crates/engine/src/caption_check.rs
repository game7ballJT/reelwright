//! Caption readability: Check Readability.
//!
//! | Id | Menu | Notes |
//! |---|---|---|
//! | `captions.checkReadability` | Sequence ▸ Captions ▸ Check Readability… | reading speed, line length, lines per caption, duration and the gap to the next caption, for one caption track or all of them |
//!
//! The defaults follow the limits that subtitle style guides for streaming services commonly ask
//! for (they are guidelines, not a standard, and every one of them can be changed per call):
//!
//! | Limit | Parameter | Default |
//! |---|---|---|
//! | reading speed, characters per second (spaces and punctuation count) | `maxCps` | 17 |
//! | characters in a line | `maxLine` | 42, or the track format's column limit (CEA-608: 32, Teletext: 40) |
//! | lines in a caption | `maxLines` | 2 |
//! | shortest caption, seconds | `minSeconds` | 5/6 |
//! | longest caption, seconds | `maxSeconds` | 7 |
//! | smallest gap between two captions, frames (touching captions are fine) | `minGapFrames` | 2 |
//!
//! The check only reports. With `select: true` the captions that have a problem become the caption
//! selection, so a following `captions.setTimes`, `captions.merge` or `captions.delete` acts on them.
//! Lengths come from the exact integer time of the captions; text is measured after markup (`<i>`,
//! `<b>`, entities) is removed, as it is drawn.

use reelwright_project::{Caption, CaptionTrack};
use reelwright_time::{Tick, TimeDisplay, format_time};
use serde_json::{Value, json};

use crate::captions::track_param;
use crate::commands::{CommandSpec, bad, bool_p, f64_p, has_seq, u64_p};
use crate::{EngineError, Result, Session};

pub(crate) fn commands() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: "captions.checkReadability",
        label: "Check Caption Readability…",
        menu: &["Sequence", "Captions"],
        shortcut: None,
        params: r#"{"track":id|"C1"?,"maxCps":f64=17,"maxLine":int?,"maxLines":int=2,"minSeconds":f64=0.833,"maxSeconds":f64=7,"minGapFrames":int=2,"select":bool=false} → {tracks, captions, ok, counts:{kind:n}, issues:[{track, trackName, caption, kind, message, start, seconds, in}], limits}"#,
        enabled: has_captions,
        run: check,
        journal: false,
    }]
}

fn has_captions(s: &Session) -> std::result::Result<(), String> {
    has_seq(s)?;
    if s.active_sequence().is_some_and(|q| q.caption_tracks.iter().any(|t| !t.captions.is_empty())) { Ok(()) } else { Err("there are no captions".into()) }
}

/// What a caption is checked against (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    pub max_cps: f64,
    pub max_line: usize,
    pub max_lines: usize,
    pub min_seconds: f64,
    pub max_seconds: f64,
    pub min_gap_frames: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { max_cps: 17.0, max_line: 42, max_lines: 2, min_seconds: 5.0 / 6.0, max_seconds: 7.0, min_gap_frames: 2 }
    }
}

/// The kind of problem found in a caption.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Empty,
    TooFast,
    LineTooLong,
    TooManyLines,
    TooShort,
    TooLong,
    TightGap,
}

impl Kind {
    pub const ALL: [Kind; 7] = [Kind::Empty, Kind::TooFast, Kind::LineTooLong, Kind::TooManyLines, Kind::TooShort, Kind::TooLong, Kind::TightGap];

    pub fn id(self) -> &'static str {
        match self {
            Kind::Empty => "empty",
            Kind::TooFast => "tooFast",
            Kind::LineTooLong => "lineTooLong",
            Kind::TooManyLines => "tooManyLines",
            Kind::TooShort => "tooShort",
            Kind::TooLong => "tooLong",
            Kind::TightGap => "tightGap",
        }
    }
}

/// One problem found in one caption.
#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub caption: Caption,
    pub kind: Kind,
    pub message: String,
}

/// Every problem in `track`, in caption order. `frame` is the duration of one frame of the
/// sequence (a gap is measured in whole frames).
pub fn check_track(track: &CaptionTrack, lim: &Limits, frame: Tick) -> Vec<Issue> {
    let mut caps: Vec<&Caption> = track.captions.iter().collect();
    caps.sort_by_key(|c| c.start);
    let mut out = Vec::new();
    for (i, c) in caps.iter().enumerate() {
        let mut add = |kind: Kind, message: String| out.push(Issue { caption: (*c).clone(), kind, message });
        let lines: Vec<String> = c.plain_lines().iter().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        let chars: usize = lines.iter().map(|l| l.chars().count()).sum();
        let seconds = c.duration.seconds();
        if lines.is_empty() {
            add(Kind::Empty, "the caption has no text".into());
        }
        let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        if longest > lim.max_line {
            add(Kind::LineTooLong, format!("a line has {longest} characters (up to {} fit)", lim.max_line));
        }
        if lines.len() > lim.max_lines {
            add(Kind::TooManyLines, format!("{} lines (up to {})", lines.len(), lim.max_lines));
        }
        if seconds <= 0.0 {
            add(Kind::TooShort, "the caption has no duration".into());
        } else {
            if seconds < lim.min_seconds {
                add(Kind::TooShort, format!("{seconds:.2} s on screen (at least {:.2} s)", lim.min_seconds));
            }
            if seconds > lim.max_seconds {
                add(Kind::TooLong, format!("{seconds:.1} s on screen (up to {:.1} s)", lim.max_seconds));
            }
            let cps = chars as f64 / seconds;
            if chars > 0 && cps > lim.max_cps {
                add(Kind::TooFast, format!("{cps:.1} characters per second (up to {:.0})", lim.max_cps));
            }
        }
        if let Some(next) = caps.get(i + 1)
            && frame.0 > 0
        {
            let gap = next.start.0.saturating_sub(c.end().0);
            let needed = frame.0.saturating_mul(i64::try_from(lim.min_gap_frames).unwrap_or(i64::MAX));
            if gap > 0 && gap < needed {
                let frames = gap / frame.0;
                add(Kind::TightGap, format!("{frames} frame{} before the next caption (at least {})", if frames == 1 { "" } else { "s" }, lim.min_gap_frames));
            }
        }
    }
    out
}

/// A numeric parameter that must be a finite number above zero when given.
fn positive(p: &Value, key: &str) -> Result<Option<f64>> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => match f64_p(p, key) {
            Some(v) if v.is_finite() && v > 0.0 => Ok(Some(v)),
            _ => Err(bad("captions.checkReadability", format!("{key} must be a number above zero"))),
        },
    }
}

fn count_p(p: &Value, key: &str) -> Result<Option<usize>> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => match u64_p(p, key).filter(|n| *n > 0 && p[key].as_f64().is_some_and(|f| f.fract() == 0.0)) {
            Some(n) => Ok(Some(usize::try_from(n).unwrap_or(usize::MAX))),
            None => Err(bad("captions.checkReadability", format!("{key} must be a whole number above zero"))),
        },
    }
}

fn check(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "captions.checkReadability";
    let mut lim = Limits::default();
    if let Some(v) = positive(p, "maxCps")? {
        lim.max_cps = v;
    }
    if let Some(v) = positive(p, "minSeconds")? {
        lim.min_seconds = v;
    }
    if let Some(v) = positive(p, "maxSeconds")? {
        lim.max_seconds = v;
    }
    if let Some(v) = count_p(p, "maxLines")? {
        lim.max_lines = v;
    }
    if let Some(v) = count_p(p, "minGapFrames")? {
        lim.min_gap_frames = u64::try_from(v).unwrap_or(u64::MAX);
    }
    let max_line = count_p(p, "maxLine")?;
    if lim.min_seconds > lim.max_seconds {
        return Err(bad(C, "minSeconds is more than maxSeconds"));
    }
    let q = s.active_sequence().ok_or(EngineError::NoSequence)?;
    let tracks: Vec<&CaptionTrack> = if p.get("track").is_some_and(|t| !t.is_null()) {
        let id = track_param(s, p).ok_or_else(|| bad(C, "no such caption track"))?;
        q.caption_track(id).into_iter().collect()
    } else {
        q.caption_tracks.iter().collect()
    };
    let (rate, df) = (q.settings.frame_rate, q.settings.drop_frame);
    let frame = rate.frame_duration();
    let tc = |t: Tick| format_time(t, rate, df, TimeDisplay::Timecode, 48_000);
    let (mut issues, mut checked) = (Vec::new(), 0usize);
    let mut counts = serde_json::Map::new();
    for k in Kind::ALL {
        counts.insert(k.id().to_string(), json!(0));
    }
    let mut flagged: Vec<reelwright_project::ClipId> = Vec::new();
    for t in &tracks {
        checked += t.captions.len();
        // CEA-608 and Teletext have fixed columns; an explicit maxLine always wins
        let per_track = Limits { max_line: max_line.or_else(|| t.format.max_columns()).unwrap_or(lim.max_line), ..lim };
        for i in check_track(t, &per_track, frame) {
            if let Some(n) = counts.get_mut(i.kind.id()) {
                *n = json!(n.as_u64().unwrap_or(0) + 1);
            }
            if !flagged.contains(&i.caption.id) {
                flagged.push(i.caption.id);
            }
            issues.push(json!({
                "track": t.id.0,
                "trackName": t.name,
                "caption": i.caption.id.0,
                "kind": i.kind.id(),
                "message": i.message,
                "start": i.caption.start.0,
                "seconds": (i.caption.start.seconds() * 1000.0).round() / 1000.0,
                "in": tc(i.caption.start),
                "text": i.caption.text,
            }));
        }
    }
    let result = json!({
        "tracks": tracks.len(),
        "captions": checked,
        "flagged": flagged.len(),
        "ok": issues.is_empty(),
        "counts": counts,
        "issues": issues,
        "limits": {
            "maxCps": lim.max_cps,
            "maxLine": max_line.unwrap_or(lim.max_line),
            "maxLines": lim.max_lines,
            "minSeconds": lim.min_seconds,
            "maxSeconds": lim.max_seconds,
            "minGapFrames": lim.min_gap_frames,
        },
    });
    if bool_p(p, "select").unwrap_or(false) && !flagged.is_empty() {
        s.execute("captions.select", json!({"captions": flagged.iter().map(|c| c.0).collect::<Vec<_>>()}))?;
    }
    s.toast(if flagged.is_empty() {
        format!("{checked} captions checked: all comfortable to read")
    } else {
        format!("{checked} captions checked: {} need attention", flagged.len())
    });
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reelwright_project::{CaptionFormat, ClipId, TrackId};

    fn cap(id: u64, start: f64, seconds: f64, text: &str) -> Caption {
        Caption {
            id: ClipId(id),
            start: Tick::from_seconds_f64(start),
            duration: Tick::from_seconds_f64(seconds),
            text: text.to_string(),
            speaker: None,
            cue_id: None,
            settings: String::new(),
        }
    }

    fn track(caps: Vec<Caption>) -> CaptionTrack {
        let mut t = CaptionTrack::new(TrackId(1), "English".into(), CaptionFormat::Subtitle);
        t.captions = caps;
        t
    }

    fn frame() -> Tick {
        Tick::from_seconds_f64(1.0 / 24.0)
    }

    fn kinds(t: &CaptionTrack, lim: &Limits) -> Vec<(u64, Kind)> {
        check_track(t, lim, frame()).iter().map(|i| (i.caption.id.0, i.kind)).collect()
    }

    #[test]
    fn a_comfortable_caption_has_no_issues() {
        let t = track(vec![cap(1, 0.0, 2.0, "Houston, Tranquility Base here."), cap(2, 2.5, 2.0, "The Eagle has landed.")]);
        assert!(check_track(&t, &Limits::default(), frame()).is_empty());
    }

    #[test]
    fn each_limit_is_reported_for_the_caption_that_breaks_it() {
        let long = "x".repeat(50);
        let t = track(vec![
            cap(1, 0.0, 1.0, "This line is read much too quickly by anyone"),
            cap(2, 3.0, 4.0, &long),
            cap(3, 8.0, 3.0, "one\ntwo\nthree"),
            cap(4, 12.0, 0.5, "Hi"),
            cap(5, 14.0, 9.0, "Stays on screen"),
            cap(6, 24.0, 2.0, "  \n "),
        ]);
        let k = kinds(&t, &Limits::default());
        assert!(k.contains(&(1, Kind::TooFast)), "{k:?}");
        assert!(k.contains(&(2, Kind::LineTooLong)), "{k:?}");
        assert!(k.contains(&(3, Kind::TooManyLines)), "{k:?}");
        assert!(k.contains(&(4, Kind::TooShort)), "{k:?}");
        assert!(k.contains(&(5, Kind::TooLong)), "{k:?}");
        assert!(k.contains(&(6, Kind::Empty)), "{k:?}");
        assert!(!k.iter().any(|(id, kind)| *id == 4 && *kind == Kind::TooFast), "2 characters in half a second is 4 cps: {k:?}");
    }

    #[test]
    fn markup_is_not_counted_and_line_breaks_are_not_characters() {
        // 34 visible characters in 2 s = 17 cps exactly, which is allowed
        let t = track(vec![cap(1, 0.0, 2.0, "<i>Seventeen characters</i>\n&amp; fourteen more")]);
        let lim = Limits { max_cps: 17.0, ..Limits::default() };
        let chars = "Seventeen characters& fourteen more".chars().count();
        assert_eq!(chars, 35, "the test text changed");
        let k = kinds(&t, &lim);
        assert!(k.contains(&(1, Kind::TooFast)), "35 characters in 2 s is 17.5 cps: {k:?}");
        let t = track(vec![cap(1, 0.0, 2.0, "<i>Seventeen characters</i>\n& fourteen mor")]);
        assert!(kinds(&t, &lim).is_empty());
    }

    #[test]
    fn only_a_small_positive_gap_is_a_problem() {
        let f = 1.0 / 24.0;
        let t = track(vec![cap(1, 0.0, 2.0, "A"), cap(2, 2.0, 2.0, "B"), cap(3, 4.0 + f, 2.0, "C"), cap(4, 6.0 + f + 5.0 * f, 2.0, "D")]);
        let k = kinds(&t, &Limits::default());
        assert_eq!(k, vec![(2, Kind::TightGap)], "touching captions (1 to 2) and a 5-frame gap are fine; 1 frame is not: {k:?}");
    }

    #[test]
    fn zero_length_captions_never_divide_by_zero() {
        let t = track(vec![cap(1, 0.0, 0.0, "Instant")]);
        let k = kinds(&t, &Limits::default());
        assert_eq!(k, vec![(1, Kind::TooShort)]);
    }

    #[test]
    fn the_command_reports_counts_limits_and_can_select_the_problems() {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        assert!(s.execute("captions.checkReadability", json!({})).is_err(), "no captions yet");
        s.execute("captions.newTrack", json!({})).unwrap();
        let tid = s.active_sequence().unwrap().caption_tracks[0].id;
        s.edit_sequence("Add captions", |q, _, _| {
            let t = q.caption_tracks.iter_mut().find(|t| t.id == tid).unwrap();
            t.captions.push(cap(9001, 0.0, 3.0, "A calm, readable caption."));
            t.captions.push(cap(9002, 4.0, 1.0, "This one is read far too quickly to follow"));
            Ok(())
        })
        .unwrap();
        let r = s.execute("captions.checkReadability", json!({"select": true})).unwrap();
        assert_eq!((r["captions"].as_u64(), r["flagged"].as_u64(), r["ok"].as_bool()), (Some(2), Some(1), Some(false)), "{r}");
        assert_eq!(r["counts"]["tooFast"], 1, "{r}");
        assert_eq!(r["issues"][0]["caption"], 9002);
        assert_eq!(r["limits"]["maxLine"], 42);
        assert_eq!(s.state.caption_selection, vec![ClipId(9002)]);
        // looser limits: nothing to report
        let r = s.execute("captions.checkReadability", json!({"maxCps": 60, "track": "C1"})).unwrap();
        assert_eq!(r["ok"], true, "{r}");
        // bad parameters are refused, not clamped
        for bad_params in
            [json!({"maxCps": 0}), json!({"maxCps": -3}), json!({"maxLines": 1.5}), json!({"minSeconds": 9, "maxSeconds": 2}), json!({"track": 12345})]
        {
            assert!(s.execute("captions.checkReadability", bad_params.clone()).is_err(), "{bad_params}");
        }
    }

    #[test]
    fn a_608_track_uses_its_32_column_limit_unless_maxline_is_given() {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        s.execute("captions.newTrack", json!({"format": "CEA-608"})).unwrap();
        let tid = s.active_sequence().unwrap().caption_tracks[0].id;
        let line = "y".repeat(36);
        s.edit_sequence("Add caption", |q, _, _| {
            let t = q.caption_tracks.iter_mut().find(|t| t.id == tid).unwrap();
            t.captions.push(cap(9100, 0.0, 6.0, &line));
            Ok(())
        })
        .unwrap();
        let r = s.execute("captions.checkReadability", json!({})).unwrap();
        assert_eq!(r["counts"]["lineTooLong"], 1, "{r}");
        let r = s.execute("captions.checkReadability", json!({"maxLine": 40})).unwrap();
        assert_eq!(r["counts"]["lineTooLong"], 0, "{r}");
    }
}
