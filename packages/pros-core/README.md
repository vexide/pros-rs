# pros-core

Low level core functionality for [`pros-rs`](https://crates.io/crates/pros).

![Maintained: no](https://img.shields.io/maintenance/no/2024)

> [!important]
> pros-rs is deprecated and unmaintained; new projects should instead use
> [`vexide`](https://crates.io/crates/vexide), a similar library with features like differential
> uploading and `std` support.

This core crate is used in all other crates in the pros-rs ecosystem.

Included in this crate:
- Global allocator
- Errno handling
- Serial terminal printing
- No-std `Instant`s
- Synchronization primitives
- FreeRTOS task management
