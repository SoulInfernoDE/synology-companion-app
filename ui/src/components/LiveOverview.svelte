<script lang="ts">
  import Overview from './Overview.svelte';
  import PowerActions from './PowerActions.svelte';
  import {
    overview as fetchOverview,
    toFailure,
    updates as fetchUpdates,
    type Failure,
    type Updates,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import type { Overview as OverviewData } from '../lib/nas';

  /** Refresh interval; DSM itself refreshes its widgets about this often. */
  const REFRESH_MS = 10_000;
  /** Update checks ask Synology's servers; once on opening and then every half hour. */
  const UPDATES_MS = 30 * 60_000;

  let { name, onlost }: { name: string; onlost: (reason: Failure) => void } = $props();

  let data = $state<OverviewData | null>(null);
  let failure = $state<Failure | null>(null);
  let updates = $state<Updates | null | undefined>(undefined);

  async function loadUpdates() {
    try {
      updates = await fetchUpdates();
    } catch {
      updates = null;
    }
  }

  async function load() {
    try {
      data = await fetchOverview();
      failure = null;
    } catch (error) {
      const reason = toFailure(error);
      if (reason.key === 'session-expired' || reason.key === 'not-connected') onlost(reason);
      else failure = reason;
    }
  }

  $effect(() => {
    void load();
    void loadUpdates();
    const timer = setInterval(load, REFRESH_MS);
    const updateTimer = setInterval(loadUpdates, UPDATES_MS);
    return () => {
      clearInterval(timer);
      clearInterval(updateTimer);
    };
  });
</script>

{#if failure}
  <p class="failure" role="alert">
    {data ? t('overview-refresh-failed') : t(`error-${failure.key}`)}
    {#if failure.key === 'permission-denied'}<span class="block">{t('overview-account-hint')}</span
      >{/if}
    {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
  </p>
{/if}

{#if data}
  <Overview overview={data} {name} demo={false} {updates}>
    {#snippet actions()}
      <PowerActions
        ondone={(action) => onlost({ key: action === 'reboot' ? 'rebooting' : 'shutting-down' })}
      />
    {/snippet}
  </Overview>
{:else if !failure}
  <p class="loading muted">{t('overview-loading')}</p>
{/if}

<style>
  .failure {
    max-width: 1080px;
    margin: 0 auto 16px;
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    user-select: text;
  }

  .block {
    display: block;
  }

  .loading {
    margin-top: 20vh;
    text-align: center;
  }
</style>
