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
