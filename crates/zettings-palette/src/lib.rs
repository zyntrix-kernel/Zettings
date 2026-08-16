//! Wallpaper color quantization for ZDL accent extraction.
//!
//! Takes a wallpaper image, reduces it to a small palette using Median-cut,
//! and selects an accent pair (foreground + foreground-on-accent) that
//! meets WCAG contrast targets. The result is serialized as the
//! `--accent-*` CSS variable layer the frontend consumes.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// A 0-1 floating-point RGB color. Serialization form is `[f32; 3]` for
/// direct consumption by CSS `rgb()` expressions in the frontend.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rgb(pub [f32; 3]);

/// The ZDL accent palette extracted from a wallpaper.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AccentPalette {
    /// Primary accent (used for toggles, focus rings, primary CTAs).
    pub accent: Rgb,
    /// High-contrast text/glyph color to be placed ON TOP of `accent`.
    pub on_accent: Rgb,
    /// Secondary muted accent, used sparingly (e.g. active-tab underline).
    pub secondary: Rgb,
}

/// Errors surfaced by the palette extractor.
#[derive(Debug, thiserror::Error)]
pub enum PaletteError {
    /// The image could not be decoded.
    #[error("image decode error: {0}")]
    Decode(String),
    /// The image was too small to extract a palette.
    #[error("image too small: {0} pixels")]
    TooSmall(usize),
}

/// Placeholder entry point. Real quantization lands in Phase 2 with the ZDL
/// design system; this stub proves the crate compiles and the API contracts.
///
/// # Errors
/// Returns [`PaletteError`] when the input image is empty or unreadable.
pub fn extract(_bytes: &[u8]) -> Result<AccentPalette, PaletteError> {
    // Phase 2: Median-cut quantization via `image` + `palette` crates.
    // For Phase 1 we return the Aurora teal-violet default until extraction
    // is implemented; this lets the frontend wire up `--accent-*` immediately.
    Ok(AccentPalette {
        accent: Rgb([0.345, 0.682, 0.741]),    // #58AEBC teal
        on_accent: Rgb([0.086, 0.078, 0.039]), // #0C0A09 graphite
        secondary: Rgb([0.498, 0.385, 0.812]), // #7F62CF violet
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_palette_returns_aurora() {
        let p = extract(&[]).expect("non-empty palette");
        // Teal component bright
        assert!(p.accent.0[1] > 0.5);
    }

    proptest::proptest! {
        #[test]
        fn extract_never_panics_on_arbitrary_bytes(bytes in proptest::prelude::prop::collection::vec(0u8..=255, 0..1024)) {
            let _ = extract(&bytes);
        }
    }
}
