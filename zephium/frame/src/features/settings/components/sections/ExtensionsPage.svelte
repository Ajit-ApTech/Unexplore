<script lang="ts">
  import { onMount } from "svelte";
  import {
    PuzzleIcon,
    Globe02Icon,
    Folder01Icon,
    PackageIcon,
    InformationCircleIcon,
    ArrowUpRight01Icon,
    Download01Icon,
  } from "@hugeicons/core-free-icons";
  import { webext } from "$domain/webext";
  import { commands } from "$shared/ipc/bindings";
  import Button from "$shared/ui/Button";
  import Icon from "$shared/ui/Icon";
  import EmptyState from "$shared/ui/EmptyState";
  import SettingsGroup from "$shared/ui/SettingsGroup";
  import SettingsRow from "$shared/ui/SettingsRow";
  import * as m from "$shared/i18n/messages";
  import { catalog } from "$features/webext/lib/catalog";
  import CatalogCard from "$features/webext/components/CatalogCard.svelte";
  import ExtensionRow from "$features/webext/components/ExtensionRow.svelte";

  let extensions = $derived(webext.list());
  let failure = $derived(webext.error());
  let installedIds = $derived(new Set(extensions.map((extension) => extension.id)));
  let storeUrlInput = $state("");

  onMount(() => {
    void webext.refresh();
  });

  const openUrl = (url: string) => {
    void commands.browserOpenUrl(url, true);
  };
</script>

<div class="extensions-settings">
  {#if failure !== null}
    <div class="failure-banner" role="alert">
      <p>{failure}</p>
    </div>
  {/if}

  <!-- Add Extensions Actions -->
  <SettingsGroup
    title={m.webext_install_file()}
    description="Add extensions directly from the Chrome Web Store, or install local packages and unpacked developer folders."
  >
    <SettingsRow
      title="Direct Install by Store Link or ID"
      description="Paste any Chrome Web Store URL or 32-character extension ID to download and install."
    >
      <form
        class="direct-install-form"
        onsubmit={(e) => {
          e.preventDefault();
          if (storeUrlInput.trim()) void webext.prepareStore(storeUrlInput.trim());
        }}
      >
        <input
          type="text"
          class="direct-install-input"
          bind:value={storeUrlInput}
          placeholder="Paste URL or ID..."
          disabled={webext.isPreparing() || webext.isInstalling()}
        />
        <Button
          variant="primary"
          type="submit"
          disabled={!storeUrlInput.trim() || webext.isPreparing() || webext.isInstalling()}
          pending={webext.isPreparing()}
        >
          <Icon icon={Download01Icon} size={15} />
          <span>Install</span>
        </Button>
      </form>
    </SettingsRow>

    <SettingsRow
      title={m.webext_open_store()}
      description="Search and install thousands of extensions from the official Chrome Web Store."
    >
      <Button variant="primary" onclick={() => openUrl("https://chromewebstore.google.com/")}>
        <Icon icon={Globe02Icon} size={16} />
        <span>{m.webext_open_store()}</span>
        <Icon icon={ArrowUpRight01Icon} size={14} />
      </Button>
    </SettingsRow>

    <SettingsRow
      title={m.webext_install_package()}
      description="Install an extension from a .crx package or a .zip archive on your computer."
    >
      <Button variant="secondary" onclick={() => void webext.chooseFile(false)}>
        <Icon icon={PackageIcon} size={16} />
        <span>Select File…</span>
      </Button>
    </SettingsRow>

    <SettingsRow
      title={m.webext_install_folder()}
      description="Load an unpacked extension directory (Developer mode or extracted extension folder)."
    >
      <Button variant="secondary" onclick={() => void webext.chooseFile(true)}>
        <Icon icon={Folder01Icon} size={16} />
        <span>Select Folder…</span>
      </Button>
    </SettingsRow>
  </SettingsGroup>

  <!-- Chrome Extension Migration Notice -->
  <section class="chrome-guide-card">
    <div class="guide-header">
      <span class="guide-icon"><Icon icon={InformationCircleIcon} size={20} /></span>
      <h3>Why didn't my Google Chrome extension load automatically?</h3>
    </div>
    <p class="guide-explanation">
      Google Chrome stores installed extensions inside its own isolated application directory (<code
        >~/Library/Application Support/Google/Chrome/Default/Extensions/</code
      >). Because Zephium runs as an independent browser, it cannot access or sync Chrome's private
      directory automatically.
    </p>
    <div class="guide-steps">
      <div class="guide-step">
        <span class="step-num">1</span>
        <div class="step-content">
          <strong>Direct Store Install (Recommended)</strong>
          <span
            >Click <em>Chrome Web Store</em> above, locate your extension, and click
            <em>Add to Zephium</em> in the address bar.</span
          >
        </div>
      </div>
      <div class="guide-step">
        <span class="step-num">2</span>
        <div class="step-content">
          <strong>Load Unpacked Folder</strong>
          <span
            >Click <em>Select Folder…</em> above and choose the extension's folder from your Chrome installation
            directory or developer files.</span
          >
        </div>
      </div>
      <div class="guide-step">
        <span class="step-num">3</span>
        <div class="step-content">
          <strong>Drag & Drop</strong>
          <span
            >You can also drag and drop any <code>.crx</code> or <code>.zip</code> file directly onto
            any Zephium window to install it immediately.</span
          >
        </div>
      </div>
    </div>
  </section>

  <!-- Installed Extensions Section -->
  <SettingsGroup
    title={`${m.webext_installed()} (${extensions.length})`}
    description="Manage permissions, site access, options, or remove installed extensions."
  >
    {#if extensions.length === 0}
      <div class="empty-extensions">
        <EmptyState title={m.webext_empty_title()} description={m.webext_empty_help()}>
          {#snippet icon()}<Icon icon={PuzzleIcon} size={28} />{/snippet}
          {#snippet action()}
            <Button variant="primary" onclick={() => openUrl("https://chromewebstore.google.com/")}>
              <Icon icon={Globe02Icon} size={16} />
              <span>{m.webext_open_store()}</span>
            </Button>
          {/snippet}
        </EmptyState>
      </div>
    {:else}
      <ul class="installed-list">
        {#each extensions as extension (extension.id)}
          <ExtensionRow {extension} />
        {/each}
      </ul>
    {/if}
  </SettingsGroup>

  <!-- Recommended Extensions Section -->
  <SettingsGroup
    title={m.webext_recommended()}
    description="Popular extensions that have been verified to work smoothly in Zephium."
  >
    <div class="recommended-catalog">
      {#each catalog as group (group.title)}
        <h4 class="catalog-group-title">{group.title()}</h4>
        <ul class="catalog-grid">
          {#each group.entries as entry (entry.id)}
            <CatalogCard
              {entry}
              installed={installedIds.has(entry.id)}
              onget={() => void webext.prepareStore(entry.id)}
            />
          {/each}
        </ul>
      {/each}
    </div>
  </SettingsGroup>
</div>

<style>
  .extensions-settings {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .failure-banner {
    padding: 12px 16px;
    margin-bottom: 16px;
    border-radius: var(--radius-card);
    background: var(--color-danger-subtle, rgba(239, 68, 68, 0.1));
    border: 1px solid var(--color-danger);
    color: var(--color-danger);
    font-size: var(--text-label);
  }

  .failure-banner p {
    margin: 0;
  }

  .chrome-guide-card {
    margin-block: 0 34px;
    padding: 18px 20px;
    border-radius: var(--radius-card);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-card);
  }

  .guide-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
  }

  .guide-icon {
    display: grid;
    place-items: center;
    color: var(--color-accent);
  }

  .guide-header h3 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text);
  }

  .guide-explanation {
    margin: 0 0 16px;
    font-size: var(--text-label);
    line-height: 1.5;
    color: var(--color-muted);
  }

  .guide-explanation code {
    font-family: var(--font-mono, monospace);
    font-size: 0.9em;
    padding: 2px 5px;
    border-radius: 4px;
    background: var(--color-fill);
  }

  .guide-steps {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .guide-step {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-control-compact);
    background: var(--color-fill);
  }

  .step-num {
    display: grid;
    place-items: center;
    flex: none;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--color-accent);
    color: var(--color-accent-text, #ffffff);
    font-size: 11px;
    font-weight: 600;
  }

  .step-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: var(--text-label);
  }

  .step-content strong {
    color: var(--color-text);
    font-weight: 550;
  }

  .step-content span {
    color: var(--color-muted);
    line-height: 1.4;
  }

  .step-content em {
    font-style: normal;
    color: var(--color-text);
    font-weight: 500;
  }

  .step-content code {
    font-family: var(--font-mono, monospace);
    font-size: 0.9em;
  }

  .empty-extensions {
    padding: 16px;
  }

  .installed-list {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .recommended-catalog {
    padding: 16px;
  }

  .catalog-group-title {
    margin: 12px 0 8px;
    font-size: var(--text-label);
    font-weight: 550;
    color: var(--color-muted);
  }

  .catalog-group-title:first-child {
    margin-top: 0;
  }

  .catalog-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 8px;
    margin: 0 0 16px;
    padding: 0;
    list-style: none;
  }

  .direct-install-form {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 280px;
    max-width: 420px;
  }

  .direct-install-input {
    flex: 1;
    height: 32px;
    padding-inline: 10px;
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
