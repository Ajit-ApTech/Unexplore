<script lang="ts">
  import * as m from "$shared/i18n/messages";
  import { commands } from "$shared/ipc/bindings";
  import { tabs } from "$domain/tabs";

  let profile = $derived(tabs.profile()?.id ?? "");
  let stats = $state<number | null>(null);
  let last7Days = $state<number | null>(null);
  let days = $state<(number | null)[]>([]);
  let loading = $state(true);

  $effect(() => {
    if (!profile) return;
    loading = true;
    void commands.blockerStats(profile).then(
      (result) => {
        if (result.status === "ok" && result.data) {
          stats = result.data.today;
          last7Days = result.data.last7Days;
          days = result.data.days;
        }
        loading = false;
      },
      () => {
        loading = false;
      },
    );
  });
</script>

<div class="privacy-dashboard" aria-label={m.privacy_dashboard()}>
  {#if loading}
    <div class="loading">Loading tracker statistics…</div>
  {:else if stats !== null || last7Days !== null}
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-number">{stats ?? 0}</div>
        <div class="stat-label">{m.ntp_trackers_blocked()}</div>
      </div>
      <div class="stat-card">
        <div class="stat-number">{last7Days ?? 0}</div>
        <div class="stat-label">Last 7 days</div>
      </div>
    </div>

    {#if days.length > 0}
      <div class="chart">
        <h3>Trackers over time</h3>
        <div class="bars">
          {#each days as count, i (i)}
            <div
              class="bar"
              title={count !== null ? `Day ${i + 1}: ${count} blocked` : undefined}
              style="height: {count !== null ? (count / (last7Days ?? 1)) * 100 : 0}%"
              aria-hidden="true"
            ></div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="no-data">No tracker data available</div>
  {/if}
</div>

<style>
  .privacy-dashboard {
    padding: var(--sidebar-inset);
  }

  .stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: var(--sidebar-row-gap);
    margin-bottom: var(--sidebar-row-gap);
  }

  .stat-card {
    background: var(--color-card);
    border-radius: var(--radius-card);
    padding: var(--sidebar-row-text) 0;
    text-align: center;
  }

  .stat-number {
    font-size: var(--text-title);
    font-weight: var(--sidebar-row-weight-current);
    color: var(--color-lit);
    margin-bottom: 4px;
  }

  .stat-label {
    font-size: var(--text-caption);
    color: var(--color-muted);
  }

  .chart {
    margin-top: var(--sidebar-row-gap);
  }

  .chart h3 {
    font-size: var(--text-label);
    margin-bottom: var(--sidebar-row-text);
  }

  .bars {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .bar {
    background: var(--color-fill);
    border-radius: 2px;
    transition: height var(--motion-fast) var(--ease-out);
  }
</style>
