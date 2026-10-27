<script lang="ts">
  import * as m from "$shared/i18n/messages";
  import { tabs } from "$domain/tabs";
  import PreviewDialog from "./PreviewDialog.svelte";
  import Button from "$shared/ui/Button";
  import Field from "$shared/ui/Field";
  import Select from "$shared/ui/Select";
  import SearchField from "$shared/ui/SearchField";

  type ExceptionEntry = { id: string; name: string; value: string };

  const kinds = [
    { value: "camera", label: m.pref_privacy_camera },
    { value: "microphone", label: m.pref_privacy_microphone },
    { value: "location", label: m.pref_privacy_location },
    { value: "notifications", label: m.pref_privacy_notifications },
    { value: "popups", label: m.permission_popups },
    { value: "clipboard", label: m.permission_clipboard },
  ];
  const policies = [
    { value: "ask", label: m.permission_ask },
    { value: "allow", label: m.permission_allow },
    { value: "block", label: m.permission_block },
  ];

  function storageKey() {
    return `unexplore:privacy:site-exceptions:${tabs.profile()?.id ?? "initial"}`;
  }

  function loadEntries(): ExceptionEntry[] {
    try {
      const raw = localStorage.getItem(storageKey());
      if (raw) return JSON.parse(raw);
    } catch {}
    return [];
  }

  let allEntries = $state<ExceptionEntry[]>(loadEntries());

  function persistEntries(items: ExceptionEntry[]) {
    allEntries = items;
    try {
      localStorage.setItem(storageKey(), JSON.stringify(items));
    } catch {}
  }

  function save(entry: ExceptionEntry) {
    const idx = allEntries.findIndex((item) => item.id === entry.id);
    const updated =
      idx < 0
        ? [...allEntries, entry].slice(0, 100)
        : allEntries.map((item) => (item.id === entry.id ? entry : item));
    persistEntries(updated);
  }

  function deleteEntry(removeId: string) {
    persistEntries(allEntries.filter((item) => item.id !== removeId));
  }

  let open = $state(false);
  let remove = $state<string | null>(null);
  let id = $state("");
  let site = $state("");
  let kind = $state("camera");
  let policy = $state("ask");
  let query = $state("");

  let origin = $derived.by(() => {
    try {
      const url = new URL(site);
      return ["https:", "http:"].includes(url.protocol) ? url.origin : null;
    } catch {
      return null;
    }
  });

  let duplicate = $derived(
    allEntries.some(
      (entry) => entry.id !== id && entry.name === origin && entry.value.split(":")[0] === kind,
    ),
  );

  let entries = $derived(
    allEntries.filter((entry) =>
      (entry.name + " " + entry.value).toLowerCase().includes(query.toLowerCase()),
    ),
  );

  function edit(entry?: ExceptionEntry) {
    id = entry?.id ?? crypto.randomUUID();
    site = entry?.name ?? "";
    kind = entry?.value.split(":")[0] ?? "camera";
    policy = entry?.value.split(":")[1] ?? "ask";
    open = true;
  }
</script>

<section class="settings-collection" data-setting="privacy.exceptions">
  <header>
    <div>
      <h2>{m.pref_privacy_exceptions()}</h2>
      <p>{m.permission_context()}</p>
    </div>
    <Button
      onclick={() => edit()}
      disabled={allEntries.length >= 100}>{m.action_add()}</Button
    >
  </header>
  {#if allEntries.length}<SearchField
      label={m.settings_search()}
      placeholder={m.settings_search()}
      bind:value={query}
    />{/if}
  {#if entries.length === 0}<div class="collection-empty">{m.permission_empty()}</div>{:else}<ul>
      {#each entries as entry (entry.id)}{@const parts = entry.value.split(":")}
        <li>
          <button type="button" class="collection-entry" onclick={() => edit(entry)}
            ><strong>{entry.name}</strong><span
              >{kinds.find((item) => item.value === parts[0])?.label()} · {policies
                .find((item) => item.value === parts[1])
                ?.label()}</span
            ></button
          ><Button variant="ghost" onclick={() => (remove = entry.id)}>{m.action_remove()}</Button>
        </li>{/each}
    </ul>{/if}
</section>
<PreviewDialog
  bind:open
  title={m.permission_edit()}
  description={m.permission_context()}
  valid={origin !== null && !duplicate}
  onapply={() => {
    save({ id, name: origin!, value: `${kind}:${policy}` });
    open = false;
  }}
  ><Field
    label={m.permission_site()}
    bind:value={site}
    type="url"
    placeholder="https://example.com"
    required
    maxlength={500}
    error={site && !origin
      ? m.permission_invalid()
      : duplicate
        ? m.permission_duplicate()
        : undefined}
  /><Select
    label={m.permission_kind()}
    bind:value={kind}
    options={kinds.map((item) => ({ value: item.value, label: item.label() }))}
  /><Select
    label={m.permission_behavior()}
    bind:value={policy}
    options={policies.map((item) => ({ value: item.value, label: item.label() }))}
  /></PreviewDialog
>
<PreviewDialog
  open={remove !== null}
  title={m.preview_delete_title()}
  description={m.preview_delete_body()}
  onclose={() => (remove = null)}
  onapply={() => {
    if (remove) deleteEntry(remove);
    remove = null;
  }}>{m.action_remove()}</PreviewDialog
>
