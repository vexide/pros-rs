# pros-async

Tiny async runtime and robot traits for `pros-rs`.

![Maintained: no](https://img.shields.io/maintenance/no/2024)

> [!important]
> pros-rs is deprecated and unmaintained; new projects should instead use
> [`vexide`](https://crates.io/crates/vexide), a similar library with features like differential
> uploading and `std` support.

The async executor supports spawning tasks and blocking on futures.
It has a reactor to improve the performance of some futures.
FreeRTOS tasks can still be used, but it is recommended to use only async tasks for performance.
