v0.3.0 (in development)
-----------------------
- Increased MSRV to 1.69

v0.2.0 (2023-12-22)
-------------------
- Files opened by the `load()` & `dump()` methods & functions are now wrapped
  in `std::io::BufReader`/`std::io::BufWriter`
- Added `Flush` variant to `DumpError`

v0.1.0 (2023-10-30)
-------------------
Initial release
