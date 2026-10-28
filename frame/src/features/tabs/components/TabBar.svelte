<script lang="ts">
  import * as tabDrag from "$session/tab-drag.svelte";
  import { SvelteSet } from "svelte/reactivity";
  import type { SidebarSectionView, TabView } from "$shared/ipc/bindings";
  import { tabs } from "$domain/tabs";
  import { preferences } from "$domain/preferences";
  import FolderRow from "./FolderRow.svelte";
  import {
    sidebarDisplayUnits,
    collapsedSidebarUnits,
    type SidebarEntry,
  } from "../lib/sidebar-model";
  import SplitGroupRow from "./SplitGroupRow.svelte";
  import TabRow from "./TabRow.svelte";
  import { untrack } from "svelte";
  import { ListMotion } from "$shared/lib/list-motion";
  import { RowDrag } from "$session/row-drag.svelte";
  import { favicons } from "$domain/favicons";
  import DragMark from "$shared/ui/DragMark";
  import { Add01Icon } from "@hugeicons/core-free-icons";
  import Icon from "$shared/ui/Icon";
  import * as m from "$shared/i18n/messages";

  let {
    entries,
    section,
    cascadeFrom = 0,
    label = "",
    onSelect,
  }: {
    /** Where this list's first row falls in the launch cascade. */
    cascadeFrom?: number;
    entries: SidebarEntry[];
    section: SidebarSectionView;
    /** Accessible label for the tab bar */
    label?: string;
    onSelect: (id: string) => void;
  } = $props();

  let displayUnits = $derived(sidebarDisplayUnits(section, entries, tabs.splitGroup()));
  const folded = new SvelteSet<string>();
  let hiddenUnits = $derived(collapsedSidebarUnits(displayUnits, folded, tabs.activeId()));

  const drag = new RowDrag({
    list: () => list,
    region: () => list?.closest<HTMLElement>("[data-tabs-drop]") ?? undefined,
    reorder: () => (section === "today" ? { essential: false } : null),
    drop: ({ id, x, y, over, before }) => {
      if (over?.closest("[data-essentials-drop]") && !tabDrag.keepable(id)) return;
      if (tabs.inSplit(id) && over?.closest("[data-essentials-drop], [data-tabs-drop]")) {
        const essential = !!over.closest("[data-essentials-drop]");
        void tabDrag.leaveSplit(id, essential || before ? { essential, before } : null);
        return;
      }
      if (over?.closest("[data-essentials-drop]")) {
        void tabDrag.move(id, true, before);
        return;
      }
      if (over?.closest("[data-tabs-drop]") && section !== "today") {
        void tabDrag.move(id, false, before);
        return;
      }
      tabs.dropTab(id, x, y);
    },
  });
  const handlePointerDown = (event: PointerEvent, tab: TabView) => drag.down(event, tab.id);
  const handlePointerMove = (event: PointerEvent) => drag.move(event);
  const handlePointerUp = (event: PointerEvent) => drag.up(event);
  const handlePointerCancel = (event: PointerEvent) => drag.cancel(event);

  let list = $state<HTMLUListElement>();
  const motion = new ListMotion({ enter: () => "rise" });
  let shape = $derived(
    `${displayUnits.map((unit) => unit.key).join(" ")}|${[...hiddenUnits].join(" ")}`,
  );
  $effect.pre(() => {
    void shape;
    untrack(() => {
      motion.capture(list);
      drag.landed();
    });
  });
  $effect(() => {
    void shape;
    untrack(() => motion.play(list));
  });

  function toggleFolder(key: string) {
    if (folded.has(key)) folded.delete(key);
    else folded.add(key);
  }

  function handleContextMenu(event: MouseEvent, tab: TabView) {
    event.preventDefault();
    drag.abandon();
    tabs.openTabMenu(tab.id, event.clientX, event.clientY);
  }

  function select(id: string) {
    if (drag.swallowClick()) return;
    onSelect(id);
  }

  function showsChild(position: number, depth: number): boolean {
    for (let next = position + 1; next < displayUnits.length; next++) {
      const child = displayUnits[next]!;
      if (child.depth <= depth) return false;
      if (!hiddenUnits.has(child.key)) return true;
    }
    return false;
  }

  // Show new tab button when there's room
  let showNewTab = $state(true);
  $effect(() => {
    const observer = new ResizeObserver(() => {
      if (!list) return;
      const listWidth = list.getBoundingClientRect().width;
      const childrenWidth = Array.from(list.children).reduce(
        (sum, child) => sum + child.getBoundingClientRect().width,
        0,
      );
      showNewTab = childrenWidth + 40 < listWidth - 20;
    });
    if (list) observer.observe(list);
    return () => observer.disconnect();
  });
</script>

<!-- Horizontal tab bar at the top of the content area -->
<nav class="tab-bar" aria-label={label} data-close-buttons={preferences.value("tabs.close-button")}>
  <ul bind:this={list} class="tab-bar-list" role="list" data-tabs-drop>
    {#each displayUnits as unit, index (unit.key)}
      {#if unit.kind === "folder"}
        <FolderRow
          motionKey={unit.key}
          expanded={!folded.has(unit.key) || showsChild(index, unit.depth)}
          ontoggle={() => toggleFolder(unit.key)}
          name={unit.node.kind.name}
          depth={unit.depth}
          class={hiddenUnits.has(unit.key) ? "hidden" : ""}
        />
      {:else if unit.kind === "split"}
        <SplitGroupRow
          motionKey={unit.key}
          tabs={unit.tabs.map((entry) => entry.tab)}
          activeId={tabs.activeId()}
          splitting={false}
          closable={section === "today"}
          class={hiddenUnits.has(unit.key) ? "hidden" : ""}
          onSelect={select}
          onClose={tabs.close}
          onContextMenu={handleContextMenu}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerCancel}
        />
      {:else}
        <TabRow
          cascade={cascadeFrom + index}
          tab={unit.tab}
          active={unit.tab.id === tabs.activeId()}
          depth={unit.depth}
          closable={section === "today" && !tabs.loneNewTab(unit.tab.id)}
          class={hiddenUnits.has(unit.key) ? "hidden" : ""}
          splitCandidate={false}
          onSelect={select}
          onClose={tabs.close}
          onContextMenu={handleContextMenu}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerCancel={handlePointerCancel}
        />
      {/if}
    {/each}
    {#if showNewTab}
      <li class="tab-bar-new-tab">
        <button
          type="button"
          class="tab-bar-new-tab-button"
          title={m.new_tab()}
          aria-label={m.new_tab()}
          onclick={tabs.open}
        >
          <Icon icon={Add01Icon} size={16} />
        </button>
      </li>
    {/if}
  </ul>
</nav>

{#if drag.ghost !== null}
  {@const held = tabs.tabs().find((tab) => tab.id === drag.ghost?.id)}
  <DragMark
    x={drag.ghost.x}
    y={drag.ghost.y}
    image={favicons.image(held?.icon)}
    tone={favicons.tone(held?.icon)}
    title={held?.title}
  />
{/if}

<style>
  .tab-bar {
    position: sticky;
    top: 0;
    z-index: 100;
    display: flex;
    flex: none;
    flex-direction: row;
    align-items: stretch;
    height: var(--titlebar-height);
    min-height: var(--titlebar-height);
    background: var(--color-chrome);
    border-bottom: 1px solid var(--color-border);
    padding-inline: var(--sidebar-inset);
  }

  .tab-bar-list {
    display: flex;
    flex: 1;
    flex-direction: row;
    align-items: stretch;
    gap: 2px;
    min-width: 0;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }

  .tab-bar-list::-webkit-scrollbar {
    display: none;
  }

  .tab-bar-new-tab {
    flex: none;
    display: flex;
    align-items: center;
  }

  .tab-bar-new-tab-button {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--color-muted);
    cursor: default;
    transition:
      background-color var(--motion-fast) var(--ease-out),
      color var(--motion-fast) var(--ease-out);
  }

  .tab-bar-new-tab-button:hover {
    background: var(--row-hover);
    color: var(--color-label-secondary);
  }

  .tab-bar-new-tab-button:active {
    scale: 0.94;
    transition-duration: var(--motion-instant);
  }

  :global(.tab-bar .browse-tab) {
    height: 100%;
    min-width: 120px;
    max-width: 240px;
    flex: 0 0 auto;
  }

  :global(.tab-bar .tab-open) {
    height: 100%;
    padding-inline: 12px;
  }

  :global(.tab-bar .tab-label) {
    font-size: var(--text-label);
  }

  :global(.tab-bar .tab-close) {
    width: 18px;
    height: 18px;
  }

  :global(.tab-bar .browse-tab-grouped) {
    border-radius: 0;
  }

  :global(.tab-bar .browse-tab[data-selected="true"]) {
    box-shadow: inset 0 -2px 0 var(--color-accent);
  }

  :global(.tab-bar .browse-tab:not([data-selected="true"]):hover) {
    background: var(--row-hover);
  }

  :global([data-close-buttons="always"]) :global(.tab-bar .tab-close) {
    opacity: 1;
  }
</style>
