# geo-coding

[![Crates.io Version](https://img.shields.io/crates/v/geo-coding)](https://crates.io/geo-coding/geo-coding)
[![Docs](https://docs.rs/geo-coding/badge.svg)](https://docs.rs/geo-coding)
[![dependency status](https://deps.rs/repo/github/igankevich/geo-coding/status.svg)](https://deps.rs/repo/github/igankevich/geo-coding)

This is offline reverse geocoding crate that uses files derived from OSM data.
In memory the data is stored in a two-dimensional tree that enables efficient queruing for nearest neighbours.
The total size of all files related to Europe is around 200 MiB.
You can use `geo-coding-cli` utility to produce your own files for particular Earth regions or for the full planet.
