<div align="center">

# `v4l-tui`

_TUI app for configuring cameras on Linux_

[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

</div>

[![demo video](media/demo.avif)](media/demo.avif?raw=true)

## Installation

Not yet published on https://crates.io.

For now, install via:

```sh
cargo install --git https://github.com/sermuns/v4l-tui
```

and run it with

```sh
v4l-tui
```

## TODO

- [ ] handle errors more gracefully, don't just `panic`
- [ ] support all values types for camera controls. currently only `Integer` and `Boolean` values are supported.
- [ ] document+support the "preview" feature better

## Disclaimer

No artificial intelligence was used in the making of this.

<a href="https://brainmade.org/">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://brainmade.org/white-logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://brainmade.org/black-logo.svg">
  <img alt="brainmade" src="https://brainmade.org/white-logo.svg">
</picture>
</a>
