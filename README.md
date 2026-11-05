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

## Check environment

```bash
echo "=== OS ==="
cat /etc/os-release

echo "=== CPU ==="
lscpu | grep -E 'Model name|Architecture|CPU\(s\)'

echo "=== RAM ==="
free -h

echo "=== NVIDIA GPU ==="
lspci | grep -iE 'nvidia|vga|3d'

echo "=== NVIDIA DRIVER ==="
nvidia-smi

echo "=== CUDA COMPILER ==="
nvcc --version

echo "=== RUST ==="
rustc --version
cargo --version

echo "=== C++ COMPILER ==="
g++ --version | head -1

echo "=== CMAKE ==="
cmake --version | head -1
```

## Build

```bash
cargo build --workspace
```
