<script lang="ts">
  import Pause from '@lucide/svelte/icons/pause';
  import Play from '@lucide/svelte/icons/play';
  import Trash2 from '@lucide/svelte/icons/trash-2';
  import {
    addDownload,
    downloadAction,
    downloads as fetchDownloads,
    toFailure,
    type Download,
    type DownloadAction,
    type Failure,
  } from '../lib/backend';
  import { bytes, percent, rate } from '../lib/format';
  import { t } from '../lib/i18n.svelte';
  import { demoDownloads } from '../lib/nas';

  let { demo }: { demo: boolean } = $props();

  /** Download Station updates its numbers every few seconds. */
  const REFRESH = 5000;

  let list = $state<Download[] | null>(null);
  let failure = $state<Failure | null>(null);
  let link = $state('');
  let adding = $state(false);
  let deleting = $state<string | null>(null);

  async function load() {
    if (demo) {
      list ??= demoDownloads;
      return;
    }
    try {
      list = await fetchDownloads();
      if (failure?.key !== 'download-link') failure = null;
    } catch (error) {
      failure = toFailure(error);
    }
  }

  $effect(() => {
    void load();
    const timer = setInterval(() => void load(), REFRESH);
    return () => clearInterval(timer);
  });

  async function add() {
    adding = true;
    failure = null;
    try {
      await addDownload(link);
      link = '';
      await load();
    } catch (error) {
      failure = toFailure(error);
    } finally {
      adding = false;
    }
  }

  async function act(task: Download, action: DownloadAction) {
    failure = null;
    deleting = null;
    if (demo) {
      list = (list ?? [])
        .filter((known) => action !== 'delete' || known.id !== task.id)
        .map((known) =>
          known.id === task.id
            ? { ...known, status: action === 'pause' ? 'paused' : 'downloading' }
            : known,
        );
      return;
    }
    try {
      await downloadAction(task.id, action);
      await load();
    } catch (error) {
      failure = toFailure(error);
    }
  }

  const share = (task: Download) => (task.size > 0 ? task.downloaded / task.size : 0);
  const active = (status: string) =>
    ['waiting', 'downloading', 'finishing', 'hash_checking', 'extracting', 'seeding'].includes(
      status,
    );
  const known = new Set([
    'waiting',
    'downloading',
    'paused',
    'finishing',
    'finished',
    'hash_checking',
    'seeding',
    'extracting',
    'error',
  ]);
  const statusText = (status: string) =>
    known.has(status) ? t(`downloads-status-${status}`) : status;
</script>

<div class="downloads">
  <header>
    <h1>{t('nav-downloads')}</h1>
    <p class="muted">{t('downloads-intro')}</p>
  </header>

  {#if failure?.key !== 'downloads-missing'}
    <form
      class="add"
      onsubmit={(event) => {
        event.preventDefault();
        void add();
      }}
    >
      <input
        class="input"
        bind:value={link}
        placeholder={t('downloads-link')}
        aria-label={t('downloads-link')}
        required
      />
      <button class="button primary" disabled={adding || demo}>
        {adding ? t('signin-busy') : t('downloads-add')}
      </button>
    </form>
  {/if}

  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}

  {#if list && list.length > 0}
    <section class="card">
      <ul>
        {#each list as task (task.id)}
          <li>
            <div class="info">
              <strong>{task.title}</strong>
              <div
                class="bar"
                class:done={task.status === 'finished' || task.status === 'seeding'}
                class:error={task.status === 'error'}
              >
                <div class="fill" style:width="{share(task) * 100}%"></div>
              </div>
              <span class="subtle">
                {statusText(task.status)} · {percent(share(task) * 100)} ·
                {bytes(task.downloaded)} / {bytes(task.size)}
                {#if task.speed > 0}· {rate(task.speed)}{/if}
                {#if task.destination}· {task.destination}{/if}
                {#if task.error}· {task.error}{/if}
              </span>
            </div>
            {#if deleting === task.id}
              <button class="button" onclick={() => (deleting = null)}>{t('signin-cancel')}</button>
              <button class="button danger" onclick={() => act(task, 'delete')}>
                {t('downloads-delete')}
              </button>
            {:else}
              {#if active(task.status)}
                <button
                  class="button icon"
                  onclick={() => act(task, 'pause')}
                  title={t('downloads-pause')}
                  aria-label={t('downloads-pause')}
                >
                  <Pause size={15} aria-hidden="true" />
                </button>
              {:else if task.status === 'paused' || task.status === 'error'}
                <button
                  class="button icon"
                  onclick={() => act(task, 'resume')}
                  title={t('downloads-resume')}
                  aria-label={t('downloads-resume')}
                >
                  <Play size={15} aria-hidden="true" />
                </button>
              {/if}
              <button
                class="button icon"
                onclick={() => (deleting = task.id)}
                title={t('downloads-delete')}
                aria-label={t('downloads-delete')}
              >
                <Trash2 size={15} aria-hidden="true" />
              </button>
            {/if}
          </li>
        {/each}
      </ul>
    </section>
    <p class="subtle">{t('downloads-delete-hint')}</p>
  {:else if list}
    <p class="muted">{t('downloads-none')}</p>
  {:else if !failure}
    <p class="muted">{t('overview-loading')}</p>
  {/if}
  {#if demo}<p class="subtle">{t('downloads-demo')}</p>{/if}
</div>

<style>
  .downloads {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 820px;
    margin: 0 auto;
  }

  header {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .add {
    display: flex;
    gap: 8px;
  }

  .add .input {
    flex: 1;
    min-width: 0;
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 0;
  }

  li + li {
    border-top: 1px solid var(--card-border);
  }

  .info {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .info strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .bar {
    height: 6px;
    overflow: hidden;
    border-radius: 3px;
    background: var(--track);
  }

  .fill {
    height: 100%;
    border-radius: 3px;
    background: var(--accent);
    transition: width 0.4s ease;
  }

  .bar.done .fill {
    background: var(--success);
  }

  .bar.error .fill {
    background: var(--danger);
  }

  .icon {
    display: grid;
    place-items: center;
    width: var(--control-height);
    padding: 0;
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

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
