# Async Migration Plan — Async-within-Sync

**Goal:** Speed up dependency resolution and JAR downloads by adding a global Tokio runtime and using it to run concurrent network requests, while keeping all public function signatures synchronous.

---

## Strategy

- Create a single global `tokio::runtime::Runtime` via `std::sync::LazyLock`.
- Inside network-heavy functions, call `RUNTIME.block_on(async { ... })` to run concurrent work.
- The app stays 100% synchronous at the public API level (`fn main`, no `async` propagation).
- No changes to error types, CLI args, lock file format, or tests.

---

## Changes by File

### `Cargo.toml`
- Remove `blocking` feature from `reqwest`.
- Ensure `tokio` has `rt-multi-thread` and `macros` features.

### `src/maven_central/get_maven.rs`
- Add a global `LazyLock<reqwest::Client>` for connection pooling.
- Add a global `LazyLock<tokio::runtime::Runtime>`.
- Replace `reqwest::blocking::get(url)` with:
  ```rust
  RUNTIME.block_on(async {
      CLIENT.get(&url).send().await
  })
  ```
- Import swaps: `reqwest::blocking::Response` → `reqwest::Response`.

### `src/maven_central/get.rs`
- `get_pom`, `get_jar`, `get_artifact_metadata` stay synchronous (callers unchanged).
- Add async internal helpers (`get_pom_async`, `get_jar_async`, etc.) that the batch-fetch code in `pom_list.rs` can use directly inside `block_on`.

### `src/maven_central/pom/pom_list.rs` — **Biggest change**
- `resolve_related_poms` stays recursive-sync.
- When iterating a POM's transitive dependencies, collect all child POM IDs, then fetch them in parallel with a single `block_on` + `join_all`:

```rust
let child_futs: Vec<_> = child_ids.iter().map(|child_id| {
    get_pom_async(child_id, cache.clone())
}).collect();
let child_poms = RUNTIME.block_on(async { join_all(child_futs).await });
```

- `Cache` stays `HashMap` (single-threaded) — no `Mutex` needed since all concurrent work happens inside one `block_on` on the same thread pool, but we need `Arc` for sharing:
  - `Cache` becomes `Arc<std::sync::Mutex<HashMap<u64, PomState>>>`.
  - Or keep it simpler: within a single `block_on`, the parent POM's deps are fetched concurrently (same runtime thread pool), so use `Arc<tokio::sync::Mutex>`.

### `src/lock_file/lock_file.rs`
- `validate_current_packages` stays sync.
- The second pass (download missing JARs) uses a single `block_on` + `join_all`:

```rust
let results: Vec<Result<(), LockFileError>> = RUNTIME.block_on(async {
    let futs: Vec<_> = map.iter().map(|(key, value)| {
        let key = key.clone();
        let lib = lib.to_path_buf();
        let url = value.url.clone();
        async move {
            let bin = fetch_bin_async(&url).await?;
            tokio::fs::write(lib.join(&key), bin).await?;
            Ok::<_, LockFileError>(())
        }
    }).collect();
    join_all(futs).await
});
```

- Fix `file_stem()` → `file_name()` bug in the first pass (orphan removal).

### Files Unchanged
- `src/main.rs` — stays `fn main`.
- `src/lazy_java.rs` — stays `fn execute`.
- `src/maven_central/maven_error.rs` — `reqwest::Error` is the same type.
- `src/lock_file/lock_file_error.rs` — same.
- All test files — no behavioral changes expected.

---

## Performance Gains

| Operation | Before | After (est.) |
|-----------|--------|--------------|
| Add Guava (all transitive POMs) | ~3–5s sequential | ~1–2s (concurrent sibling fetches) |
| JAR download (10 JARs @ 200ms each) | ~2s sequential | ~200ms concurrent |
| `sync` with all JARs present | ~0ms | ~0ms (no network) |
| `sync` with missing JARs | 200ms × N sequential | ~200ms batch |

---

## Trade-offs

- Less aggressive parallelism than full async could achieve (deeper interleaving of independent branches), but much simpler to implement and maintain.
- Thread pool overhead for small operations (negligible with global runtime reuse).
- `block_on` from sync code blocks the calling thread — fine for a CLI tool that's idle while waiting on network anyway.
