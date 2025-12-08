v0.7.0 (in development)
-----------------------
- Increased MSRV to 1.88
- Moved example program to a workspace crate and removed "examples" feature
- Updated `json5` to 1.0.0
- Removed `Json5Syntax` variant of `DeserializeError`

v0.6.0 (2025-11-14)
-------------------
- Update `ron` to 0.12.0
- Increased MSRV to 1.82

v0.5.0 (2025-08-29)
-------------------
- Update `ron` to 0.11.0
- The payload of the `Ron` variant of `DeserializeError` is now wrapped in a
  `Box`

v0.4.0 (2025-07-14)
-------------------
- Increased MSRV to 1.80
- Update `toml` to 0.9.0

v0.3.0 (2025-04-14)
-------------------
- Update `ron` to 0.10.1

v0.2.2 (2025-02-10)
-------------------
- Update `strum` to 0.27.0

v0.2.1 (2025-01-13)
-------------------
- Increased MSRV to 1.70
- Document that `Format::extensions()` returns results in lexicographic order
- Update dependencies

v0.2.0 (2023-12-22)
-------------------
- Files opened by the `load()` & `dump()` methods & functions are now wrapped
  in `std::io::BufReader`/`std::io::BufWriter`
- Added `Flush` variant to `DumpError`

v0.1.0 (2023-10-30)
-------------------
Initial release
