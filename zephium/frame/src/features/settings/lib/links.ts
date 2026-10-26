import * as m from "$shared/i18n/messages";
import { IS_MAC } from "$shared/platform";
import { HOMEPAGE_URL } from "$shared/brand";

export const WEBSITE = HOMEPAGE_URL;

export const links = [
  { id: "website", url: WEBSITE, label: () => m.about_link_website() },
  // Without a menu bar there is no Help menu, so the documentation it leads
  // to is named Help here, where Windows and Linux look for it.
  {
    id: "docs",
    url: `${WEBSITE}/docs`,
    label: () => (IS_MAC ? m.about_link_docs() : m.native_help()),
  },
  { id: "github", url: "https://github.com/[your-org]/[your-browser-repo]", label: () => "GitHub" },
  { id: "discord", url: "https://discord.gg/[your-discord-invite]", label: () => "Discord" },
  { id: "youtube", url: "https://www.youtube.com/@[your-channel]", label: () => "YouTube" },
] as const;
