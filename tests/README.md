# Test Assets

This directory contains Lua assets used to validate and benchmark Ferret.

- `fixtures/` contains small supported programs used by automated VM execution
  tests.
- `scripts/` contains the broader Lua corpus imported from the original project.
- `lua54_conformance/` contains focused Lua 5.4 semantic fixtures imported from
  the original project.
- `vendor/lua-5.4.8-tests/` contains the upstream Lua 5.4.8 test files used as a
  stress corpus for coverage and benchmarking.

The OSS VM intentionally rejects unsupported Lua constructs rather than falling
back to weaker source reconstruction. Benchmarks over these assets report both
supported and rejected files so coverage gaps stay visible.
