<script lang="ts">
  import ShieldAlert from '@lucide/svelte/icons/shield-alert';
  import { t } from '../lib/i18n.svelte';
  import { settings } from '../lib/settings.svelte';

  let { open = $bindable() }: { open: boolean } = $props();

  let dialog: HTMLDialogElement;

  $effect(() => {
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  });

  function confirm() {
    settings.expert = true;
    open = false;
  }
</script>

<dialog bind:this={dialog} onclose={() => (open = false)} aria-labelledby="expert-title">
  <div class="icon"><ShieldAlert size={28} aria-hidden="true" /></div>
  <h2 id="expert-title">{t('expert-dialog-title')}</h2>
  <p>{t('expert-dialog-body')}</p>
  <p class="muted">{t('expert-dialog-test')}</p>
  <div class="actions">
    <button class="button" onclick={() => (open = false)}>{t('expert-dialog-cancel')}</button>
    <button class="button danger" onclick={confirm}>{t('expert-dialog-confirm')}</button>
  </div>
</dialog>

<style>
  dialog {
    max-width: min(460px, calc(100vw - 32px));
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

  .icon {
    color: var(--warning);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }
</style>
