<script lang="ts">
  import CircleCheck from '@lucide/svelte/icons/circle-check';
  import CircleX from '@lucide/svelte/icons/circle-x';
  import Cloud from '@lucide/svelte/icons/cloud';
  import LoaderCircle from '@lucide/svelte/icons/loader-circle';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import {
    nextcloud as fetchEnvironment,
    nextcloudCheck,
    nextcloudStep,
    nextcloudSteps,
    nextcloudWorkFolder,
    phpFits,
    toFailure,
    type Failure,
    type KnownDevice,
    type NextcloudCheck,
    type NextcloudEnvironment,
    type NextcloudInstall,
    type NextcloudStep,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import { demoNextcloud } from '../lib/nas';

  let {
    device,
    demo,
    onssh,
  }: {
    device: KnownDevice | null;
    demo: boolean;
    onssh: () => void;
  } = $props();

  type StepState = 'pending' | 'running' | 'done' | 'failed';

  let environment = $state<NextcloudEnvironment | null>(null);
  let failure = $state<Failure | null>(null);
  let busy = $state(false);
  let waiting = $state<(() => Promise<void>) | null>(null);
  let password = $state('');
  let checks = $state<Record<string, NextcloudCheck>>({});

  /** The update in progress: one installation at a time. */
  let updating = $state<NextcloudInstall | null>(null);
  let confirming = $state<NextcloudInstall | null>(null);
  let work = $state<string | null>(null);
  let states = $state<Record<NextcloudStep, StepState>>(blank());
  let outputs = $state<Partial<Record<NextcloudStep, string>>>({});

  const linked = $derived(demo || device?.ssh !== undefined);

  function blank(): Record<NextcloudStep, StepState> {
    return Object.fromEntries(nextcloudSteps.map((step) => [step, 'pending'])) as Record<
      NextcloudStep,
      StepState
    >;
  }

  const secret = () => (password === '' ? undefined : password);

  /** Runs `job`, asking for the sudo password when the keychain has none. */
  async function withPassword(job: () => Promise<void>) {
    busy = true;
    failure = null;
    try {
      await job();
      waiting = null;
    } catch (error) {
      const reason = toFailure(error);
      if (reason.key === 'password-required') waiting = job;
      else {
        if (reason.key === 'sudo') {
          password = '';
          waiting = job;
        }
        failure = reason;
      }
    } finally {
      busy = false;
    }
  }

  function load() {
    if (demo) {
      environment = demoNextcloud;
      return;
    }
    void withPassword(async () => {
      environment = await fetchEnvironment(secret());
    });
  }

  $effect(() => {
    if (linked) load();
  });

  function check(install: NextcloudInstall) {
    if (demo) {
      checks[install.path] = {
        available: '32.0.1',
        requirements: { major: 32, phpMin: '8.1', phpBelow: '8.5' },
      };
      return;
    }
    void withPassword(async () => {
      checks[install.path] = await nextcloudCheck(install, secret());
    });
  }

  function start(install: NextcloudInstall) {
    confirming = null;
    updating = install;
    states = blank();
    outputs = {};
    work = null;
    void continueUpdate();
  }

  /** Runs the remaining steps in order and stops at the first that fails. */
  async function continueUpdate() {
    const install = updating;
    if (!install) return;
    await withPassword(async () => {
      work ??= await nextcloudWorkFolder();
      for (const step of nextcloudSteps) {
        if (states[step] === 'done') continue;
        states[step] = 'running';
        const run = await nextcloudStep(install, step, work, secret()).catch((error) => {
          states[step] = 'pending';
          throw error;
        });
        outputs[step] = run.output;
        states[step] = run.ok ? 'done' : 'failed';
        if (!run.ok) return;
      }
      load();
    });
  }

  /** After a failed step: leaves maintenance mode and restores the owners. */
  async function leave() {
    const install = updating;
    if (!install || !work) return;
    const folder = work;
    await withPassword(async () => {
      for (const step of ['closePermissions', 'maintenanceOff'] as const) {
        states[step] = 'running';
        const run = await nextcloudStep(install, step, folder, secret());
        outputs[step] = run.output;
        states[step] = run.ok ? 'done' : 'failed';
      }
      load();
    });
  }

  const failed = $derived(nextcloudSteps.find((step) => states[step] === 'failed'));
  const finished = $derived(nextcloudSteps.every((step) => states[step] === 'done'));
  const opened = $derived(states.maintenanceOn === 'done' && states.maintenanceOff !== 'done');
</script>

<div class="nextcloud">
  <header>
    <h1>{t('nav-nextcloud')}</h1>
    <p class="muted">{t('nextcloud-intro')}</p>
  </header>

  {#if !linked}
    <p class="hint">
      <span>{t('nextcloud-needs-ssh')}</span>
      <button class="button" onclick={onssh}>{t('packages-set-up-ssh')}</button>
    </p>
  {/if}

  {#if waiting}
    <form
      class="card password"
      onsubmit={(event) => {
        event.preventDefault();
        const job = waiting;
        if (job) void withPassword(job);
      }}
    >
      <strong>{t('containers-password')}</strong>
      <span class="muted">{t('nextcloud-password-text')}</span>
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
        <button class="button primary" disabled={busy}>{t('containers-continue')}</button>
      </div>
    </form>
  {/if}

  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}

  {#if environment}
    {#each environment.installs as install (install.path)}
      {@const result = checks[install.path]}
      {@const requirements = result?.requirements ?? null}
      {@const fits = requirements ? phpFits(install.phpVersion, requirements) : null}
      {@const better = requirements
        ? environment.php.filter((version) => phpFits(version, requirements))
        : []}
      <section class="card install">
        <div class="head">
          <span class="icon"><Cloud size={20} aria-hidden="true" /></span>
          <div class="grow">
            <h2>Nextcloud {install.version}</h2>
            <code class="subtle">{install.path}</code>
          </div>
          {#if install.maintenance}
            <span class="badge warning">{t('nextcloud-maintenance')}</span>
          {/if}
          {#if install.needsDbUpgrade}
            <span class="badge warning">{t('nextcloud-needs-upgrade')}</span>
          {/if}
        </div>
        <dl>
          <dt>PHP</dt>
          <dd>{install.phpVersion} <code class="subtle">{install.php}</code></dd>
          <dt>{t('nextcloud-database')}</dt>
          <dd>
            {install.database ?? t('ssh-diag-unknown')}
            {#if install.database === 'mysql' && environment.mariadb}· MariaDB {environment.mariadb}{/if}
          </dd>
          <dt>Redis</dt>
          <dd>
            {install.redis ? t('nextcloud-redis-used') : t('nextcloud-redis-unused')}
            {#if environment.redis}· {environment.redis}{/if}
          </dd>
          <dt>{t('nextcloud-runs-as')}</dt>
          <dd><code>{install.owner}</code></dd>
        </dl>

        {#if result}
          <div class="check">
            {#if result.available}
              <strong>{t('nextcloud-available', { version: result.available })}</strong>
            {:else}
              <strong>{t('nextcloud-current')}</strong>
            {/if}
            {#if requirements}
              <span class="line" class:bad={fits === false}>
                {#if fits}
                  <CircleCheck size={16} aria-hidden="true" />
                {:else}
                  <CircleX size={16} aria-hidden="true" />
                {/if}
                {t('nextcloud-php-range', {
                  major: requirements.major,
                  min: requirements.phpMin,
                  below: requirements.phpBelow ?? '–',
                  current: install.phpVersion,
                })}
              </span>
              {#if fits === false}
                <span class="subtle">
                  {better.length > 0
                    ? t('nextcloud-php-switch', { versions: better.join(', ') })
                    : t('nextcloud-php-install', { min: requirements.phpMin })}
                </span>
              {/if}
            {:else}
              <span class="subtle">{t('nextcloud-requirements-unknown')}</span>
            {/if}
          </div>
        {/if}

        {#if updating?.path === install.path}
          <ol class="steps">
            {#each nextcloudSteps as step (step)}
              <li class="state-{states[step]}">
                {#if states[step] === 'done'}
                  <CircleCheck size={16} aria-hidden="true" />
                {:else if states[step] === 'failed'}
                  <CircleX size={16} aria-hidden="true" />
                {:else if states[step] === 'running'}
                  <LoaderCircle size={16} aria-hidden="true" class="spin" />
                {:else}
                  <span class="dot"></span>
                {/if}
                <div class="grow">
                  <span>{t(`nextcloud-step-${step}`)}</span>
                  {#if outputs[step]}
                    <details open={states[step] === 'failed'}>
                      <summary class="subtle">{t('extensions-output')}</summary>
                      <pre>{outputs[step]}</pre>
                    </details>
                  {/if}
                </div>
              </li>
            {/each}
          </ol>
          {#if finished}
            <p class="done">{t('nextcloud-finished')}</p>
          {:else if failed}
            <p class="failure">{t('nextcloud-step-failed')}</p>
            <div class="actions">
              {#if opened}
                <button class="button" disabled={busy} onclick={leave}>
                  {t('nextcloud-leave')}
                </button>
              {/if}
              <button class="button primary" disabled={busy} onclick={continueUpdate}>
                {t('nextcloud-retry')}
              </button>
            </div>
          {/if}
        {:else if confirming?.path === install.path}
          <div class="plan">
            <p class="warning">
              <TriangleAlert size={16} aria-hidden="true" />
              <span>{t('nextcloud-confirm')}</span>
            </p>
            <ol>
              {#each nextcloudSteps as step (step)}
                <li>{t(`nextcloud-step-${step}`)}</li>
              {/each}
            </ol>
            <div class="actions">
              <button class="button" onclick={() => (confirming = null)}>
                {t('signin-cancel')}
              </button>
              <button class="button danger" disabled={demo} onclick={() => start(install)}>
                {t('nextcloud-start')}
              </button>
            </div>
          </div>
        {:else}
          <div class="actions">
            <button class="button" disabled={busy} onclick={() => check(install)}>
              {t('nextcloud-check')}
            </button>
            <button
              class="button primary"
              disabled={busy || !result?.available || fits === false || updating !== null}
              onclick={() => (confirming = install)}
            >
              {t('nextcloud-update')}
            </button>
          </div>
        {/if}
      </section>
    {:else}
      <p class="muted">{t('nextcloud-none')}</p>
    {/each}
    <p class="subtle">
      {t('nextcloud-nas-php', { versions: environment.php.join(', ') || '–' })}
    </p>
  {:else if linked && !failure && !waiting}
    <p class="muted">{t('overview-loading')}</p>
  {/if}
  {#if demo}<p class="subtle">{t('nextcloud-demo')}</p>{/if}
</div>

<style>
  .nextcloud {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 820px;
    margin: 0 auto;
  }

  header,
  .install,
  .password,
  .check,
  .plan {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  header {
    gap: 6px;
  }

  .install {
    gap: 12px;
  }

  .head,
  .row,
  .actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .row {
    flex-wrap: wrap;
  }

  .actions {
    justify-content: flex-end;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  h2 {
    font-size: 1.05em;
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

  dl {
    display: grid;
    grid-template-columns: minmax(8em, max-content) 1fr;
    gap: 6px 20px;
    margin: 0;
  }

  dt {
    color: var(--text-2);
  }

  dd {
    margin: 0;
  }

  .check,
  .plan {
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: var(--track);
  }

  .line {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--success);
  }

  .line.bad {
    color: var(--danger);
  }

  ol {
    margin: 0;
    padding-left: 1.4em;
    line-height: 1.6;
  }

  .steps {
    padding: 0;
    list-style: none;
  }

  .steps li {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 4px 0;
    color: var(--text-2);
  }

  .steps li :global(svg),
  .dot {
    flex: none;
    margin-top: 4px;
  }

  .steps li.state-done {
    color: var(--text);
  }

  .steps li.state-done :global(svg) {
    color: var(--success);
  }

  .steps li.state-failed :global(svg) {
    color: var(--danger);
  }

  .steps li.state-running {
    color: var(--text);
    font-weight: 600;
  }

  .dot {
    width: 16px;
    height: 16px;
    border: 2px solid var(--track);
    border-radius: 50%;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  pre {
    max-height: 260px;
    overflow: auto;
    margin: 6px 0 0;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: var(--track);
    font-size: 0.82em;
    white-space: pre-wrap;
    user-select: text;
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
    word-break: break-all;
    user-select: text;
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
    margin: 0;
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

  @media (max-width: 560px) {
    dl {
      grid-template-columns: 1fr;
    }
  }
</style>
