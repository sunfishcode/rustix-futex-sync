<div align="center">
  <h1><code>rustix-futex-sync</code></h1>

  <p>
    <strong>Linux futex-based synchronization</strong>
  </p>

  <p>
    <a href="https://github.com/sunfishcode/rustix-futex-sync/actions?query=workflow%3ACI"><img src="https://github.com/sunfishcode/rustix-futex-sync/workflows/CI/badge.svg" alt="Github Actions CI Status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/206238-general"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <a href="https://crates.io/crates/rustix-futex-sync"><img src="https://img.shields.io/crates/v/rustix-futex-sync.svg" alt="crates.io page" /></a>
    <a href="https://docs.rs/rustix-futex-sync"><img src="https://docs.rs/rustix-futex-sync/badge.svg" alt="docs.rs docs" /></a>
  </p>
</div>

Linux futex-based implementations of [`Mutex`], [`RwLock`], [`Condvar`],
[`Once`], and [`OnceLock`], as well as [`RawMutex`], [`RawRwLock`], and
[`RawCondvar`], derived from the futex code in std, factored out to
a standalone `no_std` crate using [`rustix`] to do the futex and [`lock_api`]
to provide most of the public API.

`lock_api` does not support poisoning, so support for poisoning is omitted.

[`Mutex`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.Mutex.html
[`RwLock`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.RwLock.html
[`Condvar`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.Condvar.html
[`Once`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.Once.html
[`OnceLock`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.OnceLock.html
[`RawMutex`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.RawMutex.html
[`RawRwLock`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.RawRwLock.html
[`RawCondvar`]: https://docs.rs/rustix-futex-sync/latest/rustix_futex_sync/type.RawCondvar.html
[`rustix`]: https://github.com/bytecodealliance/rustix#readme
[`lock_api`]: https://crates.io/crates/lock_api
