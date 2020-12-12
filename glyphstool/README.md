# Glyphstool

This subdirectory contains some Rust tools for manipulating font files in the Glyphs format. These are special-purpose for doing manipulations on the Inconsolata sources, but could perhaps be adapted into more general purpose tools.

Perhaps the most valuable going forward is the "info-syms" script, which generates a set of line and box drawing glyphs. This is inspired by the [box-drawing] library that was developed for Source Code Pro, but has its own drawing logic, largely to support a wide range of widths and weights.

The best source of documentation is "read the source," sadly. If people navigate through it and make notes, those will gladly be accepted as a PR.

## License

Licensed under either of
  * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
    http://www.apache.org/licenses/LICENSE-2.0)
  * MIT license ([LICENSE-MIT](LICENSE-MIT) or
    http://opensource.org/licenses/MIT) at your option.

[box-drawing]: https://github.com/adobe-type-tools/box-drawing
