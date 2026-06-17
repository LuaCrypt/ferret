# Contributing

Ferret aims to be a strong, practical, free OSS Lua VM obfuscator. Contributions
should improve real protection, correctness, test coverage, or usability without
making dishonest claims about what open source obfuscation can guarantee.

## Priorities

- Keep output VM-only. Do not add source-reconstruction or `load` fallback paths.
- Prefer clear rejection for unsupported Lua over silently weaker output.
- Improve the compiler, bytecode format, runtime VM, encryption layers, tests, and
  diagnostics.
- Keep generated protection honest: static hardening is in scope; stealthy
  anti-debugging, malware evasion, and false "unbreakable" claims are not.

## Code Standards

- Keep every tracked `.rs`, `.lua`, and `.md` implementation file at 300 lines or
  fewer.
- Put crate internals in subfolders under `src/`; avoid dumping large modules in
  crate roots.
- Keep public behavior deterministic when `--seed` is provided.
- Return clear errors for unsupported syntax or unsafe inputs.
- Do not introduce a new dependency unless it meaningfully reduces complexity or
  improves correctness.

## Tests

Before opening a PR, run:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
```

Add tests when changing parser support, IR lowering, bytecode encoding, runtime
semantics, CLI behavior, metadata, or rejection rules. If a Lua feature becomes
supported, include a fixture that compares original output against obfuscated VM
output.

## Pull Requests

- Keep PRs focused and explain the protection or correctness improvement.
- Update README or metadata when behavior or support scope changes.
- Include any known limitations honestly in the PR description.
- Do not submit code intended to hide abuse, bypass detection, or misrepresent
  Ferret's guarantees.
