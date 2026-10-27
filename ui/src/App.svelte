<script lang="ts">
  import LiveOverview from './components/LiveOverview.svelte';
  import Overview from './components/Overview.svelte';
  import Planned from './components/Planned.svelte';
  import Settings from './components/Settings.svelte';
  import Shares from './components/Shares.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Welcome from './components/Welcome.svelte';
  import { disconnect, type Failure, type KnownDevice } from './lib/backend';
  import { demoDevices, demoOverview, demoUpdates } from './lib/nas';
  import { sections, type SectionId } from './lib/sections';
  import { persistAndApply } from './lib/settings.svelte';

  interface Current {
    name: string;
    model: string;
    demo: boolean;
  }

  let current = $state<Current | null>(null);
  let section = $state<SectionId | 'settings'>('overview');
  let notice = $state<Failure | null>(null);

  // Reads every setting, so it re-runs whenever one changes.
  $effect(persistAndApply);

  const planned = $derived(sections.find((entry) => entry.id === section));

  function open(next: Current) {
    current = next;
    section = 'overview';
    notice = null;
  }

  function openDevice(device: KnownDevice) {
    open({ name: device.name, model: device.model, demo: false });
  }

  function openDemo() {
    const sample = demoDevices[0]!;
    open({ name: sample.name, model: sample.model, demo: true });
  }

  async function leave(reason: Failure | null = null) {
    const wasDemo = current?.demo ?? true;
    current = null;
    notice = reason;
    if (!wasDemo) await disconnect().catch(() => undefined);
  }
</script>

{#if current}
  <div class="shell">
    <Sidebar device={current} bind:section ondisconnect={() => leave()} />
    <main>
      {#if section === 'settings'}
        <Settings />
      {:else if section === 'overview'}
        {#if current.demo}
          <Overview overview={demoOverview} name={current.name} demo updates={demoUpdates} />
        {:else}
          <LiveOverview name={current.name} onlost={leave} />
        {/if}
      {:else if section === 'files'}
        <Shares demo={current.demo} />
      {:else if planned}
        <Planned section={planned} />
      {/if}
    </main>
  </div>
{:else}
  <Welcome onconnected={openDevice} ondemo={openDemo} {notice} />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    grid-template-rows: minmax(0, 1fr);
    height: 100%;
  }

  main {
    overflow-y: auto;
    padding: 28px 32px 40px;
  }

  @media (max-width: 720px) {
    .shell {
      grid-template-columns: 64px 1fr;
    }

    main {
      padding: 20px 16px 32px;
    }
  }
</style>
