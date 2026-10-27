<script lang="ts">
  import FolderOpen from '@lucide/svelte/icons/folder-open';
  import Lock from '@lucide/svelte/icons/lock';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import {
    openShare,
    shares as fetchShares,
    toFailure,
    type Failure,
    type Shares,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import { demoShares } from '../lib/nas';

  let { demo }: { demo: boolean } = $props();

  let data = $state<Shares | null>(null);
  let failure = $state<Failure | null>(null);
  let opening = $state<string | null>(null);

  $effect(() => {
    if (demo) {
      data = demoShares;
      return;
    }
    fetchShares()
      .then((result) => (data = result))
      .catch((error) => (failure = toFailure(error)));
  });

  async function open(name: string) {
    opening = name;
    failure = null;
    try {
      await openShare(name);
    } catch (error) {
      failure = toFailure(error);
    } finally {
      opening = null;
    }
  }
</script>

<div class="shares">
  <header>
    <h1>{t('nav-files')}</h1>
    <p class="muted">{t('shares-intro')}</p>
  </header>

  {#if data?.smbEnabled === false}
    <p class="warning">
      <TriangleAlert size={16} aria-hidden="true" />
      <span>{t('shares-smb-off')}</span>
    </p>
  {/if}

  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}

  {#if data}
    <section class="card">
      {#if data.shares.length === 0}
        <p class="subtle">{t('shares-none')}</p>
      {:else}
        <ul>
          {#each data.shares as share (share.name)}
            <li>
              <FolderOpen size={20} strokeWidth={1.7} aria-hidden="true" />
              <div class="share">
                <strong>
                  {share.name}
                  {#if share.encrypted}
                    <span class="lock" title={t('shares-encrypted')}>
                      <Lock size={13} aria-label={t('shares-encrypted')} />
                    </span>
                  {/if}
                </strong>
                {#if share.description}<span class="subtle">{share.description}</span>{/if}
              </div>
              <button
                class="button"
                disabled={demo || data.smbEnabled === false || opening !== null}
                onclick={() => open(share.smbName)}
              >
                {opening === share.smbName ? t('signin-busy') : t('shares-open')}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </section>
    <p class="subtle">{demo ? t('shares-demo') : t('shares-hint')}</p>
  {:else if !failure}
    <p class="muted">{t('overview-loading')}</p>
  {/if}
</div>

<style>
  .shares {
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

  .share {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    color: var(--text);
    line-height: 1.3;
  }

  .lock {
    display: inline-flex;
    margin-left: 4px;
    color: var(--text-3);
    vertical-align: -1px;
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

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
