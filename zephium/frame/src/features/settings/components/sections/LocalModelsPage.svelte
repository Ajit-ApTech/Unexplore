<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { tabs } from "$domain/tabs";
  import { commands } from "$shared/ipc/bindings";
  import type {
    CuratedModel,
    LocalModelStatus,
    ModelDownloadProgress,
    ModelFileInfo,
  } from "$shared/ipc/bindings";
  import Button from "$shared/ui/Button";
  import Badge from "$shared/ui/Badge";
  import SettingsGroup from "$shared/ui/SettingsGroup";
  import Icon from "$shared/ui/Icon";
  import {
    Folder01Icon,
    Download01Icon,
    PlayIcon,
    PauseIcon,
    StopIcon,
    Delete02Icon,
    CheckmarkCircle02Icon,
    AlertCircleIcon,
    FlashIcon,
  } from "@hugeicons/core-free-icons";

  let status = $state<LocalModelStatus>({
    is_running: false,
    model_name: null,
    model_path: null,
    port: 11436,
    endpoint: "http://127.0.0.1:11436/v1",
    metal_accelerated: true,
    error: null,
  });

  let curated = $state<CuratedModel[]>([]);
  let stored = $state<ModelFileInfo[]>([]);
  let downloads = $state<Record<string, ModelDownloadProgress>>({});
  let loadingModel = $state(false);
  let customUrl = $state("");
  let customFilename = $state("");
  let errorMessage = $state<string | null>(null);

  let customDownloads = $derived(
    Object.values(downloads).filter(
      (d) =>
        (d.status === "downloading" || d.status === "paused") &&
        !curated.some((c) => c.id === d.id),
    ),
  );

  function formatBytes(bytes: number): string {
    if (!bytes || bytes <= 0) return "0 B";
    const gb = 1024 * 1024 * 1024;
    const mb = 1024 * 1024;
    const kb = 1024;
    if (bytes >= gb) {
      return (bytes / gb).toFixed(2) + " GB";
    } else if (bytes >= mb) {
      return (bytes / mb).toFixed(1) + " MB";
    } else if (bytes >= kb) {
      return (bytes / kb).toFixed(0) + " KB";
    }
    return Math.round(bytes) + " B";
  }

  function handleProgress(progress: ModelDownloadProgress) {
    if (!progress || !progress.id) return;
    if (progress.status === "completed" || progress.status === "cancelled") {
      const next = { ...downloads };
      delete next[progress.id];
      downloads = next;
      void refresh();
    } else {
      downloads = {
        ...downloads,
        [progress.id]: { ...progress },
      };
      if (progress.status === "error") {
        errorMessage = progress.error ?? "Download failed.";
        void refresh();
      }
    }
  }

  async function refresh() {
    try {
      const s = await commands.localEngineGetStatus();
      status = s;
    } catch (_) {}

    try {
      const c = await commands.localEngineCuratedModels();
      if ("status" in c && c.status === "ok") {
        curated = c.data;
      }
    } catch (_) {}

    try {
      const list = await commands.localEngineListModels();
      if ("status" in list && list.status === "ok") {
        stored = list.data;
      }
    } catch (_) {}

    try {
      const activeDownloads = await commands.localEngineGetDownloads();
      if ("status" in activeDownloads && activeDownloads.status === "ok") {
        const next: Record<string, ModelDownloadProgress> = { ...downloads };
        for (const item of activeDownloads.data) {
          next[item.id] = item;
        }
        downloads = next;
      }
    } catch (_) {}
  }

  onMount(() => {
    void refresh();
    const interval = setInterval(() => void refresh(), 2500);

    const onCustomEvent = (e: Event) => {
      const custom = e as CustomEvent<ModelDownloadProgress>;
      if (custom && custom.detail) {
        handleProgress(custom.detail);
      }
    };
    window.addEventListener("unexplore:model-download-progress", onCustomEvent);

    let unlistenProgress: (() => void) | undefined;
    void listen<ModelDownloadProgress>("unexplore:model-download-progress", (event) => {
      if (event && event.payload) {
        handleProgress(event.payload);
      }
    }).then((un) => {
      unlistenProgress = un;
    });

    return () => {
      clearInterval(interval);
      window.removeEventListener("unexplore:model-download-progress", onCustomEvent);
      unlistenProgress?.();
    };
  });

  async function selectModelForActiveProfile(modelPath: string | null) {
    try {
      const profileId = tabs.profile()?.id;
      if (!profileId) return;
      if (modelPath) {
        const filename = modelPath.split("/").pop() ?? "";
        const alias = filename.replace(/\.gguf$/i, "");
        const modelId = `compatible/${alias}`;
        await commands.workChooseModel(profileId, "lead", modelId);
        await commands.workChooseModel(profileId, "page", modelId);
      } else {
        await commands.workChooseModel(profileId, "lead", null);
        await commands.workChooseModel(profileId, "page", null);
      }
    } catch (e) {
      console.warn("Failed to set model choice:", e);
    }
  }

  async function pickAndLoadFromDisk() {
    errorMessage = null;
    try {
      const res = await commands.localEnginePickFile();
      if ("status" in res && res.status === "ok" && res.data) {
        const file = res.data;
        loadingModel = true;
        const loadRes = await commands.localEngineLoadModel(file.path);
        loadingModel = false;
        if ("status" in loadRes && loadRes.status === "ok") {
          status = loadRes.data;
          await selectModelForActiveProfile(file.path);
          await refresh();
        } else if ("status" in loadRes && loadRes.status === "error") {
          errorMessage = loadRes.error;
        }
      }
    } catch (e: any) {
      loadingModel = false;
      errorMessage = e?.toString() ?? "Failed to pick or load model.";
    }
  }

  async function loadModel(path: string) {
    errorMessage = null;
    loadingModel = true;
    try {
      const res = await commands.localEngineLoadModel(path);
      loadingModel = false;
      if ("status" in res && res.status === "ok") {
        status = res.data;
        await selectModelForActiveProfile(path);
        await refresh();
      } else if ("status" in res && loadError(res)) {
        errorMessage = (res as any).error;
      }
    } catch (e: any) {
      loadingModel = false;
      errorMessage = e?.toString() ?? "Failed to load model.";
    }
  }

  function loadError(res: any): boolean {
    return res && res.status === "error";
  }

  async function unloadModel() {
    errorMessage = null;
    try {
      const res = await commands.localEngineUnloadModel();
      if ("status" in res && res.status === "ok") {
        status = res.data;
        await selectModelForActiveProfile(null);
        await refresh();
      }
    } catch (e: any) {
      errorMessage = e?.toString() ?? "Failed to unload model.";
    }
  }

  async function startDownload(model: CuratedModel) {
    errorMessage = null;
    downloads = {
      ...downloads,
      [model.id]: {
        id: model.id,
        downloaded_bytes: 0,
        total_bytes: 0,
        percent: 0,
        speed_mbps: 0,
        status: "downloading",
        error: null,
      },
    };
    try {
      await commands.localEngineDownloadModel(model.id, model.url, model.filename);
    } catch (e: any) {
      errorMessage = e?.toString() ?? `Failed to download ${model.name}`;
      const next = { ...downloads };
      delete next[model.id];
      downloads = next;
    }
  }

  async function pauseDownload(id: string) {
    errorMessage = null;
    try {
      await commands.localEnginePauseDownload(id);
      if (downloads[id]) {
        downloads = {
          ...downloads,
          [id]: { ...downloads[id], status: "paused" },
        };
      }
    } catch (e: any) {
      errorMessage = e?.toString() ?? "Failed to pause download.";
    }
  }

  async function resumeDownload(id: string) {
    errorMessage = null;
    try {
      await commands.localEngineResumeDownload(id);
      if (downloads[id]) {
        downloads = {
          ...downloads,
          [id]: { ...downloads[id], status: "downloading" },
        };
      }
    } catch (e: any) {
      errorMessage = e?.toString() ?? "Failed to resume download.";
    }
  }

  async function cancelDownload(id: string) {
    try {
      await commands.localEngineCancelDownload(id);
      const next = { ...downloads };
      delete next[id];
      downloads = next;
      await refresh();
    } catch (_) {}
  }

  async function deleteModel(filename: string) {
    errorMessage = null;
    try {
      await commands.localEngineDeleteModel(filename);
      await refresh();
    } catch (e: any) {
      errorMessage = e?.toString() ?? "Failed to delete model file.";
    }
  }

  async function startCustomDownload() {
    if (!customUrl.trim()) return;
    errorMessage = null;
    const url = customUrl.trim();
    let filename = customFilename.trim();
    if (!filename) {
      try {
        const parsed = new URL(url);
        const segments = parsed.pathname.split("/").filter(Boolean);
        filename = segments[segments.length - 1] || "custom-model.gguf";
      } catch (_) {
        filename = "custom-model.gguf";
      }
    }
    if (!filename.endsWith(".gguf")) filename += ".gguf";
    const id = `custom-${Date.now()}`;
    downloads = {
      ...downloads,
      [id]: {
        id,
        downloaded_bytes: 0,
        total_bytes: 0,
        percent: 0,
        speed_mbps: 0,
        status: "downloading",
        error: null,
      },
    };
    customUrl = "";
    customFilename = "";
    try {
      await commands.localEngineDownloadModel(id, url, filename);
    } catch (e: any) {
      errorMessage = e?.toString() ?? "Failed to start custom download.";
      const next = { ...downloads };
      delete next[id];
      downloads = next;
    }
  }

  function isStored(filename: string) {
    return stored.some((s) => s.filename === filename);
  }

  function getStoredPath(filename: string) {
    return stored.find((s) => s.filename === filename)?.path ?? null;
  }
</script>

<div class="local-models-hub">
  <!-- Status Banner Card -->
  <div class="engine-hero" class:running={status.is_running}>
    <div class="hero-left">
      <div class="status-indicator-badge">
        <span class="status-dot" class:active={status.is_running}></span>
        <span class="status-label">
          {status.is_running ? "Native GGUF Engine Active" : "Native Engine Idle"}
        </span>
      </div>
      {#if status.is_running}
        <h3 class="active-model-title">{status.model_name ?? "Local Model"}</h3>
        <p class="active-model-sub">
          Endpoint: <code>{status.endpoint}</code> · Port {status.port}
        </p>
        <div class="hero-tags">
          <span class="metal-pill">
            <Icon icon={FlashIcon} size={13} />
            Apple Silicon Metal GPU (Offload: 99 Layers)
          </span>
          <span class="uncensored-pill">Zero Cloud · 100% Local & Private</span>
        </div>
      {:else}
        <h3 class="active-model-title">No Local Model Loaded</h3>
        <p class="active-model-sub">
          Load an uncensored GGUF model from disk or download one below to chat locally with Metal
          GPU acceleration.
        </p>
      {/if}
    </div>
    <div class="hero-actions">
      {#if status.is_running}
        <Button variant="danger" size="compact" onclick={unloadModel}>
          <Icon icon={StopIcon} size={14} />
          Unload Model
        </Button>
      {/if}
      <Button variant="primary" pending={loadingModel} onclick={pickAndLoadFromDisk}>
        <Icon icon={Folder01Icon} size={15} />
        Pick GGUF from Disk
      </Button>
    </div>
  </div>

  {#if errorMessage}
    <div class="error-banner" role="alert">
      <Icon icon={AlertCircleIcon} size={16} />
      <span>{errorMessage}</span>
    </div>
  {/if}

  <!-- Curated Uncensored Models Section -->
  <SettingsGroup
    title="Curated Uncensored Models"
    description="Pre-tested high performance uncensored models with no moralizing, guardrails, or refusal."
  >
    <div class="curated-grid">
      {#each curated as model (model.id)}
        {@const onDisk = isStored(model.filename)}
        {@const storedPath = getStoredPath(model.filename)}
        {@const isActive = status.is_running && status.model_path === storedPath}
        {@const dl = downloads[model.id]}
        <div class="curated-card" class:active-card={isActive}>
          <div class="card-header">
            <div class="card-title-row">
              <h4>{model.name}</h4>
              <div class="card-badges">
                {#if model.badge}
                  <Badge tone={model.badge === "HERETIC" ? "danger" : "accent"}>
                    {model.badge}
                  </Badge>
                {/if}
                <Badge tone="neutral">{model.label}</Badge>
              </div>
            </div>
            <p class="card-desc">{model.description}</p>
            <div class="card-meta">
              <span>Size: <strong>{model.size_gb} GB</strong></span>
              <span>Min RAM: <strong>{model.min_ram_gb} GB</strong></span>
            </div>
          </div>

          {#if dl && (dl.status === "downloading" || dl.status === "paused")}
            <div class="card-downloading" class:paused={dl.status === "paused"}>
              <div class="dl-progress-bar">
                <div
                  class="dl-fill"
                  style="width: {dl.percent ?? 0}%"
                  class:paused-fill={dl.status === "paused"}
                ></div>
              </div>
              <div class="dl-stats">
                {#if dl.status === "paused"}
                  <span class="paused-text">Paused ({(dl.percent ?? 0).toFixed(1)}%)</span>
                {:else}
                  <span
                    >{(dl.percent ?? 0).toFixed(1)}% · {(dl.speed_mbps ?? 0).toFixed(1)} MB/s</span
                  >
                {/if}
                <div class="dl-control-buttons">
                  {#if dl.status === "paused"}
                    <button
                      type="button"
                      class="control-btn resume-btn"
                      title="Resume download"
                      onclick={() => void resumeDownload(model.id)}
                    >
                      <Icon icon={PlayIcon} size={11} />
                      Resume
                    </button>
                  {:else}
                    <button
                      type="button"
                      class="control-btn pause-btn"
                      title="Pause download"
                      onclick={() => void pauseDownload(model.id)}
                    >
                      <Icon icon={PauseIcon} size={11} />
                      Pause
                    </button>
                  {/if}
                  <button
                    type="button"
                    class="control-btn cancel-btn"
                    title="Cancel download"
                    onclick={() => void cancelDownload(model.id)}
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </div>
          {:else}
            <div class="card-footer">
              {#if isActive}
                <span class="active-badge">
                  <Icon icon={CheckmarkCircle02Icon} size={14} />
                  Running on Metal
                </span>
                <Button size="compact" variant="danger" onclick={unloadModel}>Unload</Button>
              {:else if onDisk && storedPath}
                <div class="on-disk-actions">
                  <Button
                    size="compact"
                    variant="primary"
                    pending={loadingModel}
                    onclick={() => void loadModel(storedPath)}
                  >
                    <Icon icon={PlayIcon} size={14} />
                    Load Model
                  </Button>
                  <button
                    type="button"
                    class="delete-icon-btn"
                    title="Delete model from disk"
                    onclick={() => void deleteModel(model.filename)}
                  >
                    <Icon icon={Delete02Icon} size={14} />
                  </button>
                </div>
              {:else}
                <Button
                  size="compact"
                  variant="secondary"
                  onclick={() => void startDownload(model)}
                >
                  <Icon icon={Download01Icon} size={14} />
                  Download ({model.size_gb} GB)
                </Button>
              {/if}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </SettingsGroup>

  <!-- Stored Models on Disk -->
  {#if stored.length > 0}
    <SettingsGroup
      title="Downloaded Models on Disk"
      description="Models saved in ~/Library/Application Support/app.unexplore/models/"
    >
      <div class="stored-models-list">
        {#each stored as file (file.path)}
          {@const isActive = status.is_running && status.model_path === file.path}
          <div class="stored-row" class:active-row={isActive}>
            <div class="file-info">
              <span class="file-name">{file.name}</span>
              <span class="file-details">
                {file.formatted_size} · <code>{file.filename}</code>
              </span>
            </div>
            <div class="file-actions">
              {#if isActive}
                <Badge tone="accent">Active</Badge>
                <Button size="compact" variant="danger" onclick={unloadModel}>Unload</Button>
              {:else}
                <Button
                  size="compact"
                  variant="secondary"
                  pending={loadingModel}
                  onclick={() => void loadModel(file.path)}
                >
                  <Icon icon={PlayIcon} size={13} />
                  Load
                </Button>
                <button
                  type="button"
                  class="delete-icon-btn"
                  title="Delete from disk"
                  onclick={() => void deleteModel(file.filename)}
                >
                  <Icon icon={Delete02Icon} size={14} />
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    </SettingsGroup>
  {/if}

  <!-- Active Custom Downloads -->
  {#if customDownloads.length > 0}
    <SettingsGroup
      title="Active URL Downloads"
      description="Models downloading or paused from custom URLs."
    >
      <div class="custom-active-dls">
        {#each customDownloads as dl (dl.id)}
          <div class="custom-dl-item" class:paused={dl.status === "paused"}>
            <div class="custom-dl-header">
              <span class="custom-dl-id">{dl.id}</span>
              <span class="custom-dl-state-badge" class:paused-badge={dl.status === "paused"}>
                {dl.status === "paused" ? "Paused" : "Downloading"}
              </span>
            </div>
            <div class="dl-progress-bar">
              <div
                class="dl-fill"
                style="width: {dl.percent ?? 0}%"
                class:paused-fill={dl.status === "paused"}
              ></div>
            </div>
            <div class="dl-stats">
              {#if dl.status === "paused"}
                <span class="paused-text"
                  >Paused at {(dl.percent ?? 0).toFixed(1)}% ({formatBytes(
                    dl.downloaded_bytes ?? 0,
                  )} / {formatBytes(dl.total_bytes ?? 0)})</span
                >
              {:else}
                <span
                  >{(dl.percent ?? 0).toFixed(1)}% · {(dl.speed_mbps ?? 0).toFixed(1)} MB/s ({formatBytes(
                    dl.downloaded_bytes ?? 0,
                  )} / {formatBytes(dl.total_bytes ?? 0)})</span
                >
              {/if}
              <div class="dl-control-buttons">
                {#if dl.status === "paused"}
                  <button
                    type="button"
                    class="control-btn resume-btn"
                    title="Resume download"
                    onclick={() => void resumeDownload(dl.id)}
                  >
                    <Icon icon={PlayIcon} size={11} />
                    Resume
                  </button>
                {:else}
                  <button
                    type="button"
                    class="control-btn pause-btn"
                    title="Pause download"
                    onclick={() => void pauseDownload(dl.id)}
                  >
                    <Icon icon={PauseIcon} size={11} />
                    Pause
                  </button>
                {/if}
                <button
                  type="button"
                  class="control-btn cancel-btn"
                  title="Cancel download"
                  onclick={() => void cancelDownload(dl.id)}
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </SettingsGroup>
  {/if}

  <!-- Custom HuggingFace URL Downloader -->
  <SettingsGroup
    title="Download from URL"
    description="Paste any direct GGUF download URL from Hugging Face or other open-source repositories."
  >
    <div class="custom-dl-container">
      <div class="custom-dl-fields">
        <input
          type="url"
          class="url-input"
          placeholder="https://huggingface.co/.../model.gguf"
          bind:value={customUrl}
        />
        <input
          type="text"
          class="name-input"
          placeholder="Optional custom filename (e.g. my-model.gguf)"
          bind:value={customFilename}
        />
      </div>
      <Button
        size="compact"
        variant="primary"
        disabled={!customUrl.trim()}
        onclick={startCustomDownload}
      >
        <Icon icon={Download01Icon} size={14} />
        Download URL
      </Button>
    </div>
  </SettingsGroup>
</div>

<style>
  .local-models-hub {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding-bottom: 24px;
  }

  .engine-hero {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 20px 24px;
    border-radius: var(--radius-card, 16px);
    background: var(--color-surface, #1e1e24);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    margin: 0 16px;
  }

  .engine-hero.running {
    border-color: rgba(76, 175, 80, 0.35);
    background: linear-gradient(
      135deg,
      rgba(76, 175, 80, 0.08) 0%,
      var(--color-surface, #1e1e24) 60%
    );
  }

  .hero-left {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .status-indicator-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-caption, 11px);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted, #9e9e9e);
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #757575;
  }

  .status-dot.active {
    background: #4caf50;
    box-shadow: 0 0 10px rgba(76, 175, 80, 0.8);
  }

  .active-model-title {
    margin: 0;
    font-size: var(--text-title-2, 18px);
    font-weight: 600;
    color: var(--color-text, #ffffff);
  }

  .active-model-sub {
    margin: 0;
    font-size: var(--text-label, 13px);
    color: var(--color-muted, #a0a0a0);
  }

  .active-model-sub code {
    background: rgba(255, 255, 255, 0.06);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 12px;
  }

  .hero-tags {
    display: flex;
    gap: 8px;
    margin-top: 4px;
    flex-wrap: wrap;
  }

  .metal-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(255, 171, 0, 0.15);
    color: #ffb300;
    border: 1px solid rgba(255, 171, 0, 0.3);
    padding: 3px 9px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
  }

  .uncensored-pill {
    display: inline-flex;
    align-items: center;
    background: rgba(142, 68, 173, 0.18);
    color: #bb86fc;
    border: 1px solid rgba(142, 68, 173, 0.35);
    padding: 3px 9px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
  }

  .hero-actions {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-shrink: 0;
  }

  .error-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 16px;
    padding: 12px 16px;
    border-radius: 8px;
    background: rgba(244, 67, 54, 0.12);
    border: 1px solid rgba(244, 67, 54, 0.3);
    color: #ef5350;
    font-size: 13px;
  }

  /* Curated Models Grid */
  .curated-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 14px;
    padding: 14px 16px;
  }

  .curated-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    background: var(--color-surface, #1e1e24);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.08));
    border-radius: var(--radius-card, 12px);
    padding: 16px;
    transition:
      transform 0.15s ease,
      border-color 0.15s ease;
  }

  .curated-card:hover {
    border-color: rgba(255, 255, 255, 0.18);
  }

  .curated-card.active-card {
    border-color: rgba(76, 175, 80, 0.5);
    background: linear-gradient(
      180deg,
      rgba(76, 175, 80, 0.06) 0%,
      var(--color-surface, #1e1e24) 100%
    );
  }

  .card-header {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .card-title-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
  }

  .card-title-row h4 {
    margin: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--color-text, #ffffff);
  }

  .card-badges {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .card-desc {
    margin: 0;
    font-size: 12px;
    line-height: 1.4;
    color: var(--color-muted, #a0a0a0);
    min-height: 34px;
  }

  .card-meta {
    display: flex;
    gap: 16px;
    font-size: 11px;
    color: var(--color-muted, #888888);
  }

  .card-meta strong {
    color: var(--color-text, #e0e0e0);
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 14px;
    padding-top: 10px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .on-disk-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    justify-content: space-between;
  }

  .active-badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 500;
    color: #4caf50;
  }

  .delete-icon-btn {
    background: transparent;
    border: none;
    color: var(--color-muted, #757575);
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    transition:
      color 0.15s,
      background 0.15s;
  }

  .delete-icon-btn:hover {
    color: #f44336;
    background: rgba(244, 67, 54, 0.1);
  }

  .card-downloading {
    margin-top: 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .card-downloading.paused {
    background: rgba(245, 158, 11, 0.05);
    border-color: rgba(245, 158, 11, 0.2);
  }

  .dl-progress-bar {
    width: 100%;
    height: 7px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    overflow: hidden;
  }

  .dl-fill {
    height: 100%;
    background: linear-gradient(90deg, #3b82f6, #60a5fa);
    box-shadow: 0 0 10px rgba(59, 130, 246, 0.5);
    transition: width 0.25s ease;
  }

  .dl-fill.paused-fill {
    background: linear-gradient(90deg, #f59e0b, #fbbf24);
    box-shadow: 0 0 10px rgba(245, 158, 11, 0.4);
  }

  .dl-stats {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    color: var(--color-muted, #9e9e9e);
  }

  .paused-text {
    color: #fbbf24;
    font-weight: 500;
  }

  .dl-control-buttons {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .control-btn {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 4px;
    padding: 2px 7px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .control-btn.pause-btn {
    color: #fbbf24;
    background: rgba(245, 158, 11, 0.12);
    border-color: rgba(245, 158, 11, 0.28);
  }

  .control-btn.pause-btn:hover {
    background: rgba(245, 158, 11, 0.22);
    border-color: rgba(245, 158, 11, 0.45);
  }

  .control-btn.resume-btn {
    color: #60a5fa;
    background: rgba(59, 130, 246, 0.12);
    border-color: rgba(59, 130, 246, 0.28);
  }

  .control-btn.resume-btn:hover {
    background: rgba(59, 130, 246, 0.22);
    border-color: rgba(59, 130, 246, 0.45);
  }

  .control-btn.cancel-btn {
    color: #f87171;
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.22);
  }

  .control-btn.cancel-btn:hover {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.4);
  }

  .custom-active-dls {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px 16px;
  }

  .custom-dl-item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .custom-dl-item.paused {
    background: rgba(245, 158, 11, 0.05);
    border-color: rgba(245, 158, 11, 0.2);
  }

  .custom-dl-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .custom-dl-id {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text, #fff);
  }

  .custom-dl-state-badge {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
  }

  .custom-dl-state-badge.paused-badge {
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
  }

  /* Stored models list */
  .stored-models-list {
    display: flex;
    flex-direction: column;
    padding: 8px 16px;
  }

  .stored-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 14px;
    border-radius: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .stored-row:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .stored-row.active-row {
    background: rgba(76, 175, 80, 0.08);
  }

  .file-info {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .file-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text, #ffffff);
  }

  .file-details {
    font-size: 11px;
    color: var(--color-muted, #888888);
  }

  .file-details code {
    background: rgba(255, 255, 255, 0.05);
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 10px;
  }

  .file-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  /* Custom download */
  .custom-dl-container {
    display: flex;
    gap: 12px;
    padding: 14px 16px;
    align-items: flex-start;
  }

  .custom-dl-fields {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
  }

  .url-input,
  .name-input {
    width: 100%;
    box-sizing: border-box;
    padding: 8px 12px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid var(--color-border, rgba(255, 255, 255, 0.1));
    color: var(--color-text, #ffffff);
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
  }

  .url-input:focus,
  .name-input:focus {
    border-color: var(--color-accent, #3b82f6);
  }
</style>
