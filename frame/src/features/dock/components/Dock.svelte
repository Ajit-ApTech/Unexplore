<script lang="ts">
  import type { Snippet } from "svelte";
  import ToolShelf from "./ToolShelf.svelte";
  import * as m from "$shared/i18n/messages";
  import { BubbleChatIcon } from "@hugeicons/core-free-icons";
  import Icon from "$shared/ui/Icon";
  import * as tools from "$session/tools.svelte";
  import { preferences } from "$domain/preferences";
  import { tabs } from "$domain/tabs";
  import { profiles } from "$domain/profiles";
  import { keymap } from "$domain/keymap";
  import { acceleratorKeys } from "$shared/lib/accelerator";
  import { IS_MAC } from "$shared/platform";

  let {
    sites,
    above,
    compact = false,
    tools: hasTools = true,
    foot,
  }: {
    sites?: Snippet;
    /** Kept sites that did not fit beside the shelf, in rows over it. */
    above?: Snippet;
    compact?: boolean;
    /** The profile and its tools; Work keeps its own on the canvas. */
    tools?: boolean;
    /** What closes the rail at its very foot, below the kept sites. */
    foot?: Snippet;
  } = $props();

  const privately = $derived(profiles.inPrivate() || tabs.profile()?.kind === "incognito");
  const showAiButton = $derived(preferences.aiEnabled() && !privately && hasTools);

  function keysFor(command: string) {
    const accelerator = keymap.all().find((entry) => entry.id === command)?.accelerator;
    return accelerator ? acceleratorKeys(accelerator, IS_MAC).join(IS_MAC ? "" : "+") : null;
  }

  function toggleAi() {
    if (tools.activeTool() === "ai") tools.close();
    else tools.open("ai");
  }
</script>

<!--
  The base of the column. Its bottom row is the mini-apps and as many kept
  sites as fit beside them; the rest of the sites stack in full-width rows
  above, growing upwards, so the shelf never moves and the column above only
  gives up a row when a whole row has been filled.
-->
<!-- The base of the column settles last, once the list above it has. -->
<footer
  class="dock"
  data-compact={compact}
  data-foot={foot ? "" : undefined}
  data-cascade
  style:--cascade={8}
>
  {#if showAiButton}
    <div class="dock-ai-slot">
      <button
        type="button"
        class="dock-ai-btn"
        class:dock-ai-active={tools.activeTool() === "ai"}
        onclick={toggleAi}
        title={compact ? `${m.panel_ask_zephium()} (${keysFor("tool.ai") ?? "⌘E"})` : undefined}
        aria-label={m.panel_ask_zephium()}
      >
        <span class="dock-ai-icon"><Icon icon={BubbleChatIcon} size={16} /></span>
        {#if !compact}
          <span class="dock-ai-label">{m.panel_ask_zephium()}</span>
          {#if keysFor("tool.ai")}<span class="dock-ai-keys">{keysFor("tool.ai")}</span>{/if}
        {/if}
      </button>
    </div>
  {/if}

  {#if above}<div class="dock-above" data-under-panel>{@render above()}</div>{/if}
  <!-- In the row the profile leads; down the rail it closes the column. -->
  <div class="dock-base">
    {#if hasTools && !compact}<ToolShelf />
      <span class="dock-rule" aria-hidden="true"></span>{/if}
    {#if sites}<div class="dock-sites-slot" data-under-panel={compact || undefined}>
        {@render sites()}
      </div>{/if}
    {#if hasTools && compact}<span class="dock-rule" aria-hidden="true"></span><ToolShelf
        compact
      />{/if}
    {#if foot}<span class="dock-rule" aria-hidden="true"></span>{@render foot()}{/if}
  </div>
</footer>

<style>
  .dock {
    flex: none;
    display: flex;
    flex-direction: column;
    gap: var(--dock-gap);
    margin-block-start: 8px;
    padding: 0 var(--sidebar-inset) var(--sidebar-inset);
  }

  .dock-above {
    display: flex;
    flex-direction: column;
    gap: var(--dock-gap);
  }

  .dock-above:not(:has(> :not([hidden]))) {
    display: none;
  }

  .dock-base {
    display: flex;
    align-items: center;
    gap: var(--dock-shelf-gap);
  }

  /* Ours on one side of the rule, the web's on the other. */
  .dock-rule {
    flex: none;
    align-self: center;
    width: 1px;
    height: calc(var(--dock-tile) - 14px);
    border-radius: var(--radius-capsule);
    background: var(--color-border);
  }

  .dock-sites-slot {
    flex: 1;
    min-width: 0;
  }

  /* The rail stacks the same three parts it does at full width. */
  .dock[data-compact="true"] {
    align-items: center;
    gap: 8px;
    padding-block-end: 8px;
  }

  /* Work's window has no bottom gutter of its own, so its foot brings Browse's: the profile stands where it does there. */
  .dock[data-compact="true"][data-foot] {
    padding-block-end: 16px;
  }

  .dock[data-compact="true"] .dock-base {
    flex-direction: column;
    gap: 8px;
  }

  .dock[data-compact="true"] .dock-rule {
    width: 22px;
    height: 1px;
  }

  .dock[data-compact="true"] .dock-sites-slot {
    flex: none;
  }

  .dock-ai-slot {
    display: flex;
    width: 100%;
    margin-block-end: 2px;
  }

  .dock-ai-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 36px;
    padding: 0 10px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md, 8px);
    background: var(--row-active);
    color: var(--color-text);
    font: inherit;
    font-size: var(--text-body);
    font-weight: 500;
    cursor: default;
    box-shadow: var(--shine-top);
    transition:
      background-color var(--motion-fast) var(--ease-out),
      border-color var(--motion-fast) var(--ease-out),
      color var(--motion-fast) var(--ease-out),
      scale var(--motion-slow) var(--ease-spring);
  }

  .dock-ai-btn:hover {
    background: var(--row-pressed);
    border-color: var(--color-accent);
    color: var(--color-text);
  }

  .dock-ai-btn:active {
    scale: 0.98;
  }

  .dock-ai-btn.dock-ai-active {
    background: var(--row-pressed);
    border-color: var(--color-accent);
    color: var(--color-accent);
    box-shadow: 0 0 10px var(--row-active);
  }

  .dock-ai-icon {
    display: flex;
    align-items: center;
    color: var(--color-accent);
  }

  .dock-ai-label {
    flex: 1;
    text-align: start;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dock-ai-keys {
    font-size: var(--text-label);
    color: var(--color-faint);
    padding: 1px 5px;
    border-radius: 4px;
    background: var(--color-base);
    font-variant-numeric: tabular-nums;
  }

  /* Rail (compact) width */
  .dock[data-compact="true"] .dock-ai-slot {
    justify-content: center;
  }

  .dock[data-compact="true"] .dock-ai-btn {
    width: 36px;
    height: 36px;
    padding: 0;
    justify-content: center;
    border-radius: var(--radius-capsule);
  }
</style>
