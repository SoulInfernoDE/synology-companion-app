<script lang="ts">
  import KeyRound from '@lucide/svelte/icons/key-round';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import ShieldAlert from '@lucide/svelte/icons/shield-alert';
  import ShieldQuestion from '@lucide/svelte/icons/shield-question';
  import SquareTerminal from '@lucide/svelte/icons/square-terminal';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import {
    diagnostics as fetchDiagnostics,
    enableSsh,
    memoryModules,
    sshLink,
    sshService,
    sshUnlink,
    toFailure,
    type Diagnostics,
    type Failure,
    type KnownDevice,
    type MemoryModule,
    type SshService,
  } from '../lib/backend';
  import { bytes } from '../lib/format';
  import { t } from '../lib/i18n.svelte';
  import { demoDiagnostics, demoMemory } from '../lib/nas';
  import { settings } from '../lib/settings.svelte';

  let {
    device,
    demo,
    onchange,
  }: {
    /** The current NAS; `null` in the demo. */
    device: KnownDevice | null;
    demo: boolean;
    /** The SSH link of the device changed. */
    onchange: () => void;
  } = $props();

  type Step = 'intro' | 'password' | 'hostKey' | 'confirmUnlink';

  let service = $state<SshService | null>(null);
  let step = $state<Step>('intro');
  // svelte-ignore state_referenced_locally
  let port = $state(device?.ssh?.port ?? 22);
  let password = $state('');
  let trustFingerprint = $state<string | undefined>();
  let hostKey = $state<{ fingerprint: string; changed: boolean } | null>(null);
  let busy = $state(false);
  let failure = $state<Failure | null>(null);

  let report = $state<Diagnostics | null>(null);
  let reportFailure = $state<Failure | null>(null);
  let loadingReport = $state(false);
  let memory = $state<MemoryModule[] | null>(null);
  let memoryFailure = $state<Failure | null>(null);
  let memoryPassword = $state('');
  let askMemoryPassword = $state(false);
  let readingMemory = $state(false);
  let terminal = $state(false);

  const linked = $derived(demo || device?.ssh !== undefined);

  $effect(() => {
    if (demo) {
      report = demoDiagnostics;
      return;
    }
    sshService()
      .then((result) => {
        service = result;
        if (!device?.ssh) port = result.port;
      })
      .catch((error) => (failure = toFailure(error)));
  });

  $effect(() => {
    if (!demo && device?.ssh) void loadReport();
  });

  async function loadReport() {
    loadingReport = true;
    reportFailure = null;
    try {
      report = await fetchDiagnostics();
    } catch (error) {
      reportFailure = toFailure(error);
    } finally {
      loadingReport = false;
    }
  }

  const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

  async function link() {
    busy = true;
    failure = null;
    try {
      if (service && !service.enabled) {
        await enableSsh(port);
        service = { ...service, enabled: true, port };
        // DSM needs a moment to start its SSH server.
        await wait(2000);
      }
      const outcome = await sshLink({
        port,
        password: password === '' ? undefined : password,
        trustFingerprint,
      });
      switch (outcome.status) {
        case 'linked':
          password = '';
          step = 'intro';
          onchange();
          break;
        case 'passwordRequired':
          step = 'password';
          break;
        case 'untrustedHostKey':
        case 'hostKeyChanged':
          hostKey = {
            fingerprint: outcome.fingerprint,
            changed: outcome.status === 'hostKeyChanged',
          };
          step = 'hostKey';
          break;
        case 'failed':
          failure = { key: outcome.key, detail: outcome.detail };
          break;
      }
    } catch (error) {
      failure = toFailure(error);
    } finally {
      busy = false;
    }
  }

  function trust() {
    trustFingerprint = hostKey?.fingerprint;
    hostKey = null;
    void link();
  }

  function cancel() {
    step = 'intro';
    password = '';
    hostKey = null;
    trustFingerprint = undefined;
    failure = null;
  }

  async function unlink() {
    busy = true;
    failure = null;
    try {
      await sshUnlink();
      step = 'intro';
      terminal = false;
      report = null;
      memory = null;
      trustFingerprint = undefined;
      onchange();
    } catch (error) {
      failure = toFailure(error);
    } finally {
      busy = false;
    }
  }

  async function readMemory() {
    if (demo) {
      memory = demoMemory;
      return;
    }
    readingMemory = true;
    memoryFailure = null;
    try {
      memory = await memoryModules(memoryPassword === '' ? undefined : memoryPassword);
      askMemoryPassword = false;
      memoryPassword = '';
    } catch (error) {
      const failure = toFailure(error);
      if (failure.key === 'password-required') askMemoryPassword = true;
      else memoryFailure = failure;
    } finally {
      readingMemory = false;
    }
  }

  function dsmText(report: Diagnostics): string {
    if (!report.dsmVersion) return t('ssh-diag-unknown');
    const build = report.dsmBuild ? `-${report.dsmBuild}` : '';
    const update = report.dsmUpdate ? ` ${t('ssh-diag-update', { update: report.dsmUpdate })}` : '';
    return `DSM ${report.dsmVersion}${build}${update}`;
  }

  function speedText(speed: number | null): string {
    if (speed === null || speed <= 0) return t('ssh-diag-no-link');
    return speed >= 1000 ? `${speed / 1000} Gbit/s` : `${speed} Mbit/s`;
  }
</script>

<div class="ssh">
  <header>
    <h1>{t('nav-ssh')}</h1>
    <p class="muted">{t('ssh-intro')}</p>
  </header>

  {#snippet problem(failure: Failure | null)}
    {#if failure}
      <p class="failure" role="alert">
        {t(`error-${failure.key}`)}
        {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
      </p>
    {/if}
  {/snippet}

  <section class="card setup">
    {#if linked}
      <div class="status">
        <span class="icon"><KeyRound size={20} aria-hidden="true" /></span>
        <div class="grow">
          <strong>{t('ssh-linked-title')}</strong>
          <span class="subtle block">
            {t('ssh-linked-text', {
              account: device?.account ?? 'demo',
              port: String(device?.ssh?.port ?? 22),
            })}
          </span>
          {#if device?.ssh}<code class="subtle">{device.ssh.hostKey}</code>{/if}
        </div>
        {#if step !== 'confirmUnlink'}
          <button class="button" disabled={demo || busy} onclick={() => (step = 'confirmUnlink')}>
            {t('ssh-unlink')}
          </button>
        {/if}
      </div>
      {#if step === 'confirmUnlink'}
        <p class="muted">{t('ssh-unlink-text')}</p>
        <div class="actions">
          <button class="button" onclick={cancel}>{t('signin-cancel')}</button>
          <button class="button danger" disabled={busy} onclick={unlink}>
            {busy ? t('signin-busy') : t('ssh-unlink-confirm')}
          </button>
        </div>
      {/if}
      {@render problem(failure)}
    {:else if step === 'hostKey' && hostKey}
      <div class="status">
        <span class="icon" class:danger={hostKey.changed}>
          {#if hostKey.changed}
            <ShieldAlert size={20} aria-hidden="true" />
          {:else}
            <ShieldQuestion size={20} aria-hidden="true" />
          {/if}
        </span>
        <div class="grow">
          <strong>{t(hostKey.changed ? 'ssh-hostkey-changed-title' : 'ssh-hostkey-title')}</strong>
          <span class="muted block">
            {t(hostKey.changed ? 'ssh-hostkey-changed-text' : 'ssh-hostkey-text')}
          </span>
        </div>
      </div>
      <div class="fingerprint">
        <span class="subtle">{t('signin-cert-fingerprint')}</span>
        <code>{hostKey.fingerprint}</code>
      </div>
      <div class="actions">
        <button class="button" onclick={cancel}>{t('signin-cancel')}</button>
        <button class="button {hostKey.changed ? 'danger' : 'primary'}" onclick={trust}>
          {t(hostKey.changed ? 'signin-cert-trust-changed' : 'ssh-hostkey-trust')}
        </button>
      </div>
    {:else}
      <div class="status">
        <span class="icon"><SquareTerminal size={20} aria-hidden="true" /></span>
        <div class="grow">
          <strong>{t('ssh-setup-title')}</strong>
          <span class="muted block">{t('ssh-setup-text')}</span>
        </div>
      </div>
      <ul class="steps">
        {#if service && !service.enabled}<li>{t('ssh-setup-step-enable')}</li>{/if}
        <li>{t('ssh-setup-step-key')}</li>
        <li>{t('ssh-setup-step-install')}</li>
        <li>{t('ssh-setup-step-keychain')}</li>
      </ul>
      {#if service && !service.homes}
        <p class="warning">
          <TriangleAlert size={16} aria-hidden="true" />
          <span>{t('ssh-homes-off')}</span>
        </p>
      {/if}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void link();
        }}
      >
        <label class="field port">
          <span>{t('ssh-port')}</span>
          <input class="input" type="number" min="1" max="65535" bind:value={port} required />
        </label>
        {#if step === 'password'}
          <label class="field">
            <span>{t('ssh-password', { account: device?.account ?? '' })}</span>
            <input
              class="input"
              type="password"
              bind:value={password}
              autocomplete="current-password"
              required
            />
            <span class="subtle">{t('ssh-password-hint')}</span>
          </label>
        {/if}
        {@render problem(failure)}
        <div class="actions">
          {#if step === 'password'}
            <button type="button" class="button" onclick={cancel}>{t('signin-cancel')}</button>
          {/if}
          <button class="button primary" disabled={busy || service === null}>
            {#if busy}
              {t('signin-busy')}
            {:else if service && !service.enabled}
              {t('ssh-setup-enable-and-link')}
            {:else}
              {t('ssh-setup-link')}
            {/if}
          </button>
        </div>
      </form>
    {/if}
  </section>

  {#if linked}
    <section class="card">
      <div class="title">
        <h2>{t('ssh-diag-title')}</h2>
        <button
          class="button icon-button"
          disabled={demo || loadingReport}
          onclick={loadReport}
          title={t('ssh-diag-refresh')}
          aria-label={t('ssh-diag-refresh')}
        >
          <RefreshCw size={15} aria-hidden="true" />
        </button>
      </div>
      {@render problem(reportFailure)}
      {#if report}
        <dl>
          <dt>{t('ssh-diag-dsm')}</dt>
          <dd>{dsmText(report)}</dd>
          <dt>{t('ssh-diag-kernel')}</dt>
          <dd>{report.kernel} ({report.machine})</dd>
          <dt>io_uring</dt>
          <dd>
            {#if report.ioUring === 'available'}
              <span class="badge success">{t('ssh-diag-iouring-available')}</span>
            {:else if report.ioUring === 'disabled'}
              <span class="badge warning">{t('ssh-diag-iouring-disabled')}</span>
            {:else}
              <span class="badge warning">{t('ssh-diag-iouring-missing')}</span>
              <span class="subtle block">{t('ssh-diag-iouring-hint')}</span>
            {/if}
          </dd>
          <dt>{t('ssh-diag-network')}</dt>
          <dd>
            {#each report.network as nic (nic.interface)}
              <span class="block">
                {nic.interface}: <code>{nic.driver}</code>
                <span class="subtle">· {speedText(nic.speed)}</span>
              </span>
            {:else}
              <span class="subtle">{t('ssh-diag-unknown')}</span>
            {/each}
          </dd>
          <dt>Docker</dt>
          <dd>{report.docker ?? t('ssh-diag-docker-none')}</dd>
          <dt>{t('ssh-diag-memory')}</dt>
          <dd>
            {#if memory}
              {#each memory as module (module.slot)}
                <span class="block">
                  {module.slot}:
                  {module.sizeMb ? bytes(module.sizeMb * 1024 * 1024, 'binary') : '?'}
                  {module.kind ?? ''}
                  {#if module.speed}<span class="subtle">· {module.speed} MT/s</span>{/if}
                  {#if module.manufacturer || module.partNumber}
                    <span class="subtle block">
                      {[module.manufacturer, module.partNumber].filter(Boolean).join(' ')}
                    </span>
                  {/if}
                </span>
              {:else}
                <span class="subtle">{t('ssh-diag-unknown')}</span>
              {/each}
            {:else if askMemoryPassword}
              <form
                class="inline"
                onsubmit={(event) => {
                  event.preventDefault();
                  void readMemory();
                }}
              >
                <span class="subtle">{t('ssh-diag-memory-password')}</span>
                <input class="input" type="password" bind:value={memoryPassword} required />
                <button class="button" disabled={readingMemory}>
                  {readingMemory ? t('signin-busy') : t('ssh-diag-memory-read')}
                </button>
              </form>
            {:else}
              <button class="button" disabled={readingMemory} onclick={readMemory}>
                {readingMemory ? t('signin-busy') : t('ssh-diag-memory-read')}
              </button>
              <span class="subtle block">{t('ssh-diag-memory-hint')}</span>
            {/if}
            {@render problem(memoryFailure)}
          </dd>
        </dl>
      {:else if loadingReport}
        <p class="muted">{t('overview-loading')}</p>
      {/if}
    </section>

    {#if settings.expert}
      {#if terminal}
        <!-- xterm.js is large; it loads only when a terminal opens. -->
        {#await import('./Terminal.svelte') then { default: Terminal }}
          <Terminal onclose={() => (terminal = false)} />
        {/await}
      {:else}
        <section class="card status">
          <span class="icon"><SquareTerminal size={20} aria-hidden="true" /></span>
          <div class="grow">
            <strong>{t('ssh-terminal-title')}</strong>
            <span class="muted block">{t('ssh-terminal-text')}</span>
          </div>
          <button class="button" disabled={demo} onclick={() => (terminal = true)}>
            {t('ssh-terminal-open')}
          </button>
        </section>
      {/if}
    {/if}
    {#if demo}<p class="subtle">{t('ssh-demo')}</p>{/if}
  {/if}
</div>

<style>
  .ssh {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 820px;
    margin: 0 auto;
  }

  header,
  .setup,
  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  header {
    gap: 6px;
  }

  .status {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  .icon {
    display: grid;
    flex: none;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 10px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  .icon.danger {
    background: color-mix(in srgb, var(--danger) 16%, transparent);
    color: var(--danger);
  }

  .steps {
    margin: 0;
    padding-left: 1.4em;
    color: var(--text-2);
    line-height: 1.6;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .port input {
    max-width: 8em;
  }

  .title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }

  h2 {
    font-size: 1.05em;
  }

  .icon-button {
    display: grid;
    place-items: center;
    padding: 0;
    width: var(--control-height);
  }

  dl {
    display: grid;
    grid-template-columns: minmax(8em, max-content) 1fr;
    gap: 10px 20px;
    margin: 0;
  }

  dt {
    color: var(--text-2);
  }

  dd {
    margin: 0;
    min-width: 0;
  }

  .inline {
    flex-direction: row;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .inline .input {
    max-width: 14em;
  }

  .fingerprint {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: var(--track);
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
    word-break: break-all;
    user-select: text;
  }

  .warning {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--warning) 16%, transparent);
  }

  .warning :global(svg) {
    flex: none;
    margin-top: 2px;
    color: var(--warning);
  }

  .failure {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    user-select: text;
  }

  .block {
    display: block;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  @media (max-width: 560px) {
    dl {
      grid-template-columns: 1fr;
      gap: 4px;
    }

    dd {
      margin-bottom: 8px;
    }
  }
</style>
