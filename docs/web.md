# Reelwright on the web

`apps/reelwright-web` runs the same engine and egui UI as the desktop app in the browser: the
workspace compiled to `wasm32-unknown-unknown`, started by eframe's web runner on WebGPU (WebGL2
fallback). It is a static site: one `.wasm`, its `wasm-bindgen` JavaScript glue, `index.html`, an
`AudioWorklet` script and an icon. Nothing is uploaded anywhere; media stay on the user's machine.

## Build and run

| Tool | Version | Install |
|---|---|---|
| Rust target | `wasm32-unknown-unknown` | `rustup target add wasm32-unknown-unknown` |
| wasm-bindgen CLI | exactly the `wasm-bindgen` crate version (0.2.129) | `cargo install wasm-bindgen-cli --version 0.2.129 --locked` |
| wasm-opt (optional) | any recent binaryen | `brew install binaryen`; used by `cargo xtask web` when found |

```sh
cargo xtask web                 # size-optimised `wasm-release` build → target/web/dist (index.html, reelwright_web.js, reelwright_web_bg.wasm, …)
cargo xtask web --dev           # unoptimised build (faster to compile, slow to run)
cargo xtask web --serve 8765    # build, then serve dist on http://127.0.0.1:8765/
```

`--serve` is a tiny localhost-only static server that sends `Cross-Origin-Opener-Policy` /
`Cross-Origin-Embedder-Policy`, so the page is cross-origin isolated like a production deployment
should be. Any static server works (`python3 -m http.server -d target/web/dist 8765`); it must
serve `.wasm` as `application/wasm` for streaming compilation.

The web crate is empty on non-wasm targets, so `cargo test --workspace` and clippy are
unaffected; `cargo xtask wasm` (part of `cargo xtask ci`) checks it, with `reelwright-ui-egui`, for
`wasm32-unknown-unknown`.

URL flags: `?empty` (no demo project), `?norecover` (don't reopen the auto-saved project),
`?fresh` (also don't restore media kept in OPFS), `?open=samples/name.fcproj` (open a project
that sits next to the page, instead of the demo project; same-origin relative paths only), `?cpu` (CPU compositor), `?webgl` (WebGL2 only;
the page reloads with it when WebGPU is present but fails to start), `?nowebcodecs` (decode with
Reelwright's own decoders only).

If the app panics after start-up (or runs out of memory), `index.html` shows a "Reelwright stopped
working" overlay with the panic message and a Reload button instead of a frozen canvas, and sets
`window.reelwrightLoad.fatal`. Code the web build runs must not call `std::env::temp_dir`,
`std::time::{Instant, SystemTime}::now`, `std::thread::sleep`/`spawn` or `std::process::id`: they
panic on `wasm32-unknown-unknown` (use `reelwright_engine::temp_dir`, `web_time`, or a `cfg`).

## Sample projects

`apps/reelwright-web/web/samples/*.fcproj` are copied next to the page by `cargo xtask web` and
open with `?open=samples/<name>.fcproj`. They are made only from the generators that are built
into the editor (demo scenes, bars, text, captions, markers), so they need no media files. Each is
a few lines of `reelwright-cli` script in `apps/reelwright-web/samples-src/`; to rebuild one:

```sh
reelwright-cli --demo --save-as apps/reelwright-web/web/samples/caption-check.fcproj \
  run apps/reelwright-web/samples-src/caption-check.jsonl
```

`demo-cut.fcproj` is the built-in demo project saved as it is.

## How the desktop pieces map to the browser

| Desktop | Web |
|---|---|
| frame worker threads | `FrameServer::pump`: queued frame jobs run on the UI thread between egui frames, highest priority first, within a time budget (24 ms while playing, 40 ms otherwise) |
| export on a worker thread | `reelwright_export::Exporter`: the export advances one frame per step from `Session::pump_jobs`, 30 ms per UI frame; progress shows in the header like on the desktop |
| `std::fs` (`FsServices`) | `fs::WebServices`: a virtual file table. Picked/dropped files are `File` handles (never copied); files the app writes are kept in memory and offered as downloads |
| file dialogs | File System Access `showOpenFilePicker` where available, else `<input type=file>`; drop files anywhere on the page |
| crash-recovery journal | OPFS: `recovery/snapshot.fcproj` every 5 s while there are unsaved changes, plus copies of imported media in `media/`; the next visit reopens the snapshot |
| cpal output | WebAudio `AudioWorklet` |
| VideoToolbox & co. | WebCodecs `VideoDecoder` |
| TCP control channel / MCP | `window.reelwright` (below) |

### Threads

wasm threads need a cross-origin isolated page **and** a wasm build with atomics (a nightly
`build-std` toolchain), so this build is single-threaded everywhere: frame rendering, decoding,
mixing and encoding are cooperative on the UI thread, and `rayon` runs inline.
`reelwright.info()` reports `crossOriginIsolated` so a threaded build can choose at start-up.
Commands that need a thread on the desktop (render previews, proxies, Project Manager, mask
tracking) report an error on the web for now.

### Media reads

Containers are opened through `Services::reader` (a `reelwright_media::ByteReader`): MP4/MOV and
Matroska read their index at import and single samples afterwards, so a multi-gigabyte file is
never loaded whole. Browser `Blob` reads are asynchronous, so `BlobReader` keeps a 384 MiB LRU cache
of 1 MiB chunks: a read whose chunks are missing starts fetching them (plus three chunks of
read-ahead) and fails with `WouldBlock`, setting `reelwright_media::pending`. The frame server then
drops that frame's result and retries the job after the bytes arrive (the fetch wakes the UI);
the stepped exporter re-renders the frame; an import retries `file.import`. Before importing, the
container index is prefetched (an MP4's `moov` box wherever it is, a small Matroska file whole).
Stills, WAV and compressed-audio files are decoded whole, as on the desktop.

Limits: Matroska/WebM files larger than the chunk cache import slowly (opening scans every
cluster header); a file the user picked is only readable while the page is open, unless its OPFS
copy exists (files up to 4 GiB are copied in the background).

### Decoding

Reelwright's own decoders (H.264, HEVC, VP9, AV1, ProRes, DNxHR, MJPEG, AAC, Opus…) run in the
browser unchanged. When the browser has WebCodecs, H.264, HEVC, VP9 and AV1 video in MP4/MOV is
decoded by the browser's (usually hardware) `VideoDecoder` instead: `webcodecs::reader_opener` is
registered ahead of the built-in openers and opens such files as a `WcSource`, whose audio and
media info come from our `Mp4Source`. WebCodecs is asynchronous, so a frame request feeds samples
to the decoder and returns "pending"; the decoder's output callback converts the `VideoFrame` to
RGBA (drawn on an `OffscreenCanvas`) into a per-source frame cache, and the retried request finds
it. Codec support is probed at start-up with `VideoDecoder.isConfigSupported`; any decoder error
switches that source to our decoder.

### Audio

`audio::Handle` implements `AudioOut`. The UI thread mixes the sequence 250 ms ahead in 2048-frame
blocks and posts them to the worklet (`web/audio-worklet.js`), which counts the frames it really
played and reports them with its `currentTime`. `played_frames` extrapolates from the last report
on the audio context's clock, never past what was queued — so the audio clock is the playback
master as on the desktop, and a starved worklet holds the playhead. Browsers start audio suspended
until the first click or key press; until then playback runs on the wall clock.

### GPU

eframe creates a WebGPU device when the browser has one, else WebGL2. Reelwright's GPU compositor
(`reelwright-gpu`) is used on WebGPU; on WebGL2 frames are composited on the CPU. Check
`reelwright.info().backend` and `.compositor`.

## `window.reelwright`: the agent / test API

The control channel of the desktop app (`docs/control-protocol.md`) as promises. Results resolve
with the method's `result`, errors reject.

| Call | |
|---|---|
| `reelwright.execute(command, params)` | run an engine command (same ids and params as MCP / `reelwright-cli`) |
| `reelwright.request(method, params)` | any control method: `ui.inspect`, `ui.click`, `ui.key`, `ui.playback`, `ui.menu.invoke`… |
| `reelwright.commands()` | the command list |
| `reelwright.inspect()` | `ui.inspect` |
| `reelwright.screenshot(params)` | `{pngBase64, width, height}` of the canvas (or `{panel}`) |
| `reelwright.importFiles(files)` | import `File` / `FileList` / `File[]`; resolves `{items, errors, paths}` |
| `reelwright.importUrl(url, name?)` | fetch a URL and import it (tests) |
| `reelwright.openProject(fileOrUrl)` | open a `.fcproj` |
| `reelwright.files()` | the virtual files: `{path, size, kind}` |
| `reelwright.readFile(path)` | a file's bytes (`Uint8Array`), e.g. an export |
| `reelwright.info()` | backend, compositor, isolation, OPFS, WebCodecs support and counters, start-up time |

`window.reelwrightLoad` holds `{wasmMs, readyMs}` (module fetch+compile, and until the app runs).

```js
await reelwright.importUrl("/clip.mp4");
await reelwright.execute("file.newSequence", {fromItem: 48});
await reelwright.request("ui.playback", {action: "play"});
const {job} = await reelwright.execute("file.exportMedia", {format: "h264", path: "/exports/out.mp4"});
```

## Browser test

`apps/reelwright-web/tests/smoke.mjs` drives headless Chrome over the DevTools protocol (Node ≥ 22,
no npm packages): load, demo playback, import of a generated MP4, playback, H.264 export
(download). It writes screenshots and `report.json`:

```sh
cargo xtask web --serve 8765 &
ffmpeg -f lavfi -i testsrc2=size=640x360:rate=30 -f lavfi -i sine -t 3 -c:v libx264 -pix_fmt yuv420p -c:a aac target/web/dist/web-test.mp4
node apps/reelwright-web/tests/smoke.mjs --url http://127.0.0.1:8765/ --media web-test.mp4 --out target/web/smoke
```
