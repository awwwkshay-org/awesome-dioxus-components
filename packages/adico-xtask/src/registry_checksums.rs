//! `cargo xtask registry checksums --write`: recomputes and writes every
//! `registry.json` file's SHA-256 checksum from its actual on-disk content,
//! so a multi-file source sweep doesn't require hand-computing and
//! transcribing 60+ hashes. Does not replace `registry validate`'s role as
//! the actual checksum gate (see `packages/adico-registry-core/src/lib.rs`'s
//! `ChecksumMismatch` path) -- this only removes the manual step feeding it.
//!
//! Implemented as a targeted string substitution rather than a full
//! deserialize/reserialize round-trip: `serde_json::Value` in this workspace
//! has no `preserve_order` feature enabled, so re-serializing the whole
//! manifest would alphabetize every object's keys and produce a diff across
//! the entire file instead of just the changed checksums. Each `RegistryFile`
//! block is keyed by its `source` path, which is unique across the manifest
//! (verified: 71 files, 71 unique `source` values as of this module's
//! authoring) -- so each checksum is replaced in place by locating its
//! block's `source` line and updating the next `checksum` value after it,
//! leaving every other byte of the file untouched. A no-op run (no source
//! files changed since the checksums were last written) leaves the file
//! byte-identical.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use adico_registry_core::RegistryManifest;

pub fn write(root: &Path) -> Result<(), String> {
    let official_root = root.join("registry");
    let manifest_path = official_root.join("registry.json");
    let original = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let manifest: RegistryManifest = serde_json::from_str(&original)
        .map_err(|error| format!("registry manifest is invalid: {error}"))?;

    let mut updated = original.clone();
    let mut changed = Vec::new();

    for item in &manifest.items {
        for file in &item.files {
            let source_path = official_root.join(&file.source);
            let contents = fs::read(&source_path).map_err(|error| {
                format!(
                    "item {} references unreadable {}: {error}",
                    item.name,
                    source_path.display()
                )
            })?;
            let actual_checksum = format!("{:x}", Sha256::digest(&contents));
            if actual_checksum == file.checksum {
                continue;
            }

            let source_anchor = format!("\"source\": \"{}\"", file.source);
            let anchor_offset = updated.find(&source_anchor).ok_or_else(|| {
                format!(
                    "cannot locate source anchor for {} in {} (has the manifest been reformatted?)",
                    file.source,
                    manifest_path.display()
                )
            })?;

            let old_checksum_field = format!("\"checksum\": \"{}\"", file.checksum);
            let search_window_start = anchor_offset;
            let search_window_end = (anchor_offset + source_anchor.len() + 400).min(updated.len());
            let window = &updated[search_window_start..search_window_end];
            let field_offset_in_window = window.find(&old_checksum_field).ok_or_else(|| {
                format!(
                    "cannot locate checksum field for {} near its source anchor in {}",
                    file.source,
                    manifest_path.display()
                )
            })?;
            let absolute_field_offset = search_window_start + field_offset_in_window;

            let new_checksum_field = format!("\"checksum\": \"{actual_checksum}\"");
            updated.replace_range(
                absolute_field_offset..absolute_field_offset + old_checksum_field.len(),
                &new_checksum_field,
            );
            changed.push(format!(
                "{} ({} -> {})",
                file.source, file.checksum, actual_checksum
            ));
        }
    }

    if updated != original {
        fs::write(&manifest_path, &updated)
            .map_err(|error| format!("cannot write {}: {error}", manifest_path.display()))?;
    }

    if changed.is_empty() {
        println!("registry checksums --write: no checksums needed updating");
    } else {
        println!(
            "registry checksums --write: updated {} checksum(s):\n  {}",
            changed.len(),
            changed.join("\n  ")
        );
    }
    Ok(())
}
