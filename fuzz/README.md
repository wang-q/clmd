# Fuzz Testing for clmd

This directory contains fuzz testing targets for the clmd Markdown parser using `cargo-fuzz`.

## Setup

Install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

## Running Fuzz Tests

### Parse Fuzz Test

Tests the parser with random input:

```bash
cargo fuzz run fuzz_parse
```

### Render Fuzz Test

Tests rendering with random input:

```bash
cargo fuzz run fuzz_render
```

## Adding Corpus

To add seed inputs for the fuzzer, place files in the `corpus/` directory:

```bash
mkdir -p fuzz/corpus/fuzz_parse
echo "# Hello World" > fuzz/corpus/fuzz_parse/simple.md
```

## Finding Bugs

When the fuzzer finds a crash, it will save the crashing input to:
- `fuzz/artifacts/fuzz_parse/` for parse crashes
- `fuzz/artifacts/fuzz_render/` for render crashes

You can reproduce a crash with:

```bash
cargo fuzz run fuzz_parse fuzz/artifacts/fuzz_parse/crash-<hash>
```

## Continuous Fuzzing

For continuous fuzzing in CI, consider using:
- [ClusterFuzzLite](https://google.github.io/clusterfuzzlite/)
- [OSS-Fuzz](https://google.github.io/oss-fuzz/) for open source projects
