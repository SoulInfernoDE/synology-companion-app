<script lang="ts">
  import BellOff from '@lucide/svelte/icons/bell-off';
  import Power from '@lucide/svelte/icons/power';
  import RotateCw from '@lucide/svelte/icons/rotate-cw';
  import { power, stopBeep, toFailure, type Failure, type PowerAction } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';

  /** Called after DSM accepted a restart or shutdown; the session is gone then. */
  let { ondone }: { ondone: (action: PowerAction) => void } = $props();

  let dialog: HTMLDialogElement;
  let asking = $state<PowerAction | null>(null);
  let busy = $state(false);
  let failure = $state<Failure | null>(null);
  let note = $state<string | null>(null);

  function ask(action: PowerAction) {
    asking = action;
    failure = null;
    dialog.showModal();
  }

  async function confirm() {
    if (!asking) return;
    busy = true;
    failure = null;
    try {
      await power(asking);
      dialog.close();
      ondone(asking);
    } catch (error) {
      failure = toFailure(error);
    } finally {
      busy = false;
    }
  }

  async function silence() {
    note = null;
    try {
      await stopBeep();
      note = t('power-beep-stopped');
    } catch (error) {
      note = t(`error-${toFailure(error).key}`);
    }
  }
</script>

<div class="actions">
  <button
    class="icon-button"
    title={t('power-beep')}
    aria-label={t('power-beep')}
    onclick={silence}
  >
    <BellOff size={16} aria-hidden="true" />
  </button>
  <button
    class="icon-button"
    title={t('power-reboot')}
    aria-label={t('power-reboot')}
    onclick={() => ask('reboot')}
  >
    <RotateCw size={16} aria-hidden="true" />
  </button>
  <button
    class="icon-button"
    title={t('power-shutdown')}
    aria-label={t('power-shutdown')}
    onclick={() => ask('shutdown')}
  >
    <Power size={16} aria-hidden="true" />
  </button>
  {#if note}<span class="subtle note" role="status">{note}</span>{/if}
</div>

<dialog bind:this={dialog} onclose={() => (asking = null)} aria-labelledby="power-title">
  {#if asking}
    <h2 id="power-title">{t(`power-${asking}-title`)}</h2>
    <p>{t(`power-${asking}-text`)}</p>
    {#if failure}
      <p class="failure" role="alert">
        {t(`error-${failure.key}`)}
        {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
      </p>
    {/if}
    <div class="buttons">
      <button class="button" onclick={() => dialog.close()}>{t('expert-dialog-cancel')}</button>
      <button class="button danger" onclick={confirm} disabled={busy}>
        {busy ? t('signin-busy') : t(`power-${asking}`)}
      </button>
    </div>
  {/if}
</dialog>

<style>
  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .icon-button {
    display: grid;
    place-items: center;
    width: var(--control-height);
    height: var(--control-height);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--hover);
    color: var(--text);
  }

  .note {
    margin-left: 6px;
  }

  dialog {
    max-width: min(440px, calc(100vw - 32px));
    padding: 24px;
    border: none;
    border-radius: var(--radius-card);
    background: var(--card);
    color: var(--text);
    box-shadow:
      0 0 0 1px var(--card-border),
      0 20px 50px rgb(0 0 0 / 0.3);
  }

  dialog::backdrop {
    background: rgb(0 0 0 / 0.35);
  }

  dialog[open] {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .failure {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
  }

  .block {
    display: block;
  }

  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }

  .button:disabled {
    opacity: 0.6;
  }
</style>
