<script lang="ts">
  import Box from '@lucide/svelte/icons/box';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import {
    containers as fetchContainers,
    toFailure,
    updateProject,
    type Container,
    type Failure,
    type KnownDevice,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import { demoContainers } from '../lib/nas';

  let {
    device,
    demo,
    onssh,
  }: {
    device: KnownDevice | null;
    demo: boolean;
    onssh: () => void;
  } = $props();

  let list = $state<Container[] | null>(null);
  let failure = $state<Failure | null>(null);
  let busy = $state<string | null>(null);
  let checked = $state(false);
  let done = $state<string | null>(null);
  /** What runs once the sudo password is typed. */
  let waiting = $state<(() => Promise<void>) | null>(null);
  let password = $state('');

  const linked = $derived(demo || device?.ssh !== undefined);
  const projects = $derived.by(() => {
    const groups = new Map<string, Container[]>();
    for (const container of list ?? []) {
      if (!container.project) continue;
      groups.set(container.project, [...(groups.get(container.project) ?? []), container]);
    }
    return [...groups.entries()];
  });
  const single = $derived((list ?? []).filter((container) => !container.project));

  /** Runs `job`, asking for the sudo password when the keychain has none. */
  async function withPassword(job: () => Promise<void>, label: string) {
    busy = label;
    failure = null;
    try {
      await job();
      waiting = null;
    } catch (error) {
      const reason = toFailure(error);
      if (reason.key === 'password-required') {
        waiting = job;
      } else {
        if (reason.key === 'sudo') {
          password = '';
          waiting = job;
        }
        failure = reason;
      }
    } finally {
      busy = null;
    }
  }

  const secret = () => (password === '' ? undefined : password);

  function load(pull: boolean) {
    if (demo) {
      list = demoContainers;
      checked = pull;
      return;
    }
    void withPassword(
      async () => {
        list = await fetchContainers(pull, secret());
        if (pull) checked = true;
      },
      pull ? 'check' : 'load',
    );
  }

  $effect(() => {
    if (linked) load(false);
  });

  function update(project: string, dir: string) {
    done = null;
    void withPassword(async () => {
      await updateProject(project, dir, secret());
      done = project;
      list = await fetchContainers(false, secret());
    }, project);
  }
</script>

<div class="containers">
  <header>
    <h1>{t('nav-containers')}</h1>
    <p class="muted">{t('containers-intro')}</p>
  </header>

  {#if !linked}
    <p class="hint">
      <span>{t('containers-needs-ssh')}</span>
      <button class="button" onclick={onssh}>{t('packages-set-up-ssh')}</button>
    </p>
  {:else}
    <div class="actions">
      <span class="subtle grow">{t('containers-check-hint')}</span>
      <button class="button" disabled={busy !== null} onclick={() => load(true)}>
        <RefreshCw size={15} aria-hidden="true" />
        {busy === 'check' ? t('containers-checking') : t('containers-check')}
      </button>
    </div>
  {/if}

  {#if waiting}
    <form
      class="card password"
      onsubmit={(event) => {
        event.preventDefault();
        const job = waiting;
        if (job) void withPassword(job, 'password');
      }}
    >
      <strong>{t('containers-password')}</strong>
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
        <button class="button primary" disabled={busy !== null}>{t('containers-continue')}</button>
      </div>
    </form>
  {/if}

  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}
  {#if done}
    <p class="done" role="status">{t('containers-updated', { project: done })}</p>
  {/if}

  {#snippet row(container: Container)}
    <li>
      <Box size={18} strokeWidth={1.7} aria-hidden="true" />
      <div class="info">
        <strong>{container.name}</strong>
        <code class="subtle">{container.image}</code>
      </div>
      {#if container.update}
        <span class="badge warning">{t('containers-update-available')}</span>
      {/if}
      <span class="badge" class:success={container.state === 'running'}>
        {t(container.state === 'running' ? 'containers-running' : 'containers-stopped')}
      </span>
    </li>
  {/snippet}

  {#each projects as [project, members] (project)}
    {@const dir = members[0]?.projectDir}
    <section class="card">
      <div class="head">
        <h2 class="grow">{project}</h2>
        {#if dir && members.some((member) => member.update)}
          <button
            class="button primary"
            disabled={demo || busy !== null}
            onclick={() => update(project, dir)}
          >
            {busy === project ? t('containers-updating') : t('containers-update')}
          </button>
        {/if}
      </div>
      {#if dir}<code class="subtle">{dir}</code>{/if}
      <ul>
        {#each members as container (container.name)}
          {@render row(container)}
        {/each}
      </ul>
    </section>
  {/each}

  {#if single.length > 0}
    <section class="card">
      <h2>{t('containers-single')}</h2>
      <ul>
        {#each single as container (container.name)}
          {@render row(container)}
        {/each}
      </ul>
      {#if single.some((container) => container.update)}
        <p class="subtle">{t('containers-single-hint')}</p>
      {/if}
    </section>
  {/if}

  {#if list && list.length === 0}
    <p class="muted">{t('containers-none')}</p>
  {:else if list && checked && !list.some((container) => container.update)}
    <p class="done">{t('containers-current')}</p>
  {:else if linked && list === null && !failure && !waiting}
    <p class="muted">{t('overview-loading')}</p>
  {/if}
  {#if demo}<p class="subtle">{t('containers-demo')}</p>{/if}
</div>

<style>
  .containers {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 820px;
    margin: 0 auto;
  }

  header,
  .password {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .head,
  .actions,
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .row {
    flex-wrap: wrap;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  h2 {
    font-size: 1.05em;
  }

  ul {
    margin: 6px 0 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0;
    color: var(--text-2);
  }

  li + li {
    border-top: 1px solid var(--card-border);
  }

  .info {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    color: var(--text);
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
    word-break: break-all;
    user-select: text;
  }

  .hint {
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

  .button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
