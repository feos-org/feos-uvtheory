# FeOs - uv-theory

Implementation of the uv-theory[^vanwesten2021][^vanwesten2021] within the FeOs project. This project contains a Rust implementation as well as bindings to Python.

> The uv-theory leads to an accurate description of Mie fluids, Mie chain fluids, and mixtures.

## Usage in Python

If you want to use `feos-uvtheory` in Python, take a look at the [`feos`-repository](https://github.com/feos-org/feos). `feos` contains multiple equation of state implementations in a single, easy-to-use Python package.

## FeOs

> FeOs is a framework for equations of state and classical density function theory

You can learn more about the principles behind `FeOs` [here](https://feos-org.github.io/feos/).

## Installation

Add this to your `Cargo.toml` (does not work, not yet published on crates.io)

```toml
[dependencies]
feos-uvtheory = "0.1"
```

## Test building python wheel

From within a Python virtual environment with `maturin` installed, type

```
maturin build --release --out dist --no-sdist -m build_wheel/Cargo.toml
```

[^vanwesten2021]: [T. van Westen, and J. Gross (2021). *J. Chem. Phys.* 155, 244501 (2021) ](https://doi.org/10.1063/5.0073572)