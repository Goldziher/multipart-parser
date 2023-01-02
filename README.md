# Fast Multipart Parser

<!-- markdownlint-disable -->
<img alt="Starlite logo" src="./starlite-banner.svg" width="100%" height="auto">
<!-- markdownlint-restore -->

This library includes an ultra-fast Rust based multipart parser. This parser is used
by [`Starlite`](https://github.com/starlite-api/starlite), but is developed separately - and can of course be used separately.

<div align="center">

[![Discord](https://img.shields.io/discord/919193495116337154?color=blue&label=chat%20on%20discord&logo=discord)](https://discord.gg/X3FJqy8d2j)
[![Matrix](https://img.shields.io/badge/%5Bm%5D%20chat%20on%20Matrix-bridged-blue)](https://matrix.to/#/#starlitespace:matrix.org)

</div>

## Installation

```shell
pip install fast-multipart-parser
```

## Usage

The library exposes two functions `parse_content_header` and `parse_multipart_form_data`.

### `parse_content_header`

This function is used to parse a `Content-Disposition` or `Content-Type` like header into two components -
a value (string) and a parameters (dict).

```python
from fast_multipart_parser import parse_content_header

result = parse_content_header(b"Content-Disposition: form-data; name=\"value\"")
# form-data, {"name": "value"}
```

#### Benchmarks

TODO

### `parse_multipart_form_data`

TODO

#### Benchmarks

TODO

## Contributing

All contributions are of course welcome!

### Repository Setup

1. Run `cargo install` to setup the rust dependencies and `poetry install` to setup the python dependencies.
2. Install the pre-commit hooks with `pre-commit install` (requires [pre-commit](https://pre-commit.com/)).

### Building

Run `poetry run maturin develop --release --strip` to install a release wheel (without debugging info). This wheel can be
used in tests and benchmarks.

### Benchmarking

Benchmarks use pyperf. To execute them run `poetry run python benchrmarks/main.py`.
