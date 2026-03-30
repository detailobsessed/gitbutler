use std::path::Path;

fn main() {
    // When the embedded-frontend feature is active, rust-embed requires the
    // dist directory to exist at compile time. Fail with clear instructions
    // rather than writing placeholder files into the source tree.
    if std::env::var("CARGO_FEATURE_EMBEDDED_FRONTEND").is_ok() {
        let dist = Path::new("../../apps/desktop/build");
        if !dist.exists() {
            // Create an empty directory so rust-embed can compile. The embedded
            // server will serve no frontend assets until `pnpm build` is run:
            //
            //   VITE_BUILD_TARGET=web VITE_BUTLER_API_BASE_URL=/api \
            //   pnpm --filter @gitbutler/desktop build
            //
            // This allows `cargo check --workspace` to succeed without a prior
            // frontend build (e.g. in CI steps that only lint Rust code).
            let _ = std::fs::create_dir_all(dist);
            println!(
                "cargo:warning=Frontend assets not found at apps/desktop/build/. \
                      The embedded server will have no UI until you run `pnpm build`."
            );
            println!("cargo:rustc-env=EMBEDDED_FRONTEND_HASH=0");
            // Always watch the dist directory so Cargo reruns this script once
            // `pnpm build` creates it and populates it with assets.
            println!("cargo:rerun-if-changed=../../apps/desktop/build");
            return;
        }

        // Emit rerun-if-changed for every file in dist so this build script
        // is re-run whenever `pnpm build` updates any asset.
        emit_rerun_if_changed_recursive(dist);

        // Compute a hash of the entire dist tree and emit it as a rustc-env
        // variable. When the hash changes, Cargo recompiles the crate, which
        // causes rust-embed to re-embed the updated files.
        let hash = dir_hash(dist);
        println!("cargo:rustc-env=EMBEDDED_FRONTEND_HASH={hash}");
    }
}

/// A fast, order-independent hash of every file path + contents under `dir`.
fn dir_hash(dir: &Path) -> u64 {
    use std::collections::BTreeMap;
    use std::hash::Hash as _;

    // Collect path → contents into a sorted map so the hash is stable
    // regardless of directory traversal order.
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    collect_files(dir, dir, &mut files);

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    files.hash(&mut hasher);
    std::hash::Hasher::finish(&hasher)
}

fn collect_files(root: &Path, dir: &Path, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, out);
        } else {
            let key = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            let contents = std::fs::read(&path).unwrap_or_default();
            out.insert(key, contents);
        }
    }
}

fn emit_rerun_if_changed_recursive(dir: &Path) {
    println!("cargo:rerun-if-changed={}", dir.display());
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                emit_rerun_if_changed_recursive(&path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
