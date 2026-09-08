#!/usr/bin/env python3
"""Export compact per-case results from benchmark-background-images.py recordings."""

import argparse
from collections import defaultdict
from datetime import datetime
import json
from pathlib import Path
import statistics
import subprocess
import xml.etree.ElementTree as ET


def export_rows(trace, schema, numeric_columns, text_columns, accept):
    """Stream the Instruments XML; retain scalar references, not full XML trees."""
    xpath = f'/trace-toc/run[@number="1"]/data/table[@schema="{schema}"]'
    process = subprocess.Popen(["xcrun", "xctrace", "export", "--input", str(trace), "--xpath", xpath], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    refs = {}
    stack = []
    selected = []
    numeric_tags = {"start-time", "duration", "metal-command-buffer-id", "uint32"}
    text_tags = {"process", "gpu-channel-name", "gpu-state"}
    try:
        for event, element in ET.iterparse(process.stdout, events=("start", "end")):
            if event == "start":
                stack.append(element)
                continue
            if element.tag == "row":
                for child in element.iter():
                    identifier = child.get("id")
                    if identifier and child.tag in numeric_tags and child.text:
                        refs[int(identifier)] = int(child.text)
                    elif identifier and child.tag in text_tags:
                        refs[int(identifier)] = child.get("fmt", "")

                def value(index):
                    item = element[index]
                    identifier = item.get("ref") or item.get("id")
                    return refs.get(int(identifier)) if identifier else None

                row = [value(i) for i in numeric_columns + text_columns]
                if accept(row):
                    selected.append(row)
                stack[-2].remove(element)
                element.clear()
            stack.pop()
    finally:
        process.stdout.close()
    error = process.stderr.read().decode()
    if process.wait():
        raise RuntimeError(error)
    return selected


def union_duration(intervals):
    total = 0
    start = end = None
    for left, right in sorted(intervals):
        if start is None:
            start, end = left, right
        elif left <= end:
            end = max(end, right)
        else:
            total += end - start
            start, end = left, right
    return total + (end - start if start is not None else 0)


def quantiles(values):
    if not values:
        return dict(median=None, p95=None, count=0)
    ordered = sorted(values)
    index = (len(ordered) - 1) * 0.95
    lower = int(index)
    p95 = ordered[lower] + (ordered[min(lower + 1, len(ordered) - 1)] - ordered[lower]) * (index - lower)
    return dict(median=statistics.median(ordered), p95=p95, count=len(ordered))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    args = parser.parse_args()
    metadata = json.loads(args.manifest.read_text())
    trace = Path(metadata["trace"])
    prefix = args.manifest.with_suffix("")
    toc_path = prefix.with_suffix(".toc.xml")
    subprocess.run(["xcrun", "xctrace", "export", "--input", str(trace), "--toc", "--output", str(toc_path)], check=True)
    toc = ET.parse(toc_path).getroot()
    origin = datetime.fromisoformat(toc.findtext(".//summary/start-date")).timestamp()
    pid = f"warp-oss ({metadata['pid']})"
    gpu = export_rows(trace, "metal-gpu-intervals", [0, 1, 15], [2, 7, 10], lambda r: r[-1] == pid and r[3] in ("Vertex", "Fragment") and r[4] == "Active")
    cpu = export_rows(trace, "metal-application-command-buffer-submissions", [0, 6, 14], [7], lambda r: r[-1] == pid)
    gpu_frames = defaultdict(list)
    for start, duration, command, *_ in gpu:
        if start is None or duration is None or command is None:
            raise ValueError("Missing target GPU scalar reference")
        gpu_frames[command].append((start, start + duration))
    gpu_ms = {command: union_duration(intervals) / 1e6 for command, intervals in gpu_frames.items()}
    frames = [dict(start=origin + start / 1e9, cpu_encode_ms=duration / 1e6, gpu_ms=gpu_ms[command]) for start, duration, command, _ in cpu if command in gpu_ms]
    result = dict(variant=metadata["variant"], window=metadata["window"], pid=metadata["pid"], trace=str(trace), segments=[], pooled={})
    pooled = defaultdict(lambda: defaultdict(list))
    for segment in metadata["segments"]:
        current = [frame for frame in frames if segment["start"] <= frame["start"] < segment["end"]]
        elapsed = segment["end"] - segment["start"]
        metrics = dict(
            gpu_ms=[frame["gpu_ms"] for frame in current],
            cpu_encode_ms=[frame["cpu_encode_ms"] for frame in current],
            cpu_percent=[sample["cpu_percent"] for sample in segment["samples"]],
            rss_mib=[sample["rss_bytes"] / 1024**2 for sample in segment["samples"]],
            footprint_mib=[sample["footprint_bytes"] / 1024**2 for sample in segment["samples"]],
        )
        fps = len(current) / elapsed
        result["segments"].append(dict(case=segment["case"], repeat=segment["repeat"], frames=len(current), seconds=elapsed, fps=fps, **{key: quantiles(value) for key, value in metrics.items()}))
        for key, values in metrics.items():
            pooled[segment["case"]][key].extend(values)
        pooled[segment["case"]]["fps"].append(fps)
    for case, metrics in pooled.items():
        result["pooled"][case] = {key: quantiles(values) for key, values in metrics.items()}
    prefix.with_suffix(".frames.json").write_text(json.dumps(frames))
    prefix.with_suffix(".results.json").write_text(json.dumps(result, indent=2))
    print(json.dumps(result["pooled"], indent=2))


if __name__ == "__main__":
    main()
