# Background image adjustments

Import an ordinary image using **Appearance → Current theme → + → Select an image → Create theme**. The **Background image** section appears alongside **Dither** in the wgpu build. No shader metadata is needed.

| Control | Range | Default |
| --- | --- | --- |
| Image opacity | 0–100% | Theme opacity (30% for newly imported themes) |
| Brightness | 0–200% | 100% |
| Contrast | 0–200% | 100% |
| Bottom darkening | On/off | Off |
| Bottom darkening strength | 0–100% | 60% |
| Gradient start from top | 0–100% | 50% |
| Vignette | On/off | Off |
| Vignette strength | 0–100% | 40% |

The top retains the adjusted image brightness. Darkening starts at the selected window position and smoothly increases toward the bottom. A start of 100% disables darkening. The vignette leaves the center unchanged and softly darkens an ellipse around the window edges.

**Restore defaults** resets this section, including returning opacity to the active theme's value. It preserves Dither. Turning an effect off preserves its strength. All overrides follow image/theme changes and survive restart. Every row has its own search terms and stable widget identifier. The Command Palette offers Enable/Disable background bottom darkening and background vignette when an image is available.

## Storage and rendering

Settings are stored under `[appearance.background_image]`:

```toml
opacity = 100 # optional; omit to inherit the theme
brightness = 100
contrast = 100
gradient_enabled = false
gradient_strength = 60
gradient_start = 50
vignette_enabled = false
vignette_strength = 40
```

The imported image's default 30% opacity means a 70% theme-color overlay. The new opacity control adjusts that mixture; it is separate from Window Opacity. Appearance, workspace rendering and the terminal background overlay use the same resolved state. Native Metal retains its existing theme opacity behavior.

The existing image instance carries two additional `vec4` parameters through Image → Scene → wgpu. No texture or render pass is added. Neutral settings use the original image path. The fixed order is original sample → brightness/contrast → Dither → bottom darkening/vignette → existing alpha and theme-color compositing. Original crop, UVs, image alpha and window corner clipping are preserved. The masks use physical viewport coordinates, including when the cover image extends beyond the window. These controls do not affect glyphs, icons, other images or theme thumbnails.

BackgroundImageSettings is a separate settings group. Its changes notify the workspace and Appearance view without going through AppearanceManager's theme reload. It does not load assets, start a timer, or alter Dither settings. Static Dither bypasses the time-dependent wave and drift calculations. Only active-window animated Dither requests the existing 33.34 ms repaint interval.

## Verification

The automated checks cover settings inheritance, configuration bounds, persisted parameters with disabled switches, resetting without erasing Dither, legacy defaults, independently searchable controls, image availability, unchanged portrait/landscape geometry, and settings schemas. The GPU output test uses the production image pipeline and checks sampled color/alpha, exact identity output, gray neutrality, color math, viewport-based masks, resized windows, zero-strength composition, and time-independent static Dither.

Run the GPU test explicitly on a machine with a graphics adapter:

```sh
cargo test -p warpui --features experimental-wgpu-renderer background_image_gpu_output --lib -- --ignored --nocapture
```

## Reproducible performance comparison

Preserve an optimized pre-change executable and build both versions with the same configuration:

```sh
CARGO_PROFILE_RELEASE_DEBUG=0 \
CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=true \
CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=unpacked \
FRAMEWORK_OVERRIDE=dev \
cargo build --release -p warp --bin warp-oss --features gui,warpui/experimental-wgpu-renderer
```

Debug assertions preserve the isolated `WARP_DATA_PROFILE` mechanism; optimization remains enabled. Package and sign separate apps with separate `background-perf-*` profiles. Use the same static image with theme opacity 100%, window opacity 100%, grain size 8 and Dither strength 65. The candidate's effect cases use brightness 120%, contrast 115%, gradient 60% starting at 50%, and vignette 40%. Baseline "effects" is an additional original-image reference because the old executable does not implement these controls.

Set the desired window size and focus, then run:

```sh
python3 script/benchmark-background-images.py \
  --pid TARGET_PID --profile background-perf-baseline \
  --variant baseline --window normal --output /private/tmp/warp-image-measurements
```

Repeat for candidate and fullscreen Retina. Each case warms for 10 seconds and samples for 30 seconds, repeated three times with rotated order. Instruments records Metal System Trace plus Time Profiler. Process CPU and resident/physical memory are also sampled once per second through `proc_pid_rusage`. Cursor blinking supplies an identical low-rate redraw source for the static frame-cost comparison; repeat the idle check with it disabled to test absence of extra redraws. Do not compile or run GPU output tests during measurements.

GPU frame cost is the union of active Vertex/Fragment intervals for each render command buffer, excluding resource-upload command buffers. CPU Metal encoding time is reported separately from CPU utilization; it does not include all Rust layout/paint work. Full CPU frame duration requires additional instrumentation and must not be inferred from sampling. Frame submission frequency is measured from render command buffers, not the display refresh rate.

Measurement results and GUI verification are recorded with the delivery evidence, including unavailable metrics and limits on the comparison.

## Local verification status — 2026-09-08

The current optimized candidate is `A92A4708-09C3-3F1C-810F-2A0310284A4B`. It has been copied into `target/debug/bundle/osx/WarpOss.app` and ad-hoc signed; strict signature verification passed. The baseline executable is `DCB27101-02AF-3D9D-8E0B-A21B663609F7`. Both used the same release configuration above. The previous debug app and optimized baseline remain under `/private/tmp/warp-background-effects/`.

Passed: 11 relevant app tests, 14 image/Dither core tests, combined WGSL validation, and the explicit production-pipeline GPU output test on Metal. Clippy passed with `-D warnings` for the three affected libraries. Changed Rust and WGSL formatting checks and `git diff --check` passed. The GPU output test shares the production uniform bind-group layout, including fragment-stage access to viewport dimensions.

The CPU sampler was calibrated with a 300 ms single-core busy loop: **99.98%** measured CPU. This M4's Mach timebase is **125/3 ns per tick**; the runner applies that conversion. This calibration is a measurement of the sampler, not a performance result for Warp.

Test host: Apple M4 Pro (20 GPU cores, 14 CPU cores), 48 GiB memory, macOS 26.5.2, Xcode Instruments 26.0, AC power. The active Dell P2725QE display uses 2560×1440 logical points and a 5120×2880 backing resolution at 100 Hz. A 5-second Instruments pilot successfully collected CPU and Metal tables; it was performed during setup and is excluded from performance comparisons.

| Required verification | Status |
| --- | --- |
| Full import and live controls in the rebuilt candidate | Pending: Mac locked before the corrected candidate could be opened |
| Reset, search, theme switch and restart in the GUI | Pending unlock; corresponding settings/search tests passed |
| Normal-window baseline/candidate matrix, 10 s warmup + 30 s × 3 | Not measured |
| Fullscreen Retina matrix, same protocol | Not measured |
| Continuous output, scrolling, slider dragging, focus/minimize | Not measured |
| GPU P95 regression target | No acceptance conclusion yet |
| Full CPU frame duration | Not available from the chosen sampling method; CPU encoder time is separately reported |

The GUI checks and actual performance matrix remain required before claiming the complete plan is verified. These checks were blocked by the locked desktop during validation.
