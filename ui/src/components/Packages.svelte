<script lang="ts">
  import PackageIcon from '@lucide/svelte/icons/package';
  import Search from '@lucide/svelte/icons/search';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import {
    addPackageSource,
    hasUpdate,
    installPackage,
    packageSources,
    packages as fetchPackages,
    removePackageSource,
    toFailure,
    type Failure,
    type KnownDevice,
    type Package,
    type PackageSources,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import { demoPackages } from '../lib/nas';
  import { settings } from '../lib/settings.svelte';

  let {
    device,
    demo,
    onssh,
  }: {
    device: KnownDevice | null;
    demo: boolean;
    /** Opens the SSH section. */
    onssh: () => void;
  } = $props();

  const SYNOCOMMUNITY = 'https://packages.synocommunity.com/';

  type Tab = 'installed' | 'catalogue' | 'sources';

  let tab = $state<Tab>('installed');
  let list = $state<Package[] | null>(null);
  let failure = $state<Failure | null>(null);
  let query = $state('');
  let busy = $state<string | null>(null);
  let done = $state<string[] | null>(null);
  /** The install waiting for the sudo password. */
  let waiting = $state<Package | null>(null);
  let password = $state('');

  let sources = $state<PackageSources | null>(null);
  let sourceName = $state('');
  let sourceUrl = $state('');
  let trust = $state(true);
  let sourceBusy = $state(false);

  const linked = $derived(demo || device?.ssh !== undefined);
  const installed = $derived(
    (list ?? [])
      .filter((pkg) => pkg.installed)
      .sort((a, b) => Number(hasUpdate(b)) - Number(hasUpdate(a))),
  );
  const updates = $derived(installed.filter(hasUpdate));
  const offered = $derived.by(() => {
    const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return (list ?? []).filter(
      (pkg) =>
        !pkg.installed &&
        pkg.available &&
        words.every((word) =>
          `${pkg.name} ${pkg.description} ${pkg.publisher ?? ''}`.toLowerCase().includes(word),
        ),
    );
  });

  async function load() {
    failure = null;
    if (demo) {
      list = demoPackages;
      sources = { feeds: [{ name: 'SynoCommunity', url: SYNOCOMMUNITY }], trustLevel: 1 };
      return;
    }
    try {
      list = await fetchPackages();
    } catch (error) {
      failure = toFailure(error);
    }
  }

  $effect(() => {
    void load();
  });

  $effect(() => {
    if (tab === 'sources' && !demo && sources === null) {
      packageSources()
        .then((result) => (sources = result))
        .catch((error) => (failure = toFailure(error)));
    }
  });

  async function install(pkg: Package) {
    busy = pkg.id;
    failure = null;
    done = null;
    try {
      // A typed password stays in memory while this page is open, so updating
      // several packages asks once.
      done = await installPackage(pkg.id, password === '' ? undefined : password);
      waiting = null;
      await load();
    } catch (error) {
      const reason = toFailure(error);
      if (reason.key === 'password-required') waiting = pkg;
      else if (reason.key === 'sudo') {
        password = '';
        waiting = pkg;
        failure = reason;
      } else failure = reason;
    } finally {
      busy = null;
    }
  }

  async function updateAll() {
    for (const pkg of updates) {
      await install(pkg);
      if (failure || waiting) break;
    }
  }

  async function addSource(name: string, url: string) {
    sourceBusy = true;
    failure = null;
    try {
      sources = await addPackageSource(name, url, trust);
      sourceName = '';
      sourceUrl = '';
      await load();
    } catch (error) {
      failure = toFailure(error);
    } finally {
      sourceBusy = false;
    }
  }

  async function removeSource(url: string) {
    sourceBusy = true;
    failure = null;
    try {
      sources = await removePackageSource(url);
      await load();
    } catch (error) {
      failure = toFailure(error);
    } finally {
      sourceBusy = false;
    }
  }
</script>

<div class="packages">
  <header>
    <h1>{t('nav-packages')}</h1>
    <p class="muted">{t('packages-intro')}</p>
  </header>

  <div class="tabs" role="tablist">
    <button role="tab" aria-selected={tab === 'installed'} onclick={() => (tab = 'installed')}>
      {t('packages-installed')}
      {#if updates.length > 0}<span class="badge warning">{updates.length}</span>{/if}
    </button>
    <button role="tab" aria-selected={tab === 'catalogue'} onclick={() => (tab = 'catalogue')}>
      {t('packages-catalogue')}
    </button>
    {#if settings.expert}
      <button role="tab" aria-selected={tab === 'sources'} onclick={() => (tab = 'sources')}>
        {t('packages-sources')}
      </button>
    {/if}
  </div>

  {#if !linked && tab !== 'sources'}
    <p class="hint">
      <span>{t('packages-needs-ssh')}</span>
      <button class="button" onclick={onssh}>{t('packages-set-up-ssh')}</button>
    </p>
  {/if}

  {#if waiting}
    <form
      class="card password"
      onsubmit={(event) => {
        event.preventDefault();
        if (waiting) void install(waiting);
      }}
    >
      <strong>{t('packages-password-title', { name: waiting.name })}</strong>
      <span class="muted">{t('packages-password-text')}</span>
      <div class="row">
        <input
          class="input"
          type="password"
          bind:value={password}
          autocomplete="current-password"
          required
        />
        <button type="button" class="button" onclick={() => (waiting = null)}>
          {t('signin-cancel')}
        </button>
        <button class="button primary" disabled={busy !== null}>{t('packages-install')}</button>
      </div>
    </form>
  {/if}

  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}
  {#if done && done.length > 0}
    <p class="done" role="status">{t('packages-done', { names: done.join(', ') })}</p>
  {/if}

  {#snippet row(pkg: Package, action: 'update' | 'install' | null)}
    <li>
      <PackageIcon size={20} strokeWidth={1.7} aria-hidden="true" />
      <div class="info">
        <strong>
          {pkg.name}
          {#if pkg.publisher}<span class="badge">{pkg.publisher}</span>{/if}
          {#if pkg.beta}<span class="badge warning">Beta</span>{/if}
        </strong>
        {#if pkg.description}<span class="subtle clamp">{pkg.description}</span>{/if}
        <span class="subtle">
          {#if action === 'update'}
            {pkg.installed} → {pkg.available}
          {:else if pkg.installed}
            {pkg.installed}
            {#if pkg.status === 'stop'}· {t('packages-stopped')}{/if}
          {:else}
            {pkg.available}
            {#if pkg.depends.length > 0}· {t('packages-needs', {
                names: pkg.depends.join(', '),
              })}{/if}
          {/if}
        </span>
      </div>
      {#if action}
        <button
          class="button {action === 'update' ? 'primary' : ''}"
          disabled={demo || !linked || busy !== null}
          onclick={() => install(pkg)}
        >
          {#if busy === pkg.id}
            {t(action === 'update' ? 'packages-updating' : 'packages-installing')}
          {:else}
            {t(action === 'update' ? 'packages-update' : 'packages-install')}
          {/if}
        </button>
      {/if}
    </li>
  {/snippet}

  {#if list === null && !failure}
    <p class="muted">{t('overview-loading')}</p>
  {:else if tab === 'installed'}
    {#if updates.length > 1}
      <div class="actions">
        <button
          class="button primary"
          disabled={demo || !linked || busy !== null}
          onclick={updateAll}
        >
          {t('packages-update-all', { count: updates.length })}
        </button>
      </div>
    {/if}
    <section class="card">
      <ul>
        {#each installed as pkg (pkg.id)}
          {@render row(pkg, hasUpdate(pkg) ? 'update' : null)}
        {:else}
          <li class="subtle">{t('packages-none')}</li>
        {/each}
      </ul>
    </section>
  {:else if tab === 'catalogue'}
    <label class="search">
      <Search size={16} aria-hidden="true" />
      <input class="input" type="search" bind:value={query} placeholder={t('packages-search')} />
    </label>
    <section class="card">
      <ul>
        {#each offered as pkg (pkg.id)}
          {@render row(pkg, 'install')}
        {:else}
          <li class="subtle">{t('packages-no-match')}</li>
        {/each}
      </ul>
    </section>
    <p class="subtle">{t('packages-wizard-hint')}</p>
  {:else if tab === 'sources'}
    <p class="warning">
      <TriangleAlert size={16} aria-hidden="true" />
      <span>{t('packages-sources-warning')}</span>
    </p>
    {#if sources?.trustLevel === 0}
      <label class="check card">
        <input type="checkbox" bind:checked={trust} />
        <span>
          {t('packages-trust')}
          <span class="subtle block">{t('packages-trust-hint')}</span>
        </span>
      </label>
    {/if}
    <section class="card">
      <ul>
        {#each sources?.feeds ?? [] as feed (feed.url)}
          <li>
            <div class="info">
              <strong>{feed.name}</strong>
              <code class="subtle">{feed.url}</code>
            </div>
            <button
              class="button"
              disabled={demo || sourceBusy}
              onclick={() => removeSource(feed.url)}
            >
              {t('packages-source-remove')}
            </button>
          </li>
        {:else}
          <li class="subtle">{t('packages-sources-none')}</li>
        {/each}
      </ul>
    </section>
    {#if sources && !sources.feeds.some((feed) => feed.url === SYNOCOMMUNITY)}
      <section class="card stack">
        <strong>SynoCommunity</strong>
        <span class="muted">{t('packages-synocommunity-text')}</span>
        <div class="actions">
          <button
            class="button primary"
            disabled={demo || sourceBusy}
            onclick={() => addSource('SynoCommunity', SYNOCOMMUNITY)}
          >
            {t('packages-source-add')}
          </button>
        </div>
      </section>
    {/if}
    <form
      class="card stack"
      onsubmit={(event) => {
        event.preventDefault();
        void addSource(sourceName, sourceUrl);
      }}
    >
      <strong>{t('packages-source-other')}</strong>
      <div class="row">
        <input
          class="input"
          bind:value={sourceName}
          placeholder={t('packages-source-name')}
          required
        />
        <input
          class="input grow"
          type="url"
          bind:value={sourceUrl}
          placeholder="https://"
          required
        />
      </div>

      <div class="actions">
        <button class="button" disabled={demo || sourceBusy}>{t('packages-source-add')}</button>
      </div>
    </form>
  {/if}
  {#if demo}<p class="subtle">{t('packages-demo')}</p>{/if}
</div>

<style>
  .packages {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 820px;
    margin: 0 auto;
  }

  header,
  .stack,
  .password {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: var(--radius-control);
    background: var(--track);
    align-self: flex-start;
  }

  .tabs button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 14px;
    border: 0;
    border-radius: calc(var(--radius-control) - 2px);
    background: transparent;
    color: var(--text-2);
    font: inherit;
    cursor: pointer;
  }

  .tabs button[aria-selected='true'] {
    background: var(--card);
    color: var(--text);
    box-shadow: var(--card-shadow);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
    color: var(--text-2);
  }

  li + li {
    border-top: 1px solid var(--card-border);
  }

  .info {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    color: var(--text);
    line-height: 1.35;
  }

  .info strong {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }

  .clamp {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-3);
  }

  .search input {
    flex: 1;
  }

  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .row .input {
    min-width: 10em;
  }

  .grow {
    flex: 1;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .hint,
  .warning {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--warning) 16%, transparent);
  }

  .hint span {
    flex: 1;
  }

  .warning {
    align-items: flex-start;
  }

  .warning :global(svg) {
    flex: none;
    margin-top: 2px;
    color: var(--warning);
  }

  .check {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    cursor: pointer;
  }

  .check input {
    margin-top: 3px;
    accent-color: var(--accent);
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
    word-break: break-all;
    user-select: text;
  }

  .failure {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    user-select: text;
  }

  .done {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--success) 16%, transparent);
  }

  .block {
    display: block;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
