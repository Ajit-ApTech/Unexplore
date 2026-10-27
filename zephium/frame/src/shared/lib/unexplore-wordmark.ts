// Unexplore wordmark — replaces Zephium WORDMARK
// The actual SVG path is in public/unexplore-wordmark.svg
// This module provides the layout metrics for the new tab page.

/** The wordmark's layout metrics for the new tab page.
 *  The view box coordinates and scale match the original Zephium WORDMARK
 *  so the existing layout code continues to work. */
export const WORDMARK = {
  /** The file's view box. */
  x: 105,
  y: 90,
  width: 585,
  height: 84,
  /** The scale the file draws its letters at. */
  scaleX: 0.4,
  scaleY: 0.400901,
  /** Path data (not used - we use the SVG file directly via <img>) */
  d: "",
} as const;

/** Wordmark as an inline SVG for when we need it in components */
export const WORDMARK_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 585 84" fill="currentColor" aria-hidden="true">
  <text x="105" y="60" font-family="system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI Variable Text', 'Segoe UI', sans-serif" font-size="84" font-weight="600" letter-spacing="-0.02em">Unexplore</text>
</svg>`;
