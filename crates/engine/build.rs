//! Optional build input: with `CRAFT_FONTS_DIR=<craft-fonts checkout>`, embed every font in its
//! `fonts/manifest.txt` as `CRAFT_FONTS` (see src/fonts.rs). Unset, `CRAFT_FONTS` is empty.
//! The embedding is done by `craft-fonts-build` (685ks/craft-fonts, docs/integration.md) in its
//! legacy mode, which emits the `CraftFont { family, style, scripts, bytes }` initialisers that
//! src/fonts.rs defines; craft-fonts itself is never a Cargo dependency, and this script only reads
//! a local directory (no network).
//!
//! wasm32 embeds BIZ UDPGothic Regular (~4.5 MB) and Noto Sans Thai (~214 KiB), to keep the
//! `.wasm` within the web size budget (Cloudflare serves at most 25 MiB per file); the export
//! watermark falls back to it there. Native builds embed every font.
use std::path::{Path, PathBuf};

/// The faces the web build embeds.
const WEB_FONTS: &[(&str, &str)] = &[("BIZ UDPGothic", "Regular"), ("Noto Sans Thai", "Regular")];

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_REQUIRED");
    let required = std::env::var_os("CRAFT_FONTS_REQUIRED").is_some();
    let wasm = std::env::var("CARGO_CFG_TARGET_ARCH").is_ok_and(|a| a == "wasm32");
    let dir = std::env::var_os("CRAFT_FONTS_DIR").filter(|d| !d.is_empty()).map(PathBuf::from).map(resolve);
    let mut builder = craft_fonts_build::Builder::new().legacy(true).web_fonts(WEB_FONTS).required(required);
    builder = match &dir {
        Some(dir) => builder.dir(dir.clone()),
        None => builder.no_checkout(),
    };
    match &dir {
        Some(dir) => {
            if let Err(e) = required_faces(dir, wasm)
                && required
            {
                println!("cargo::error=CRAFT_FONTS_DIR={}: {e}", dir.display());
            }
        }
        None if required => println!("cargo::error=CRAFT_FONTS_REQUIRED is set but CRAFT_FONTS_DIR is missing"),
        None => {
            // Without them the app still runs and every language is still selectable, so Chinese
            // and Japanese just paint boxes — the one failure a user cannot diagnose from what
            // they see.
            println!(
                "cargo::warning=no craft-fonts embedded: Chinese and Japanese text will render as empty boxes. \
                 Clone https://github.com/685ks/craft-fonts next to this repo and rebuild with \
                 CRAFT_FONTS_DIR=../craft-fonts"
            );
        }
    }
    builder.emit();
}

/// Resolve a relative `CRAFT_FONTS_DIR` against both roots a caller might mean: a build script's
/// cwd is the *package* root (`crates/engine`), while the `../craft-fonts` the README and
/// craft-fonts' docs use is written relative to the *workspace* root and would otherwise resolve
/// to `crates/craft-fonts`.
fn resolve(dir: PathBuf) -> PathBuf {
    if dir.is_absolute() {
        return dir;
    }
    let package = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let workspace = package.parent().and_then(Path::parent).unwrap_or(package.as_path());
    let candidates = [workspace.join(&dir), package.join(&dir)];
    candidates.iter().find(|candidate| candidate.is_dir()).cloned().unwrap_or_else(|| candidates[0].clone())
}

/// The UI faces a release must carry (Japanese, Simplified Chinese natively, Thai): a checkout
/// without them is an error when `CRAFT_FONTS_REQUIRED` is set.
fn required_faces(dir: &Path, wasm: bool) -> Result<(), String> {
    let manifest = dir.join("fonts/manifest.txt");
    let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let entries = craft_fonts_build::parse_manifest(&text).map_err(|e| e.to_string())?;
    let has = |family: &str, script: Option<&str>| {
        entries.iter().any(|e| e.family == family && e.style == "Regular" && script.is_none_or(|s| e.scripts.iter().any(|x| x == s)))
    };
    if !has("Noto Sans Thai", Some("Thai")) {
        return Err("required Thai UI face Noto Sans Thai Regular (Thai) is missing from fonts/manifest.txt".into());
    }
    if !has("BIZ UDPGothic", None) {
        return Err("required Japanese UI face BIZ UDPGothic Regular is missing from fonts/manifest.txt".into());
    }
    if !wasm && !has("Noto Sans CJK SC", Some("Hans")) {
        return Err("required Simplified Chinese UI face Noto Sans CJK SC Regular (Hans) is missing from fonts/manifest.txt".into());
    }
    Ok(())
}
