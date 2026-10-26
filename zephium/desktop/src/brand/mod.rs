//! Product branding configuration — centralised so every crate can import
//! `zephium_desktop::brand` and get the current product name, identifiers,
//! URLs, and copy without having to hard‑code strings.

use serde::Serialize;

/// The product name shown in About, installer, and window titles.
pub const PRODUCT_NAME: &str = "[YOUR_BROWSER_NAME]";

/// The bundle identifier used by Tauri, stores, and updater metadata.
pub const BUNDLE_IDENTIFIER: &str = "app.[your_bundle_id]";

/// Short tagline shown alongside the product name.
pub const TAGLINE: &str = "A fast, privacy-focused desktop browser with a built-in AI agent";

/// Official homepage URL (update with your product’s site).
pub const HOMEPAGE_URL: &str = "https://[your-website].app";

/// Support / feedback email.
pub const SUPPORT_EMAIL: &str = "support@[your-website].app";

/// User‑agent suffix appended to HTTP requests made by the browser.
pub const USER_AGENT_SUFFIX: &str = concat!("[YOUR_BROWSER_NAME]/", env!("CARGO_PKG_VERSION"));

/// Installer metadata fields (populate in CI or build scripts).
#[derive(Serialize, Clone, Debug)]
pub struct InstallerMetadata {
    /// Reverse-domain style identifier.
    pub identifier: String,
    /// Human‑readable version string.
    pub version: String,
    /// Copyright line shown in installer panels.
    pub copyright: String,
    /// Publisher name shown in installer.
    pub publisher: String,
    /// Product name shown in installer.
    pub display_name: String,
}

impl InstallerMetadata {
    pub fn new(version: &str) -> Self {
        Self {
            identifier: BUNDLE_IDENTIFIER.to_owned(),
            version: version.to_owned(),
            copyright: format!("Copyright © 2026 Apdevlab"),
            publisher: "Apdevlab".to_owned(),
            display_name: PRODUCT_NAME.to_owned(),
        }
    }
}

/// Update-channel / updater configuration placeholders.
/// Replace the endpoints below with your own update server before shipping.
pub mod updater {
    /// Update public key — generate a new Ed25519 key or reuse an existing one.
    /// **Never commit a real private key; keep it in CI secrets only.**
    pub const PUBLIC_KEY: &str = "YOUR_UPDATER_PUBLIC_KEY_BASE64_GOES_HERE";

    /// Endpoint that returns the latest release manifest.
    /// Must return JSON compatible with Tauri’s update spec.
    pub const LATEST_JSON_ENDPOINT: &str = "https://[your-updater-domain].com/latest.json";

    /// URL of the auto‑updater binary artefacts.
    pub const BINARY_ARTIFACTS_URL: &str = "https://[your-updater-domain].com/update/{platform}/{arch}/{version}";
}

/// CSS-friendly colour name for the accent used in design tokens.
/// Use this variable name in your Svelte components so the colour can be
/// switched globally (light/dark themes share the same variable reference).
pub const ACCENT_COLOUR_VAR: &str = "--accent-colour";
/// Default accent colour (change before final release). Format: `#rrggbb`.
pub const DEFAULT_ACCENT_COLOUR: &str = "#3b82f6";

/// Locales / i18n namespace prefix — keep aligned with `frame/src/i18n`.
pub const I18N_NAMESPACE: &str = "app";