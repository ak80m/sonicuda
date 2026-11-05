# Sonicuda

Real-time GPU-accelerated audio processing engine
written in Rust and CUDA C++.

## Goals

- Mix multiple audio streams in real time.
- Accelerate audio processing using NVIDIA CUDA.
- Support compressed audio formats.
- Explore low-latency GPU processing architectures.
- Compare CPU and GPU implementations.

## Architecture

- Rust: Application and audio pipeline.
- CUDA C++: GPU kernels and device operations.
- C ABI: Rust/CUDA interoperability.

## Development

Requirements:

- Rust (rustup)
- Cargo
- CMake
- NVIDIA CUDA Toolkit (when developing CUDA components)

## Build

```bash
cargo build --workspace