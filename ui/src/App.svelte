<script lang="ts">
  import Containers from './components/Containers.svelte';
  import Downloads from './components/Downloads.svelte';
  import Extensions from './components/Extensions.svelte';
  import LiveOverview from './components/LiveOverview.svelte';
  import Nextcloud from './components/Nextcloud.svelte';
  import Overview from './components/Overview.svelte';
  import Packages from './components/Packages.svelte';
  import Planned from './components/Planned.svelte';
  import Settings from './components/Settings.svelte';
  import Shares from './components/Shares.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import Ssh from './components/Ssh.svelte';
  import Welcome from './components/Welcome.svelte';
  import { t } from './lib/i18n.svelte';
  import {
    connected,
    disconnect,
    switchTo,
    syncLanguage,
    type Connected,
    type Failure,
  } from './lib/backend';
  import { demoDevices, demoOverview, demoUpdates } from './lib/nas';
  import { sections, type SectionId } from './lib/sections';
  import { persistAndApply, settings } from './lib/settings.svelte';

  /** Signed-in NAS from the core; the demo is kept apart. */
  let sessions = $state<Connected>({ devices: [], current: null });
  let demo = $state(false);
  /** The start screen is open to connect another NAS. */
  let adding = $state(false);
  let section = $state<SectionId | 'settings'>('overview');
  let notice = $state<Failure | null>(null);

  // Reads every setting, so it re-runs whenever one changes.
  $effect(persistAndApply);

  // Tray menu and notifications follow the language of the interface.
  $effect(() => {
    void syncLanguage(settings.language).catch(() => undefined);
  });

  const planned = $derived(sections.find((entry) => entry.id === section));
  const current = $derived.by(() => {
    if (demo) {
      const sample = demoDevices[0]!;
      return { id: 'demo', name: sample.name, model: sample.model };
    }
    return sessions.devices.find((device) => device.id === sessions.current) ?? null;
  });

  function show(next: Connected) {
    sessions = next;
    adding = false;
    section = 'overview';
  }

  async function opened() {
    notice = null;
    show(await connected());
  }

  function openDemo() {
    notice = null;
    demo = true;
    adding = false;
    section = 'overview';
  }

  /** The user disconnects the current NAS; another signed-in one takes its place. */
  async function leave() {
    if (demo) {
      demo = false;
      return;
    }
    show(await disconnect().catch(() => ({ devices: [], current: null })));
  }

  /** The core ended the current session (restart, shutdown, expired sign-in). */
  async function lost(reason: Failure) {
    notice = reason;
    show(await connected().catch(() => ({ devices: [], current: null })));
  }

  async function switchDevice(id: string) {
    notice = null;
    show(await switchTo(id));
  }
</script>

{#if current && !adding}
  <div class="shell">
    <Sidebar
      device={current}
      devices={demo ? [] : sessions.devices}
      bind:section
      ondisconnect={leave}
      onswitch={demo ? undefined : switchDevice}
      onadd={demo ? undefined : () => (adding = true)}
    />
    <main>
      {#if notice}
        <p class="notice" role="status">{t(`error-${notice.key}`)}</p>
      {/if}
      {#key current.id}
        {#if section === 'settings'}
          <Settings />
        {:else if section === 'overview'}
          {#if demo}
            <Overview overview={demoOverview} name={current.name} demo updates={demoUpdates} />
          {:else}
            <LiveOverview name={current.name} onlost={lost} />
          {/if}
        {:else if section === 'files'}
          <Shares {demo} />
        {:else if section === 'packages'}
          <Packages
            {demo}
            device={demo ? null : (sessions.devices.find((d) => d.id === sessions.current) ?? null)}
            onssh={() => (section = 'ssh')}
          />
        {:else if section === 'containers'}
          <Containers
            {demo}
            device={demo ? null : (sessions.devices.find((d) => d.id === sessions.current) ?? null)}
            onssh={() => (section = 'ssh')}
          />
        {:else if section === 'downloads'}
          <Downloads {demo} />
        {:else if section === 'extensions'}
          <Extensions
            {demo}
            device={demo ? null : (sessions.devices.find((d) => d.id === sessions.current) ?? null)}
            onssh={() => (section = 'ssh')}
          />
        {:else if section === 'nextcloud'}
          <Nextcloud
            {demo}
            device={demo ? null : (sessions.devices.find((d) => d.id === sessions.current) ?? null)}
            onssh={() => (section = 'ssh')}
          />
        {:else if section === 'ssh'}
          <Ssh
            {demo}
            device={demo ? null : (sessions.devices.find((d) => d.id === sessions.current) ?? null)}
            onchange={async () => (sessions = await connected())}
          />
        {:else if planned}
          <Planned section={planned} />
        {/if}
      {/key}
    </main>
  </div>
{:else}
  <Welcome
    onconnected={opened}
    ondemo={openDemo}
    {notice}
    onback={current ? () => (adding = false) : undefined}
  />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    grid-template-rows: minmax(0, 1fr);
    height: 100%;
  }

  .notice {
    max-width: 1080px;
    margin: 0 auto 16px;
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--warning) 16%, transparent);
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
