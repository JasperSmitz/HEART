`pastel.png` is the user-supplied portrait, bundled byte-for-byte from
`/tmp/codex-clipboard-aWY4Cl.png` (SHA-256
`8e462a36024c876d856cd5388e14238506c0bd9bad4fc6aab25ece6639c29108`).
No generated artwork or runtime external file is required.

The dashboard displays it with nearest-neighbor contain scaling on known Kitty
graphics terminals (Ghostty and Kitty) when terminal pixel dimensions are available.
Unknown terminals, multiplexers, missing pixel metrics, `NO_COLOR` (nonempty), and decode failures use a
labelled text fallback. Set `HEART_PORTRAIT=text` before `--dashboard` to force this
fallback for troubleshooting. Detection uses environment hints and a window-size
ioctl only; it never queries or reads stdin. Cell metrics refresh on resize.

`cargo run -p heart --example dashboard_preview -- cat 120 40 image` exports
production-rendered cells with this exact PNG embedded in a **simulated graphics**
composition (9×18 pixel cells); it does not probe a terminal. Omit `image` for the
text fallback. `help`, `long`, and `image-long` modes exercise overlays and scrolling.
