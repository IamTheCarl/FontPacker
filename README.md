# Font Packer

Font packer is a crude tool to convert True Type Fonts into [texture atlases](https://en.wikipedia.org/wiki/Texture_atlas)
Releases can be found on [the releases page](https://github.com/IamTheCarl/FontPacker/releases), including pre-compiled binaries.

Font packer is a [CLI](https://en.wikipedia.org/wiki/Command-line_interface) application, meaning you must use it through CMD on Windows or a terminal emulator on Linux.
Run `font_packer --help` for documentation on what arguments font packer expects.

## Output files

The output of Font Packer is as crude as reasonably possible, being simple gray scale images for the textures and a single JSON file containing location and positioning information about the glyphs. Metrics about the glyph provided by FontDue can be used to calculate the positioning of gylphs when rendering them. The meaning of this information follows the [same conventions as FontDue](https://docs.rs/fontdue/0.8.0/fontdue/struct.Metrics.html). The "mapping" section of a glyph indicates which textures the glyph is contained in and where in that texture it is located.

## Engine Support

Font packer does not specifically support any specific engine. There are too many engines out there for me to support, so it is up to you to adapt its output to your engine.
If you produce a tool to convert the outputs of Font Packer to fit a specific engine, fork this repository, add a link to the converter's repository here, and make a pull request so we can start building a list of converters.
