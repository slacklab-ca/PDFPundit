# 0024. Two output switches: repair and markdown

- **Status:** accepted
- **Date:** 2026-10-24
- **Decided in:** D-162 (the owner's request, 2026-10-24); amends D-048's export trigger

## Decision

The main screen has two switches. They decide what each dropped file turns into.

| Switch | Key | On | Off |
|---|---|---|---|
| repair | `r` | writes `<name>.repaired.pdf` when the repair makes one | the repair still runs, in memory; no PDF is written |
| markdown | `m` | writes `<name>.md` after the repair, without being asked | Markdown only on request (`e`, the file menu) |

1. The default is repair on, markdown off. That is the behaviour before the switches.
2. At least one switch stays on. Turning off the last one is refused with a hint, because a drop would then make nothing.
3. A drop takes the switches as they are when it lands. Flipping one later does not change files already queued.
4. The Markdown is always made from the repaired document when the repair makes one, even with repair off. The switch decides what is written, not what is computed.
5. A failed repair gets no automatic Markdown. An encrypted file, or one no candidate could be built for, keeps its "failed" row and writes nothing. `e` after the job ends still exports it, as an explicit ask.
6. While the switch's own export is coming, `e` on that file shows a hint instead of writing a second `.md`.
7. The switches persist in `switches.toml` in the config directory. The app rewrites that file on each flip, through a temporary file it syncs and renames over the old one. `config.toml` is never written by the app. `switches.toml` is app state, not an output, so [0002](0002-outputs-placed-never-replaced.md)'s no-replace rule does not apply to it.
8. Where they show:
   - widget: the status bar, `■repair □md`, the key letter yellow. They give way when the right side needs the room.
   - full idle: two MENU rows, `[R] repair ON` and `[M] markdown OFF`, and the hotkeys row.
   - batch and results: the header, `[r] repair ON · [m] markdown OFF`.
9. A queue row whose PDF was withheld reads `markdown`, not `repaired` or `partial`. The history's run record carries `withheld: true`, so a later reader can tell it from a clean file.
10. The keys are bare `r` and `m`, or their capitals. Ctrl+R and other modified keys flip nothing.

## Why

- The owner's request: the default screen needs a way to turn repair off and markdown on, and it must hold between launches.
- The widget has no file menu, so `e` was unreachable there. A dropped file in the widget could never become Markdown.
- Computing the repair even with repair off gives better Markdown from a damaged file. It costs time, not artefacts.
- A separate file keeps the user's hand-edited `config.toml`, with its comments, untouched.

## Rejected

- A `config.toml` key. The app would have to rewrite the user's file, or the switch would not persist from the UI.
- Repair off meaning "export the input as it is". A damaged file's Markdown would be worse for no gain.
- Allowing both off as an "analyse only" mode. The widget cannot show findings, so the drop would appear to do nothing.

## Known limits

- The widget draws no hint row, so the refusal in rule 2 is silent there. The switch simply stays on. Showing hints in the widget needs a mockup frame first ([0022](0022-ui-canvas-and-goldens.md)).

## Rules for agents

- A new output kind adds a switch only with an amendment to this ADR and to [0001](0001-v1-scope.md).
- A test that drops a file states which `Outputs` it submits with.
