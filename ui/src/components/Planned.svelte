<script lang="ts">
  import Lock from '@lucide/svelte/icons/lock';
  import ExpertDialog from './ExpertDialog.svelte';
  import { t } from '../lib/i18n.svelte';
  import type { Section } from '../lib/sections';
  import { settings } from '../lib/settings.svelte';

  let { section }: { section: Section } = $props();

  let askExpert = $state(false);
  const locked = $derived(section.expert && !settings.expert);
</script>

<div class="planned">
  <div class="icon" class:locked>
    {#if locked}
      <Lock size={40} strokeWidth={1.5} aria-hidden="true" />
    {:else}
      <section.icon size={40} strokeWidth={1.5} aria-hidden="true" />
    {/if}
  </div>
  <h1>{t(section.label)}</h1>

  {#if locked}
    <p class="muted">{t('expert-locked')}</p>
    <button class="button primary" onclick={() => (askExpert = true)}>{t('expert-enable')}</button>
  {:else}
    {#if section.description}<p class="muted">{t(section.description)}</p>{/if}
    {#if section.planned}
      <span class="badge">{t('planned-version', { version: section.planned })}</span>
    {/if}
  {/if}
</div>

<ExpertDialog bind:open={askExpert} />

<style>
  .planned {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    max-width: 480px;
    margin: 12vh auto 0;
    text-align: center;
  }

  .icon {
    display: grid;
    place-items: center;
    width: 84px;
    height: 84px;
    border-radius: 24px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  .icon.locked {
    background: color-mix(in srgb, var(--warning) 16%, transparent);
    color: var(--warning);
  }
</style>
