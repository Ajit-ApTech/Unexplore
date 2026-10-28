// Product branding configuration — centralised so every Svelte component can
// import `shared/brand` and get the current product name, identifiers,
// URLs, and copy without having to hard-code strings.

/** The product name shown in About, window titles, and UI copy. */
export const PRODUCT_NAME = "Unexplore";

/** The bundle identifier used by Tauri, stores, and updater metadata. */
export const BUNDLE_IDENTIFIER = "app.unexplore";

/** Short tagline shown alongside the product name. */
export const TAGLINE = "A fast, privacy-focused desktop browser with a built-in AI agent";

/** Official homepage URL (update with your product's site). */
export const HOMEPAGE_URL = "https://unexplore.app";

/** Support / feedback email. */
export const SUPPORT_EMAIL = "support@unexplore.app";

/** User-agent suffix appended to HTTP requests made by the browser. */
export const USER_AGENT_SUFFIX = `${PRODUCT_NAME}/__VERSION__`;

/** Update-channel / updater configuration placeholders. */
export const UPDATER = {
  /** Update public key — generate a new Ed25519 key or reuse an existing one.
   *  **Never commit a real private key; keep it in CI secrets only.** */
  PUBLIC_KEY: "YOUR_UPDATER_PUBLIC_KEY_BASE64_GOES_HERE",

  /** Endpoint that returns the latest release manifest. */
  LATEST_JSON_ENDPOINT: "https://updates.unexplore.app/latest.json",

  /** URL of the auto-updater binary artefacts. */
  BINARY_ARTIFACTS_URL: "https://updates.unexplore.app/update/{platform}/{arch}/{version}",
};

/** CSS-friendly colour name for the accent used in design tokens. */
export const ACCENT_COLOUR_VAR = "--accent-colour";

/** Default accent colour (change before final release). Format: `#rrggbb`. */
export const DEFAULT_ACCENT_COLOUR = "#3b82f6";

/** Locales / i18n namespace prefix — keep aligned with frame/src/i18n. */
export const I18N_NAMESPACE = "app";

/** Build-time metadata (injected by build script). */
export interface BuildMetadata {
  version: string;
  revision: string;
}

/** Runtime-initialized build metadata. Populated by the build script. */
let runtimeMetadata: BuildMetadata | null = null;

export function setBuildMetadata(meta: BuildMetadata) {
  runtimeMetadata = meta;
}

export function getBuildMetadata(): BuildMetadata | null {
  return runtimeMetadata;
}
