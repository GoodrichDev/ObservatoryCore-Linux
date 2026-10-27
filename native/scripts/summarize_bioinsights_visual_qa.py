#!/usr/bin/env python3
"""Validate and summarize one immutable release's private graphical QA runs."""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    runs = {
        "orange": "final-release-orange-v5",
        "original": "final-release-original-v4",
        "installed_windows_fonts": "final-release-windows-fonts-v3",
        "open_noto_font": "final-release-noto-font-v2",
        "controls": "final-release-controls-v5",
        "large_survey": "final-release-stress-v5",
    }
    data = {name: json.loads((args.root / directory / "receipt.json").read_text()) for name, directory in runs.items()}
    hashes = {receipt["core_sha256"] for receipt in data.values()}
    assert len(hashes) == 1, hashes
    result = {"date": "2026-10-26", "core_sha256": hashes.pop(), "synthetic_only": True,
              "display": "owned private Xvfb; software GL", "runs": {}}
    for name, receipt in data.items():
        if name == "controls":
            for renderer in ("rust", "webkit"):
                zoom = receipt["cases"][renderer + "-zoom"]
                assert (zoom["minimum"], zoom["maximum"], zoom["reset"]) == (50, 600, 100)
                assert zoom["settings_persist_immediately"] and zoom["closes_on_focus_loss"]
                assert zoom["native_windows"] == 1
                disabled = receipt["cases"][renderer + "-disabled"]
                assert disabled["no_custom_toolbar"] and disabled["no_main_table"]
            result["runs"][name] = receipt["cases"]
            continue
        run = {"theme": receipt["theme"], "host_windows_fonts": receipt["host_windows_fonts"],
               "cases": {}, "comparisons": receipt["comparisons"]}
        for case, fields in receipt["cases"].items():
            run["cases"][case] = {key: fields[key] for key in (
                "window", "fontconfig", "idle_cpu_percent_one_core", "latency_sampling",
                "navigation_median_ms", "stress_projection") if key in fields}
            if "navigation" in fields:
                navigation = [{key: event[key] for key in ("page", "latency_ms", "native_windows")} for event in fields["navigation"]]
                assert all(event["native_windows"] == 1 for event in navigation)
                assert {event["page"] for event in navigation} == {"About", "Main", "Unsold", "CodexCompanion", "GenusBacterium", "SpeciesAlcyoneum"}
                run["cases"][case]["navigation"] = navigation
        directory = args.root / runs[name]
        for size, comparison in run["comparisons"].items():
            assert comparison["toolbar_centers_equal"] and comparison["outer_bands_equal"] and comparison["brown_row_positions_equal"], (name, size, comparison)
            # Navigation's final Main capture contains an intentional hovered
            # Main button. Genus captures move the pointer outside first.
            suffix = "genusbacterium" if (directory / f"rust-{size}-genusbacterium.png").is_file() else "main"
            images = [np.asarray(Image.open(directory / f"{renderer}-{size}-{suffix}.png").convert("RGB")) for renderer in ("rust", "webkit")]
            toolbar = receipt["cases"]["rust-" + size]["toolbar"]
            differences = {}
            for button, center in zip(("main", "unsold", "about", "codexcompanion", "popout", "zoom", "settings"), toolbar["centers"]):
                y, x = toolbar["y"], int(center) - 16
                differences[button] = float(np.abs(images[0][y:y+32, x:x+32].astype(np.int16) - images[1][y:y+32, x:x+32]).mean())
            assert max(differences.values()) < 3, (name, size, differences)
            comparison["toolbar_icon_mean_absolute_channel_difference"] = differences
        run["screenshots"] = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in sorted(directory.glob("*.png"))}
        run["raw_receipt"] = str(directory / "receipt.json")
        result["runs"][name] = run
    stress = result["runs"]["large_survey"]["cases"]["rust-3840x2160"]
    assert stress["stress_projection"] == {"bodies": 200, "species_rows": 800, "journal_events": 607}
    result["limits"] = [
        "Source HTML is independently captured from original pure builders; native and WebKit use the same captured markup/state inputs.",
        "Private network namespace intentionally exercises offline gallery/bioscan paths; missing remote pictures in these captures are not evidence of live service failure.",
        "Font rasterization differs slightly across platform engines; measured symbol centers differ by at most1.5px and fallback-font caption bounds by1px in these cases.",
        "Navigation times are observed response bounds including10ms mouse hold and sub-millisecond ROI capture, not instrumented pure paint times.",
        "WebKit CPU includes its GTK pump and forced software rendering; native is the default renderer.",
    ]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"core_sha256": result["core_sha256"], "passed_runs": list(result["runs"]), "large_survey": stress}))


if __name__ == "__main__":
    main()
