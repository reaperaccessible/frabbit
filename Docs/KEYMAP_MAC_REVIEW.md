# Mac KeyMap review notes for math65

These notes accompany the 3 files in `Contents/KeyMaps_Mac/`. They are currently byte-identical to the Windows versions, because REAPER on macOS automatically remaps modifier bits at display time:

| Keymap bit | Win shows | Mac shows |
|---|---|---|
| 0x08 | Ctrl | **Cmd** |
| 0x10 | Alt | **Option** |
| 0x20 | Win | **Control** (Mac Ctrl) |

So Ctrl+X on Windows becomes Cmd+X on Mac with **zero file changes**. Same for Alt+X -> Option+X. The cases below are the ones that *might* need a Mac-specific decision.

---

## FRC

Source: `Contents\KeyMaps_Win\KeyMap ReaperAccessible - Win - FRC.ReaperKeyMap`

### Summary

- **Win-modifier entries** (become Mac Control+X): 378
- **MediaKbd entries** (multimedia keys, may not exist on Mac): 40

### Win-modifier combinations

On Mac these become the same combination with **Control** in place of the Windows key.

- `Ctrl+Win` -> Mac displays as `Cmd+Control`: **106 entries**
- `Ctrl+Alt+Win` -> Mac displays as `Cmd+Option+Control`: **86 entries**
- `Alt+Win` -> Mac displays as `Option+Control`: **72 entries**
- `Shift+Alt+Win` -> Mac displays as `Shift+Option+Control`: **42 entries**
- `Shift+Ctrl+Win` -> Mac displays as `Shift+Cmd+Control`: **42 entries**
- `Shift+Ctrl+Alt+Win` -> Mac displays as `Shift+Cmd+Option+Control`: **13 entries**
- `Shift+Win` -> Mac displays as `Shift+Control`: **11 entries**
- `Win` -> Mac displays as `Control`: **6 entries**

### Win-modifier entries by keycode category

- Numpad: **133**
- Letter: **119**
- MIDI / extended: **67**
- Other: **28**
- Digit (top row): **15**
- Special (Backspace, Tab, Enter, Esc, Space): **10**
- Function key (F1-F24): **6**

### MediaKbd (modifier 255) entries — usually DISABLED

Most are `DISABLED DEFAULT` no-ops to block accidental triggers. Mac keyboards rarely have these media keys; leaving them disabled is safe.

  - `KEY 255 2280` (sec 0) -> `0` | _# Main : ToucheMédia+Muter : DISABLED DEFAULT_
  - `KEY 255 6888` (sec 0) -> `0` | _# Main : ToucheMédia+Mic+ : DISABLED DEFAULT_
  - `KEY 255 13288` (sec 0) -> `0` | _# Main : ToucheMédia+Canal+ : DISABLED DEFAULT_
  - `KEY 255 13544` (sec 0) -> `0` | _# Main : ToucheMédia+Canal- : DISABLED DEFAULT_
  - `KEY 255 12008` (sec 0) -> `0` | _# Main : ToucheMédia+Lecture : DISABLED DEFAULT_
  - `KEY 255 6376` (sec 0) -> `0` | _# Main : ToucheMédia+MicMuter : DISABLED DEFAULT_
  - `KEY 255 12520` (sec 0) -> `0` | _# Main : ToucheMédia+Enregistrer : DISABLED DEFAULT_
  - `KEY 255 3048` (sec 0) -> `0` | _# Main : ToucheMédia+Piste+ : DISABLED DEFAULT_
  - `KEY 255 3560` (sec 0) -> `0` | _# Main : ToucheMédia+Arrêt : DISABLED DEFAULT_
  - `KEY 255 3816` (sec 0) -> `0` | _# Main : ToucheMédia+LecturePause : DISABLED DEFAULT_
  - _(... +30 more)_

### Recommendation

1. **Function keys + Win modifier**: usually safe to keep (Control+Fn is reachable on Mac).
2. **Numpad + Win modifier**: keep if the user has an extended Mac keyboard, otherwise consider remapping to top-row digits.
3. **Letter + Win modifier**: review case-by-case. Control+letter on Mac sometimes conflicts with VoiceOver routing keys (Control+Option+letter). Test with VoiceOver enabled.
4. **MediaKbd entries**: leave as DISABLED; they won't fire on Mac keyboards lacking those keys.

---

## FRF

Source: `Contents\KeyMaps_Win\KeyMap ReaperAccessible - Win - FRF.ReaperKeyMap`

### Summary

- **Win-modifier entries** (become Mac Control+X): 255
- **MediaKbd entries** (multimedia keys, may not exist on Mac): 16

### Win-modifier combinations

On Mac these become the same combination with **Control** in place of the Windows key.

- `Ctrl+Win` -> Mac displays as `Cmd+Control`: **103 entries**
- `Ctrl+Alt+Win` -> Mac displays as `Cmd+Option+Control`: **44 entries**
- `Alt+Win` -> Mac displays as `Option+Control`: **40 entries**
- `Shift+Alt+Win` -> Mac displays as `Shift+Option+Control`: **29 entries**
- `Shift+Ctrl+Win` -> Mac displays as `Shift+Cmd+Control`: **21 entries**
- `Shift+Ctrl+Alt+Win` -> Mac displays as `Shift+Cmd+Option+Control`: **7 entries**
- `Shift+Win` -> Mac displays as `Shift+Control`: **6 entries**
- `Win` -> Mac displays as `Control`: **5 entries**

### Win-modifier entries by keycode category

- Numpad: **118**
- Letter: **62**
- MIDI / extended: **43**
- Other: **12**
- Digit (top row): **11**
- Special (Backspace, Tab, Enter, Esc, Space): **5**
- Function key (F1-F24): **4**

### MediaKbd (modifier 255) entries — usually DISABLED

Most are `DISABLED DEFAULT` no-ops to block accidental triggers. Mac keyboards rarely have these media keys; leaving them disabled is safe.

  - `KEY 255 2280` (sec 0) -> `0` | _# Main : ToucheMédia+Muter : DISABLED DEFAULT_
  - `KEY 255 6888` (sec 0) -> `0` | _# Main : ToucheMédia+Mic+ : DISABLED DEFAULT_
  - `KEY 255 6632` (sec 0) -> `0` | _# Main : ToucheMédia+Mic- : DISABLED DEFAULT_
  - `KEY 255 13288` (sec 0) -> `0` | _# Main : ToucheMédia+Canal+ : DISABLED DEFAULT_
  - `KEY 255 13544` (sec 0) -> `0` | _# Main : ToucheMédia+Canal- : DISABLED DEFAULT_
  - `KEY 255 6376` (sec 0) -> `0` | _# Main : ToucheMédia+MicMuter : DISABLED DEFAULT_
  - `KEY 255 13032` (sec 0) -> `0` | _# Main : ToucheMédia+Rembobiner : DISABLED DEFAULT_
  - `KEY 255 12520` (sec 0) -> `0` | _# Main : ToucheMédia+Enregistrer : DISABLED DEFAULT_
  - `KEY 255 3048` (sec 0) -> `0` | _# Main : ToucheMédia+Piste+ : DISABLED DEFAULT_
  - `KEY 255 3304` (sec 0) -> `0` | _# Main : ToucheMédia+Piste- : DISABLED DEFAULT_
  - _(... +6 more)_

### Recommendation

1. **Function keys + Win modifier**: usually safe to keep (Control+Fn is reachable on Mac).
2. **Numpad + Win modifier**: keep if the user has an extended Mac keyboard, otherwise consider remapping to top-row digits.
3. **Letter + Win modifier**: review case-by-case. Control+letter on Mac sometimes conflicts with VoiceOver routing keys (Control+Option+letter). Test with VoiceOver enabled.
4. **MediaKbd entries**: leave as DISABLED; they won't fire on Mac keyboards lacking those keys.

---

## USA

Source: `Contents\KeyMaps_Win\KeyMap ReaperAccessible - Win - USA.ReaperKeyMap`

### Summary

- **Win-modifier entries** (become Mac Control+X): 252
- **MediaKbd entries** (multimedia keys, may not exist on Mac): 17

### Win-modifier combinations

On Mac these become the same combination with **Control** in place of the Windows key.

- `Ctrl+Win` -> Mac displays as `Cmd+Control`: **103 entries**
- `Alt+Win` -> Mac displays as `Option+Control`: **43 entries**
- `Ctrl+Alt+Win` -> Mac displays as `Cmd+Option+Control`: **43 entries**
- `Shift+Alt+Win` -> Mac displays as `Shift+Option+Control`: **26 entries**
- `Shift+Ctrl+Win` -> Mac displays as `Shift+Cmd+Control`: **22 entries**
- `Shift+Ctrl+Alt+Win` -> Mac displays as `Shift+Cmd+Option+Control`: **6 entries**
- `Shift+Win` -> Mac displays as `Shift+Control`: **6 entries**
- `Win` -> Mac displays as `Control`: **3 entries**

### Win-modifier entries by keycode category

- Numpad: **117**
- Letter: **62**
- MIDI / extended: **44**
- Other: **11**
- Digit (top row): **11**
- Special (Backspace, Tab, Enter, Esc, Space): **4**
- Function key (F1-F24): **3**

### MediaKbd (modifier 255) entries — usually DISABLED

Most are `DISABLED DEFAULT` no-ops to block accidental triggers. Mac keyboards rarely have these media keys; leaving them disabled is safe.

  - `KEY 255 2280` (sec 0) -> `0` | _# Main : MediaKbd+Mute : DISABLED DEFAULT_
  - `KEY 255 13544` (sec 0) -> `0` | _# Main : MediaKbd+Chan- : DISABLED DEFAULT_
  - `KEY 255 6632` (sec 0) -> `0` | _# Main : MediaKbd+Mic- : DISABLED DEFAULT_
  - `KEY 255 6888` (sec 0) -> `0` | _# Main : MediaKbd+Mic+ : DISABLED DEFAULT_
  - `KEY 255 6376` (sec 0) -> `0` | _# Main : MediaKbd+MicMute : DISABLED DEFAULT_
  - `KEY 255 3048` (sec 0) -> `0` | _# Main : MediaKbd+Track+ : DISABLED DEFAULT_
  - `KEY 255 3304` (sec 0) -> `0` | _# Main : MediaKbd+Track- : DISABLED DEFAULT_
  - `KEY 255 4072` (sec 0) -> `0` | _# Main : MediaKbd+Mail : DISABLED DEFAULT_
  - `KEY 255 11496` (sec 0) -> `0` | _# Main : MediaKbd+MicOnOff : DISABLED DEFAULT_
  - `KEY 255 12520` (sec 0) -> `0` | _# Main : MediaKbd+Record : DISABLED DEFAULT_
  - _(... +7 more)_

### Recommendation

1. **Function keys + Win modifier**: usually safe to keep (Control+Fn is reachable on Mac).
2. **Numpad + Win modifier**: keep if the user has an extended Mac keyboard, otherwise consider remapping to top-row digits.
3. **Letter + Win modifier**: review case-by-case. Control+letter on Mac sometimes conflicts with VoiceOver routing keys (Control+Option+letter). Test with VoiceOver enabled.
4. **MediaKbd entries**: leave as DISABLED; they won't fire on Mac keyboards lacking those keys.

---

## How to proceed

1. Open one of the Mac files in `Contents/KeyMaps_Mac/` in a text editor.
2. For each Win-modifier combination flagged above, decide:
   - Keep it (REAPER will show it as Control+X — works, just different position).
   - Remap by changing the modifier byte (e.g., `33` -> `9` to switch Win-only -> Cmd).
   - Delete the line (the shortcut becomes unavailable on Mac).
3. Test with REAPER on macOS to validate no conflicts with VoiceOver / system shortcuts.
4. Commit the curated Mac file(s).

**No need to touch entries with modifiers 0/1/5/9/13/17/21/25/29** — REAPER handles them automatically.
