# beavor

![beavor homescreen](.resources/beavor_homescreen.png)

## Compilation

Run `make` from the root to compile the Rust backend in debug mode.

Run `make release` to compile in optimized release mode

## Use

Run with `./launch` or `python3 ./launch`

## Development

`beavor/backend.pyi` is manually written. If the backend Rust code is updated, this MUST be manually updated to ensure that type hinting in Python remains up-to-date
