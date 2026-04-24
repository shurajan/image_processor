# image_processor

A Rust workspace that demonstrates a dynamic plugin system for image processing.
The main CLI binary (`ip`) loads shared libraries at runtime and applies image
transformations — blur or mirror — driven by a JSON configuration file.

## Workspace layout

| Crate | Type | Description |
|---|---|---|
| `ip` | binary | CLI tool — entry point |
| `plugin_error` | library | Shared FFI error codes |
| `blur_plugin` | cdylib | Gaussian and box blur plugin |
| `mirror_plugin` | cdylib | Horizontal / vertical mirror plugin |

## Building

```bash
cargo build
```

Plugins are built as shared libraries (`libblur.dylib` / `libmirror.dylib` on macOS,
`libblur.so` / `libmirror.so` on Linux).

## Usage

```
ip -i <input> -o <output> -p <plugin> -d <params.json> [-l <plugin_dir>]
```

| Flag | Description | Default |
|---|---|---|
| `-i` | Input image path | — |
| `-o` | Output image path | — |
| `-p` | Plugin name (`blur`, `mirror`) | — |
| `-d` | Path to the JSON params file | — |
| `-l` | Directory containing plugin shared libraries | `target/debug` |

### Example

```bash
ip -i photo.png -o out.png -p blur -d tests/config/blur_gauss.json
```

## Plugin parameter format

Parameters are passed to each plugin as a JSON file.

### blur

```json
{ "method": "box", "radius": 3, "iterations": 2 }
```
```json
{ "method": "gauss", "radius": 9, "sigma": 3.0 }
```

### mirror

```json
{ "horizontal": true, "vertical": false }
```

## Plugin ABI

Every plugin must export the following C function:

```c
int32_t process_image(uint32_t width, uint32_t height,
                      uint8_t *rgba_data, const char *params);
```

- `rgba_data` — RGBA8 pixel buffer, modified in-place (`width × height × 4` bytes).
- `params` — null-terminated UTF-8 JSON string.
- Return value — `PluginError` code: `0` = success, `1` = invalid size,
  `2` = unknown error, `3` = invalid params.

## Running unit tests

```bash
cargo test --lib --bins
```

## Running integration tests
Integration tests live in `ip/tests/integration.rs` and require the plugins to
be built beforehand.

```bash
cargo build
cargo test --test integration
```