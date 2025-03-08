# `nitrogfx`

A library for converting various graphics file formats for the Nintendo DS.

Name of project is subject to change.

> [!CAUTION]
> Currently in an **unstable** state.
> - The API may change without notice.
> - This repository will be subject to force-pushing and thus git history may be inconsistent and not retained. 

## Features

Convert seamlessly between NDS file formats and other more accessible file formats (e.g. PNG images, JASC palettes, JSON files, etc.).

Refer to the [API docs](https://cbt6.github.io/nitrogfx) for full details.

Supported file formats and conversions:
- [ ] `NANR`
- [x] `NCER` (to/from `json`)
- [x] `NCGR` (to/from `png`)
- [x] `NCLR` (to/from `jasc`, from `png`)
- [x] `NSCR` (to `png`)
- [ ] (more formats to come)

## Example

```rust
// NCLR -> JASC
let nclr = Nclr::read_from_file("input.NCLR")?;
let metadata = nclr.metadata();
println!("nclr_metadata: {:?}", metadata);
let palette = nclr.to_palette();
Jasc::from_palette(palette).write_to_file("output.pal")?;
```

```rust
// JASC -> NCLR
let palette = Jasc::read_from_file("input.pal")?.to_palette();
let metadata = NclrMetadata::default().with_version(NtrFileVersion::Version0101);
Nclr::from_palette(palette, metadata).write_to_file("output.NCLR")?;
```

## Usage

1. Git clone this repository locally.
2. Add it as a dependency in your project's `Cargo.toml`:

```toml
[dependencies]
nitrogfx = { path = "path/to/local/nitrogfx/repo" }
```

## Tests

Test have been written to verify that conversions are bijective where applicable.

Before running tests, ensure that test files have been placed in the appropriate locations, e.g. placing `NCLR` files in `tests/assets/nclr`.

## Contributing

Code contributions are currently not being accepted.

## License

[The Unlicense](LICENSE).
