<script lang="ts">
  import {
    ArrowLeft02Icon,
    PuzzleIcon,
    Globe02Icon,
    PackageIcon,
    Folder01Icon,
    InformationCircleIcon,
    Download01Icon,
  } from "@hugeicons/core-free-icons";
  import { onMount } from "svelte";
  import { surface as browser } from "$domain/surface";
  import { webext } from "$domain/webext";
  import { commands } from "$shared/ipc/bindings";
  import { IS_WINDOWS } from "$shared/platform";
  import Button from "$shared/ui/Button";
  import EmptyState from "$shared/ui/EmptyState";
  import Icon from "$shared/ui/Icon";
  import IconButton from "$shared/ui/IconButton";
  import Menu from "$shared/ui/Menu";
  import * as m from "$shared/i18n/messages";
  import { catalog } from "../lib/catalog";
  import CatalogCard from "./CatalogCard.svelte";
  import ExtensionRow from "./ExtensionRow.svelte";

  let extensions = $derived(webext.list());
  let failure = $derived(webext.error());
  let installedIds = $derived(new Set(extensions.map((extension) => extension.id)));
  let storeInput = $state("");

  onMount(() => void webext.refresh());

  const openInTab = (url: string) => void commands.browserOpenUrl(url, true);
</script>

<section class="library-shell">
  <header class="internal-toolbar">
    <IconButton
      icon={ArrowLeft02Icon}
      label={m.settings_back()}
      onclick={() => void browser.close()}
    /><span class="toolbar-current">{m.webext_page_title()}</span>
    <span class="spacer"></span>
    <Menu
      label={m.webext_install_file()}
      triggerClass="toolbar-action"
      align="end"
      entries={[
        { kind: "item", id: "package", label: m.webext_install_package() },
        { kind: "item", id: "folder", label: m.webext_install_folder() },
      ]}
      onselect={(id) => void webext.chooseFile(id === "folder")}
    >
      {#snippet trigger()}{m.webext_install_file()}{/snippet}
    </Menu>
    <button
      type="button"
      class="toolbar-action"
      onclick={() => openInTab("https://chromewebstore.google.com/")}
      >{m.webext_open_store()}</button
    >
  </header>

  <div class="content">
    {#if failure !== null}
      <p class="failure" role="alert">{failure}</p>
    {/if}

    <!-- Direct Web Store Link / ID Install -->
    <div class="direct-install-box">
      <div class="direct-install-header">
        <Icon icon={Download01Icon} size={16} />
        <span>Install Extension by Web Store URL or ID</span>
      </div>
      <form
        class="direct-install-form"
        onsubmit={(e) => {
          e.preventDefault();
          if (storeInput.trim()) void webext.prepareStore(storeInput.trim());
        }}
      >
        <input
          type="text"
          class="direct-install-input"
          bind:value={storeInput}
          placeholder="Paste extension link (e.g. https://chromewebstore.google.com/detail/...) or 32-char ID"
          disabled={webext.isPreparing() || webext.isInstalling()}
        />
        <Button
          variant="primary"
          type="submit"
          disabled={!storeInput.trim() || webext.isPreparing() || webext.isInstalling()}
          pending={webext.isPreparing()}
        >
          <Icon icon={Download01Icon} size={15} />
          <span>Install</span>
        </Button>
      </form>
    </div>

    {#if extensions.length === 0}
      <div class="empty">
        <EmptyState title={m.webext_empty_title()} description={m.webext_empty_help()}>
          {#snippet icon()}<Icon icon={PuzzleIcon} size={28} />{/snippet}
          {#snippet action()}
            <div class="empty-actions">
              <Button
                variant="primary"
                onclick={() => openInTab("https://chromewebstore.google.com/")}
              >
                <Icon icon={Globe02Icon} size={16} />
                <span>{m.webext_open_store()}</span>
              </Button>
              <Button variant="secondary" onclick={() => void webext.chooseFile(false)}>
                <Icon icon={PackageIcon} size={16} />
                <span>{m.webext_install_package()}</span>
              </Button>
              <Button variant="secondary" onclick={() => void webext.chooseFile(true)}>
                <Icon icon={Folder01Icon} size={16} />
                <span>{m.webext_install_folder()}</span>
              </Button>
            </div>
          {/snippet}
        </EmptyState>
      </div>

      <aside class="chrome-help-card">
        <div class="chrome-help-header">
          <Icon icon={InformationCircleIcon} size={16} />
          <span>Using extensions from Google Chrome</span>
        </div>
        <p>
          Chrome and Zephium operate independently, so extensions installed in Google Chrome do not
          sync automatically. You can install them by opening the <strong>Chrome Web Store</strong>
          in Zephium and clicking <em>Add to Zephium</em>, or click
          <strong>Unpacked folder…</strong>
          to select the extension directory from
          <code>~/Library/Application Support/Google/Chrome/Default/Extensions/</code>.
        </p>
      </aside>
    {:else}
      <h2 class="heading">
        {m.webext_installed()} <span class="count">{extensions.length}</span>
      </h2>
      <ul class="group">
        {#each extensions as extension (extension.id)}
          <ExtensionRow {extension} />
        {/each}
      </ul>
    {/if}

    <h2 class="heading">{IS_WINDOWS ? m.webext_page_title() : m.webext_recommended()}</h2>
    {#each catalog as group (group.title)}
      <h3 class="group-title">{group.title()}</h3>
      <ul class="catalog">
        {#each group.entries as entry (entry.id)}
          <CatalogCard
            {entry}
            installed={installedIds.has(entry.id)}
            onget={() => void webext.prepareStore(entry.id)}
          />
        {/each}
      </ul>
    {/each}

    <p class="note">{m.webext_blocker_note()}</p>
  </div>
</section>

<style>
  .spacer {
    flex: 1;
  }

  .internal-toolbar :global(.toolbar-action) {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding-inline: 10px;
    border: 0;
    border-radius: var(--radius-control-compact);
    background: none;
    font: inherit;
    color: var(--color-muted);
    cursor: pointer;
    transition:
      background-color var(--motion-instant) var(--ease-smooth),
      color var(--motion-instant) var(--ease-smooth);
  }

  .internal-toolbar :global(.toolbar-action:hover),
  .internal-toolbar :global(.toolbar-action[data-state="open"]) {
    background: var(--row-active);
    color: var(--color-text);
  }

  /* The same reading column as History, so the destinations line up. */
  .content {
    --library-gutter: clamp(16px, 4vw, 48px);
    --library-column: 720px;

    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-inline: max(var(--library-gutter), calc((100% - var(--library-column)) / 2));
    padding-block: 8px 32px;
  }

  .failure {
    margin: 0 0 12px;
    font-size: var(--text-label);
    color: var(--color-danger);
  }

  .heading {
    display: flex;
    align-items: flex-end;
    gap: 8px;
    margin: 18px 0 8px;
    padding-inline: 10px;
    font-size: var(--text-label);
    font-weight: 550;
    letter-spacing: 0.01em;
    color: var(--color-muted);
  }

  .heading:first-child {
    margin-block-start: 4px;
  }

  .count {
    font-weight: 400;
    color: var(--color-faint);
  }

  .group {
    margin: 0;
    padding: 0;
    list-style: none;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-panel);
    background: var(--color-raised);
  }

  .catalog {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .group-title {
    margin: 14px 0 6px;
    padding-inline: 10px;
    font-size: var(--text-label);
    font-weight: 450;
    color: var(--color-faint);
  }

  .heading + .group-title {
    margin-block-start: 0;
  }

  .note {
    margin: 18px 0 0;
    padding-inline: 10px;
    font-size: var(--text-label);
    line-height: 1.45;
    color: var(--color-faint);
  }

  .empty {
    padding-block: 12px;
  }

  .empty-actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin-top: 8px;
  }

  .chrome-help-card {
    margin: 8px 0 24px;
    padding: 14px 16px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--color-border);
    background: var(--color-raised);
    font-size: var(--text-label);
  }

  .chrome-help-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
    font-weight: 550;
    color: var(--color-text);
  }

  .chrome-help-card p {
    margin: 0;
    line-height: 1.5;
    color: var(--color-muted);
  }

  .chrome-help-card code {
    font-family: var(--font-mono, monospace);
    font-size: 0.9em;
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--color-fill);
  }

  .direct-install-box {
    margin: 8px 0 16px;
    padding: 14px 16px;
    border-radius: var(--radius-panel);
    border: 1px solid var(--color-border);
    background: var(--color-surface, var(--color-raised));
    box-shadow: var(--shadow-card);
  }

  .direct-install-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    font-size: var(--text-label);
    font-weight: 600;
    color: var(--color-text);
  }

  .direct-install-form {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .direct-install-input {
    flex: 1;
    height: 34px;
    padding-inline: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-control-compact, 6px);
    background: var(--color-fill);
    color: var(--color-text);
    font: inherit;
    font-size: var(--text-label);
    outline: none;
    transition:
      border-color 0.15s ease,
      box-shadow 0.15s ease;
  }

  .direct-install-input:focus {
    border-color: var(--color-accent, #3b82f6);
    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.25);
  }

  .direct-install-input::placeholder {
    color: var(--color-faint);
  }
</style>
