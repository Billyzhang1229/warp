#!/usr/bin/env python3
"""Profile an already-open, isolated Warp test app with Instruments on macOS.

Set the window size and focus before running. The script only changes its own
background-perf-* settings profile; it never drives the UI or installs an app.
Requires Xcode Instruments and an optimized wgpu build with debug assertions
enabled, so WARP_DATA_PROFILE is honored. See specs/background-image-controls.md.
"""

import argparse
import ctypes
import ctypes.util
import json
from pathlib import Path
import re
import subprocess
import time


class Usage(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_uint8 * 16)] + [
        (name, ctypes.c_uint64)
        for name in (
            "user", "system", "idle_wakeups", "interrupt_wakeups", "pageins",
            "wired", "resident", "footprint", "start", "exit", "child_user",
            "child_system", "child_idle", "child_interrupt", "child_pageins",
            "child_elapsed", "disk_read", "disk_write",
        )
    ]


def process_usage(pid):
    result = Usage()
    if LIBPROC.proc_pid_rusage(pid, 2, ctypes.byref(result)):
        raise OSError(ctypes.get_errno(), "Unable to sample the target process")
    return result


def configure(base, settings_path, case, variant, cursor_blink):
    text = base
    for section in ("appearance.dither", "appearance.background_image", "appearance.cursor"):
        text = re.sub(r"(?m)^\[" + re.escape(section) + r"\][^\[]*", "", text)
    dither = case in ("static_dither", "animated_dither")
    animated = case == "animated_dither"
    text += f"""
[appearance.cursor]
cursor_blink = "{cursor_blink}"

[appearance.dither]
enabled = {str(dither).lower()}
pixel_size = 8
strength = 65
animated = {str(animated).lower()}
"""
    if variant == "candidate":
        enabled = case != "original"
        text += f"""
[appearance.background_image]
opacity = 100
brightness = {120 if enabled else 100}
contrast = {115 if enabled else 100}
gradient_enabled = {str(enabled).lower()}
gradient_strength = 60
gradient_start = 50
vignette_enabled = {str(enabled).lower()}
vignette_strength = 40
"""
    settings_path.write_text(text)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int, required=True)
    parser.add_argument("--profile", required=True)
    parser.add_argument("--variant", choices=("baseline", "candidate"), required=True)
    parser.add_argument("--window", choices=("normal", "retina"), required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--warmup", type=int, default=10)
    parser.add_argument("--sample", type=int, default=30)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--cursor-blink", choices=("enabled", "disabled"), default="enabled")
    args = parser.parse_args()
    if not re.fullmatch(r"background-perf-[a-z-]+", args.profile):
        parser.error("Only an isolated background-perf-* profile may be modified")
    settings_path = Path.home() / f".warp-oss-{args.profile}" / "settings.toml"
    base = settings_path.read_text()
    args.output.mkdir(parents=True, exist_ok=True)
    prefix = args.output / f"{args.variant}-{args.window}"
    trace = prefix.with_suffix(".trace")
    if trace.exists():
        parser.error(f"Refusing to overwrite {trace}")
    cases = ["original", "effects", "static_dither", "animated_dither"]
    duration = args.repeats * len(cases) * (args.warmup + args.sample) + 15
    command = [
        "xcrun", "xctrace", "record", "--template", "Metal System Trace",
        "--instrument", "Time Profiler", "--attach", str(args.pid),
        "--time-limit", f"{duration}s", "--output", str(trace),
    ]
    # proc_pid_rusage CPU times use Mach absolute ticks (24 MHz on this M4),
    # unlike the nanosecond timestamps exported by Instruments.
    timebase = (ctypes.c_uint32 * 2)()
    system = ctypes.CDLL(ctypes.util.find_library("System"))
    if system.mach_timebase_info(ctypes.byref(timebase)):
        raise RuntimeError("Unable to read Mach clock timebase")
    cpu_tick_ns = timebase[0] / timebase[1]
    metadata = dict(vars(args), output=str(args.output), trace=str(trace), segments=[])
    metadata["cpu_tick_ns"] = cpu_tick_ns
    metadata["command"] = command
    metadata["settings_before"] = base
    metadata["time_origin"] = time.time()
    with prefix.with_suffix(".record.log").open("w") as log:
        recording = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
        try:
            for repeat in range(args.repeats):
                # Rotate the order so heating and drift do not always favor one case.
                order = cases[repeat:] + cases[:repeat]
                for case in order:
                    configure(base, settings_path, case, args.variant, args.cursor_blink)
                    print(f"{args.variant}/{args.window} {case} repeat {repeat + 1}: warming up", flush=True)
                    time.sleep(args.warmup)
                    segment = dict(case=case, repeat=repeat + 1, start=time.time(), samples=[])
                    previous = process_usage(args.pid)
                    previous_time = time.monotonic()
                    for _ in range(args.sample):
                        time.sleep(1)
                        now = time.monotonic()
                        current = process_usage(args.pid)
                        cpu = ((current.user + current.system) - (previous.user + previous.system)) * cpu_tick_ns / 1e9 / (now - previous_time) * 100
                        segment["samples"].append(dict(time=time.time(), cpu_percent=cpu, rss_bytes=current.resident, footprint_bytes=current.footprint))
                        previous, previous_time = current, now
                    segment["end"] = time.time()
                    metadata["segments"].append(segment)
                    prefix.with_suffix(".json").write_text(json.dumps(metadata, indent=2))
                    print(f"{case} repeat {repeat + 1}: sampled {len(segment['samples'])} seconds", flush=True)
                    if recording.poll() is not None:
                        raise RuntimeError(f"Instruments stopped early; see {log.name}")
            # Finish the bounded recording, then let Instruments save its trace.
            if recording.wait() != 0:
                raise RuntimeError(f"Instruments failed; see {log.name}")
        finally:
            if recording.poll() is None:
                recording.terminate()
                recording.wait()
    print(f"Saved {trace}", flush=True)


if __name__ == "__main__":
    LIBPROC = ctypes.CDLL(ctypes.util.find_library("proc"), use_errno=True)
    main()
