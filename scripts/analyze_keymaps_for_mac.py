#!/usr/bin/env python3
"""Analyze the 3 Windows ReaperKeyMap files and produce a Mac review report.

The Mac files are byte-identical copies of the Windows files. REAPER on macOS
auto-translates modifier bits:
  - bit 0x08 (keymap "Ctrl") -> displays as Cmd
  - bit 0x10 (keymap "Alt")  -> displays as Option
  - bit 0x20 (keymap "Win")  -> displays as Control (Mac Ctrl)

So no mechanical bit conversion is needed. What math65 must review by hand:
  - Win-modifier entries: usable on Mac (become Control+X) but position differs;
    decide whether to remap to a Mac-friendlier modifier.
  - MediaKbd (modifier 255) entries: special multimedia keys; many Mac
    keyboards lack these.
  - Action IDs referencing Windows-only scripts or paths.
  - Numpad / specific keycodes that behave differently on Mac.

The script writes docs/KEYMAP_MAC_REVIEW.md grouping problematic entries.
"""

from __future__ import annotations

import re
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
WIN_DIR = REPO / "Contents" / "KeyMaps_Win"
OUT = REPO / "docs" / "KEYMAP_MAC_REVIEW.md"

# Modifier byte: bit layout (low nibble is the "VK flag" odd/even).
# bit 0x04 = Shift, 0x08 = Ctrl, 0x10 = Alt, 0x20 = Win, 0x01 = VK marker.
WIN_BIT = 0x20
MEDIA_MOD = 255


def parse_key(line: str):
    parts = line.rstrip("\n").split(None, 4)
    if len(parts) < 5 or parts[0] != "KEY":
        return None
    try:
        mod = int(parts[1])
        keycode = int(parts[2])
    except ValueError:
        return None
    action = parts[3]
    rest = parts[4]
    section_str, _, comment = rest.partition("\t")
    try:
        section = int(section_str.strip())
    except ValueError:
        section = -1
    return mod, keycode, action, section, comment.strip()


def categorize_keycode(keycode: int) -> str:
    # Numpad (VK_NUMPAD0..VK_DIVIDE) = 0x60..0x6F = 96..111
    if 96 <= keycode <= 111:
        return "Numpad"
    # Function keys F1-F24 = 0x70..0x87 = 112..135
    if 112 <= keycode <= 135:
        return "Function key (F1-F24)"
    # Arrow keys / nav block: 33-40 (PgUp..Down)
    if 33 <= keycode <= 40:
        return "Navigation (arrows, Home, End, PgUp, PgDn)"
    # Letters A-Z = 65..90
    if 65 <= keycode <= 90:
        return "Letter"
    # Digits 0-9 (top row) = 48..57
    if 48 <= keycode <= 57:
        return "Digit (top row)"
    if keycode in (8, 9, 13, 27, 32):
        return "Special (Backspace, Tab, Enter, Esc, Space)"
    if keycode in (45, 46):
        return "Insert/Delete"
    if keycode >= 1024:
        return "MIDI / extended"
    return "Other"


def decode_modifier(mod: int) -> str:
    if mod == MEDIA_MOD:
        return "MediaKbd"
    parts = []
    if mod & 0x04:
        parts.append("Shift")
    if mod & 0x08:
        parts.append("Ctrl")
    if mod & 0x10:
        parts.append("Alt")
    if mod & 0x20:
        parts.append("Win")
    return "+".join(parts) if parts else "(none)"


def analyze(path: Path):
    win_entries = []
    media_entries = []
    other_special = []
    win_mod_categories = Counter()
    win_mod_combos = Counter()

    with path.open(encoding="utf-8", errors="replace") as fh:
        for raw in fh:
            if not raw.startswith("KEY "):
                continue
            parsed = parse_key(raw)
            if not parsed:
                continue
            mod, keycode, action, section, comment = parsed
            if mod == MEDIA_MOD:
                media_entries.append((mod, keycode, action, section, comment))
                continue
            if mod & WIN_BIT:
                win_entries.append((mod, keycode, action, section, comment))
                win_mod_categories[categorize_keycode(keycode)] += 1
                win_mod_combos[decode_modifier(mod)] += 1
                continue
            if mod >= 128 and mod != MEDIA_MOD:
                other_special.append((mod, keycode, action, section, comment))
    return {
        "win": win_entries,
        "media": media_entries,
        "other_special": other_special,
        "win_categories": win_mod_categories,
        "win_combos": win_mod_combos,
    }


def format_entry(entry):
    mod, keycode, action, section, comment = entry
    return f"  - `KEY {mod} {keycode}` (sec {section}) -> `{action}` | _{comment}_"


def main():
    files = sorted(WIN_DIR.glob("KeyMap ReaperAccessible - Win - *.ReaperKeyMap"))
    assert files, f"No keymaps found in {WIN_DIR}"

    lines: list[str] = []
    lines.append("# Mac KeyMap review notes for math65")
    lines.append("")
    lines.append(
        "These notes accompany the 3 files in `Contents/KeyMaps_Mac/`. They are "
        "currently byte-identical to the Windows versions, because REAPER on macOS "
        "automatically remaps modifier bits at display time:"
    )
    lines.append("")
    lines.append("| Keymap bit | Win shows | Mac shows |")
    lines.append("|---|---|---|")
    lines.append("| 0x08 | Ctrl | **Cmd** |")
    lines.append("| 0x10 | Alt | **Option** |")
    lines.append("| 0x20 | Win | **Control** (Mac Ctrl) |")
    lines.append("")
    lines.append(
        "So Ctrl+X on Windows becomes Cmd+X on Mac with **zero file changes**. "
        "Same for Alt+X -> Option+X. The cases below are the ones that *might* "
        "need a Mac-specific decision."
    )
    lines.append("")
    lines.append("---")
    lines.append("")

    for path in files:
        variant = path.stem.split(" - ")[-1]
        report = analyze(path)
        win = report["win"]
        media = report["media"]
        other = report["other_special"]
        cats = report["win_categories"]
        combos = report["win_combos"]

        lines.append(f"## {variant}")
        lines.append("")
        lines.append(f"Source: `{path.relative_to(REPO)}`")
        lines.append("")

        lines.append("### Summary")
        lines.append("")
        lines.append(f"- **Win-modifier entries** (become Mac Control+X): {len(win)}")
        lines.append(f"- **MediaKbd entries** (multimedia keys, may not exist on Mac): {len(media)}")
        if other:
            lines.append(f"- **Other special modifiers (>=128, not 255)**: {len(other)}")
        lines.append("")

        if combos:
            lines.append("### Win-modifier combinations")
            lines.append("")
            lines.append("On Mac these become the same combination with **Control** in place of the Windows key.")
            lines.append("")
            for combo, count in sorted(combos.items(), key=lambda x: -x[1]):
                lines.append(f"- `{combo}` -> Mac displays as `{combo.replace('Win', 'Control').replace('Ctrl', 'Cmd').replace('Alt', 'Option')}`: **{count} entries**")
            lines.append("")

        if cats:
            lines.append("### Win-modifier entries by keycode category")
            lines.append("")
            for cat, count in sorted(cats.items(), key=lambda x: -x[1]):
                lines.append(f"- {cat}: **{count}**")
            lines.append("")

        if media:
            lines.append("### MediaKbd (modifier 255) entries — usually DISABLED")
            lines.append("")
            lines.append("Most are `DISABLED DEFAULT` no-ops to block accidental triggers. Mac keyboards rarely have these media keys; leaving them disabled is safe.")
            lines.append("")
            sample = media[:10]
            for e in sample:
                lines.append(format_entry(e))
            if len(media) > 10:
                lines.append(f"  - _(... +{len(media) - 10} more)_")
            lines.append("")

        if other:
            lines.append("### Other special modifiers (need direct inspection)")
            lines.append("")
            for e in other:
                lines.append(format_entry(e))
            lines.append("")

        lines.append("### Recommendation")
        lines.append("")
        lines.append(
            "1. **Function keys + Win modifier**: usually safe to keep (Control+Fn is reachable on Mac)."
        )
        lines.append(
            "2. **Numpad + Win modifier**: keep if the user has an extended Mac keyboard, otherwise consider remapping to top-row digits."
        )
        lines.append(
            "3. **Letter + Win modifier**: review case-by-case. Control+letter on Mac sometimes conflicts with VoiceOver routing keys (Control+Option+letter). Test with VoiceOver enabled."
        )
        lines.append(
            "4. **MediaKbd entries**: leave as DISABLED; they won't fire on Mac keyboards lacking those keys."
        )
        lines.append("")
        lines.append("---")
        lines.append("")

    lines.append("## How to proceed")
    lines.append("")
    lines.append("1. Open one of the Mac files in `Contents/KeyMaps_Mac/` in a text editor.")
    lines.append("2. For each Win-modifier combination flagged above, decide:")
    lines.append("   - Keep it (REAPER will show it as Control+X — works, just different position).")
    lines.append("   - Remap by changing the modifier byte (e.g., `33` -> `9` to switch Win-only -> Cmd).")
    lines.append("   - Delete the line (the shortcut becomes unavailable on Mac).")
    lines.append("3. Test with REAPER on macOS to validate no conflicts with VoiceOver / system shortcuts.")
    lines.append("4. Commit the curated Mac file(s).")
    lines.append("")
    lines.append("**No need to touch entries with modifiers 0/1/5/9/13/17/21/25/29** — REAPER handles them automatically.")
    lines.append("")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"Wrote {OUT.relative_to(REPO)}")


if __name__ == "__main__":
    main()
