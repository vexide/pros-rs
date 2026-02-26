# pros-panic

Panic handler implementation for [`pros-rs`](https://crates.io/crates/pros-rs).

Supports printing a backtrace when running in the simulator.
If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

![Maintained: no](https://img.shields.io/maintenance/no/2024)

> [!important]
> pros-rs is deprecated and unmaintained; new projects should instead use
> [`vexide`](https://crates.io/crates/vexide), a similar library with features like differential
> uploading and `std` support.
