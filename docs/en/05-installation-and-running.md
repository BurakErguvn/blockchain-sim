# Installation and Running

## Prerequisites

- Rust (stable toolchain)
- Cargo

You can verify installation with:

```bash
rustc --version
cargo --version
```

## Running the Application

1. Clone the repository.
2. Enter the project directory.
3. Start the simulation:

```bash
cargo run
```

## Validation Commands

### Test suite

```bash
cargo test
```

### Build check

```bash
cargo check
```

## Runtime Notes

- The simulation is interactive through command-line input.
- Observed behavior depends on block interval, network size, and difficulty settings.
- Proof of Work and validator selection are intentionally simplified for instructional use.
