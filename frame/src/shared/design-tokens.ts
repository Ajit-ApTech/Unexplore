// Design token system — TypeScript wrapper around CSS custom properties
// so components can access tokens programmatically for complex logic.
// The actual values live in styles/tokens.css and styles/axes/*.css.

/**
 * Color tokens (semantic names, not hue names).
 * Dark theme is the base; light theme mirrors by luminance.
 */
export const colors = {
  // Base surfaces (luminance ladder)
  canvas: "var(--color-canvas)",
  page: "var(--color-page)",
  chrome: "var(--color-chrome)",
  surface: "var(--color-surface)",
  raised: "var(--color-raised)",
  float: "var(--color-float)",
  menu: "var(--color-menu)",

  // Fills (interactive states)
  fill: "var(--color-fill)",
  fillHover: "var(--color-fill-hover)",
  fillActive: "var(--color-fill-active)",
  fillPressed: "var(--color-fill-pressed)",
  fillStrong: "var(--color-fill-strong)",
  card: "var(--color-card)",
  tile: "var(--color-tile)",
  tileHover: "var(--color-tile-hover)",
  address: "var(--color-address)",
  addressHover: "var(--color-address-hover)",

  // Borders
  border: "var(--color-border)",
  borderStrong: "var(--color-border-strong)",

  // Row states (sidebar)
  rowHover: "var(--row-hover)",
  rowActive: "var(--row-active)",
  rowPressed: "var(--row-pressed)",
  rowRim: "var(--row-rim)",

  // Controls
  control: "var(--color-control)",
  controlHover: "var(--color-control-hover)",
  controlPressed: "var(--color-control-pressed)",
  onControl: "var(--color-on-control)",
  onControlStrong: "var(--color-on-control-strong)",

  // Fields
  field: "var(--color-field)",
  fieldHover: "var(--color-field-hover)",
  fieldRing: "var(--color-field-ring)",

  // Raised/slider/switch
  raise: "var(--color-raise)",
  raiseActive: "var(--color-raise-active)",
  thumb: "var(--color-thumb)",
  track: "var(--color-track)",
  trackOn: "var(--color-track-on)",
  switchThumb: "var(--color-switch-thumb)",
  switchThumbOff: "var(--color-switch-thumb-off)",
  switchGlass: "var(--color-switch-glass)",
  switchRim: "var(--color-switch-rim)",

  // Lit (primary actions, on states)
  lit: "var(--color-lit)",
  litHover: "var(--color-lit-hover)",
  onLit: "var(--color-on-lit)",
  litSoft: "var(--color-lit-soft)",
  ring: "var(--color-ring)",

  // Accent (follows space tint)
  accent: "var(--color-accent)",
  accentHover: "var(--color-accent-hover)",
  onAccent: "var(--color-on-accent)",
  accentSoft: "var(--color-accent-soft)",

  // Tints (space themes)
  tintGraphite: "var(--color-tint-graphite)",
  tintSky: "var(--color-tint-sky)",
  tintSage: "var(--color-tint-sage)",
  tintRose: "var(--color-tint-rose)",
  tintAmber: "var(--color-tint-amber)",
  tintTeal: "var(--color-tint-teal)",
  tintLavender: "var(--color-tint-lavender)",
  tintOrchid: "var(--color-tint-orchid)",

  // Status
  danger: "var(--color-danger)",
  success: "var(--color-success)",
  warning: "var(--color-warning)",
  info: "var(--color-info)",

  // Text
  text: "var(--color-text)",
  labelSecondary: "var(--color-label-secondary)",
  muted: "var(--color-muted)",
  faint: "var(--color-faint)",

  // Soft colours (personal markings, agent characters)
  softSky: "var(--color-soft-sky)",
  softMint: "var(--color-soft-mint)",
  softLemon: "var(--color-soft-lemon)",
  softPeach: "var(--color-soft-peach)",
  softRose: "var(--color-soft-rose)",
  softLilac: "var(--color-soft-lilac)",
  softSkyWash: "var(--color-soft-sky-wash)",
  softMintWash: "var(--color-soft-mint-wash)",
  softLemonWash: "var(--color-soft-lemon-wash)",
  softPeachWash: "var(--color-soft-peach-wash)",
  softRoseWash: "var(--color-soft-rose-wash)",
  softLilacWash: "var(--color-soft-lilac-wash)",
  softSkyEdge: "var(--color-soft-sky-edge)",
  softMintEdge: "var(--color-soft-mint-edge)",
  softLemonEdge: "var(--color-soft-lemon-edge)",
  softPeachEdge: "var(--color-soft-peach-edge)",
  softRoseEdge: "var(--color-soft-rose-edge)",
  softLilacEdge: "var(--color-soft-lilac-edge)",
  softSkyInk: "var(--color-soft-sky-ink)",
  softMintInk: "var(--color-soft-mint-ink)",
  softLemonInk: "var(--color-soft-lemon-ink)",
  softPeachInk: "var(--color-soft-peach-ink)",
  softRoseInk: "var(--color-soft-rose-ink)",
  softLilacInk: "var(--color-soft-lilac-ink)",

  // Code
  codeComment: "var(--color-code-comment)",
  codeKeyword: "var(--color-code-keyword)",
  codeString: "var(--color-code-string)",
  codeNumber: "var(--color-code-number)",
  codeFunction: "var(--color-code-function)",
  codeType: "var(--color-code-type)",
  codeProperty: "var(--color-code-property)",
  codeTag: "var(--color-code-tag)",
  codeAttribute: "var(--color-code-attribute)",
  codeChip: "var(--color-code-chip)",
  codeChipInk: "var(--color-code-chip-ink)",

  // Agent characters
  agentLead: "var(--color-agent-lead)",
  agentBrowser: "var(--color-agent-browser)",
  agentResearch: "var(--color-agent-research)",
  agentComputer: "var(--color-agent-computer)",
  agentConnection: "var(--color-agent-connection)",

  // Charts
  chart1: "var(--chart-1)",
  chart2: "var(--chart-2)",
  chart3: "var(--chart-3)",
  chart4: "var(--chart-4)",
  chart5: "var(--chart-5)",
  chart6: "var(--chart-6)",
  chart7: "var(--chart-7)",
  chart8: "var(--chart-8)",

  // Platform
  close: "var(--color-close)",
  closePressed: "var(--color-close-pressed)",
  onClose: "var(--color-on-close)",
  youtube: "var(--color-youtube)",
  onYoutube: "var(--color-on-youtube)",
};

/**
 * Border radius tokens (named by role, not size)
 */
export const radius = {
  swatch: "var(--radius-swatch)",
  inset: "var(--radius-inset)",
  row: "var(--radius-row)",
  controlCompact: "var(--radius-control-compact)",
  control: "var(--radius-control)",
  controlLarge: "var(--radius-control-large)",
  card: "var(--radius-card)",
  panel: "var(--radius-panel)",
  capsule: "var(--radius-capsule)",
  shine: "var(--radius-shine)",
  field: "var(--radius-field)",
  menu: "var(--radius-menu)",
  content: "var(--content-radius)",
};

/**
 * Shadow tokens
 */
export const shadows = {
  control: "var(--shadow-control)",
  controlStrong: "var(--shadow-control-strong)",
  field: "var(--shadow-field)",
  fieldFocus: "var(--shadow-field-focus)",
  raise: "var(--shadow-raise)",
  track: "var(--shadow-track)",
  switchTrack: "var(--shadow-switch-track)",
  thumb: "var(--shadow-thumb)",
  thumbGrab: "var(--shadow-thumb-grab)",
  raised: "var(--shadow-raised)",
  float: "var(--shadow-float)",
  overlay: "var(--shadow-overlay)",
  popover: "var(--shadow-popover)",
  menu: "var(--shadow-menu)",
  shine: "var(--shine)",
  shineTop: "var(--shine-top)",
  sheet: "var(--shadow-sheet)",
  badge: "var(--shadow-badge)",
};

/**
 * Motion tokens
 */
export const motion = {
  instant: "var(--motion-instant)",
  fast: "var(--motion-fast)",
  base: "var(--motion-base)",
  slow: "var(--motion-slow)",
  page: "var(--motion-page)",
  easeOut: "var(--ease-out)",
  easeSmooth: "var(--ease-smooth)",
  easeInOut: "var(--ease-in-out)",
  easePress: "var(--ease-press)",
  easeExit: "var(--ease-exit)",
  easeEmphasized: "var(--ease-emphasized)",
  easeSnap: "var(--ease-snap)",
  easeSpring: "var(--ease-spring)",
};

/**
 * Typography tokens
 */
export const typography = {
  fontSans: "var(--font-sans)",
  fontMono: "var(--font-mono)",
  textCaption: "var(--text-caption)",
  textLabel: "var(--text-label)",
  textBody: "var(--text-body)",
  textPageBody: "var(--text-page-body)",
  textPageTitle: "var(--text-page-title)",
  textTitle: "var(--text-title)",
  textDisplay: "var(--text-display)",
  textHeadline: "var(--text-headline)",
  textReading: "var(--text-reading)",
  textObjectTitle: "var(--text-object-title)",
  textOverviewLabel: "var(--text-overview-label)",
  textOverviewTitle: "var(--text-overview-title)",
  textOverviewFigure: "var(--text-overview-figure)",
  textTileTitle: "var(--text-tile-title)",
};

/**
 * Layout tokens
 */
export const layout = {
  controlHeight: "var(--control-height)",
  controlCompact: "var(--control-compact)",
  controlRegular: "var(--control-regular)",
  controlLarge: "var(--control-large)",
  fieldHeight: "var(--field-height)",
  controlInset: "var(--control-inset)",
  searchHeight: "var(--search-height)",
  rowSidebar: "var(--row-sidebar)",
  rowPage: "var(--row-page)",
  sidebarInset: "var(--sidebar-inset)",
  sidebarRowGap: "var(--sidebar-row-gap)",
  dockTile: "var(--dock-tile)",
  dockGap: "var(--dock-gap)",
  dockShelfGap: "var(--dock-shelf-gap)",
  sidebarRowText: "var(--sidebar-row-text)",
  sidebarRowWeight: "var(--sidebar-row-weight)",
  sidebarRowWeightCurrent: "var(--sidebar-row-weight-current)",
  titlebarHeight: "var(--titlebar-height)",
  trafficLightInset: "var(--traffic-light-inset)",
};

/**
 * Menu tokens
 */
export const menu = {
  pad: "var(--menu-pad)",
  row: "var(--menu-row)",
  itemInset: "var(--menu-item-inset)",
  gutter: "var(--menu-gutter)",
  separator: "var(--color-menu-separator)",
};

/**
 * Get a token value at runtime (for dynamic styles)
 */
export function getToken(name: string): string | undefined {
  if (typeof window === "undefined") return undefined;
  const style = getComputedStyle(document.documentElement);
  return style.getPropertyValue(name).trim() || undefined;
}

/**
 * Set a token value at runtime (for theme switching)
 */
export function setToken(name: string, value: string): void {
  if (typeof window !== "undefined") {
    document.documentElement.style.setProperty(name, value);
  }
}

/**
 * Theme utilities
 */
export const theme = {
  isDark: () =>
    typeof window !== "undefined" &&
    document.documentElement.getAttribute("data-theme") !== "light",
  setDark: (dark: boolean) => {
    if (typeof window !== "undefined") {
      if (dark) document.documentElement.removeAttribute("data-theme");
      else document.documentElement.setAttribute("data-theme", "light");
    }
  },
  toggle: () => {
    if (typeof window !== "undefined") {
      const isDark = theme.isDark();
      theme.setDark(!isDark);
    }
  },
};

/**
 * Accent colour utilities (for space tints)
 */
export const accent = {
  set: (colour: string) => {
    if (typeof window !== "undefined") {
      document.documentElement.style.setProperty("--color-accent", colour);
    }
  },
  get: () => getToken("--color-accent") || "#3b82f6",
};

/**
 * Space tint options
 */
export const spaceTints = [
  { id: "graphite", name: "Graphite", color: "var(--color-tint-graphite)" },
  { id: "sky", name: "Sky", color: "var(--color-tint-sky)" },
  { id: "sage", name: "Sage", color: "var(--color-tint-sage)" },
  { id: "rose", name: "Rose", color: "var(--color-tint-rose)" },
  { id: "amber", name: "Amber", color: "var(--color-tint-amber)" },
  { id: "teal", name: "Teal", color: "var(--color-tint-teal)" },
  { id: "lavender", name: "Lavender", color: "var(--color-tint-lavender)" },
  { id: "orchid", name: "Orchid", color: "var(--color-tint-orchid)" },
] as const;

export type SpaceTintId = (typeof spaceTints)[number]["id"];

/**
 * CSS class helpers for common patterns
 */
export const classes = {
  // Surface hierarchy
  canvas: "[data-surface='canvas']",
  page: "[data-surface='page']",
  chrome: "[data-surface='chrome']",
  surface: "[data-surface='surface']",
  raised: "[data-surface='raised']",
  float: "[data-surface='float']",
  menu: "[data-surface='menu']",

  // Interactive states
  hover: "hover:",
  active: "active:",
  focus: "focus-visible:",
  focusWithin: "focus-within:",

  // Theme
  dark: "[data-theme='dark']",
  light: "[data-theme='light']:not([data-theme='dark'])",

  // Material
  liquidGlass: "[data-material='liquid_glass']",
  acrylic: "[data-material='acrylic']",
  vibrancy: "[data-material='vibrancy']",
  none: "[data-material='none']",

  // Scroll
  scrolls: ".scrolls",
};

/**
 * All tokens combined for easy spreading
 */
export const tokens = {
  colors,
  radius,
  shadows,
  motion,
  typography,
  layout,
  menu,
} as const;

export type Tokens = typeof tokens;
