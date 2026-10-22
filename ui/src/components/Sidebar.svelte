<script lang="ts">
  import ArrowLeftRight from '@lucide/svelte/icons/arrow-left-right';
  import SettingsIcon from '@lucide/svelte/icons/settings';
  import ShieldAlert from '@lucide/svelte/icons/shield-alert';
  import icon from '../../../assets/branding/icon.svg';
  import { t } from '../lib/i18n.svelte';
  import { sections, type SectionId } from '../lib/sections';

  let {
    device,
    section = $bindable(),
    ondisconnect,
  }: {
    device: { name: string; model: string };
    section: SectionId | 'settings';
    ondisconnect: () => void;
  } = $props();
</script>

<aside class="sidebar">
  <div class="device">
    <img src={icon} alt="" width="36" height="36" />
    <div class="device-text">
      <strong>{device.name}</strong>
      <span class="subtle">{device.model}</span>
    </div>
  </div>

  <nav aria-label={t('nav-label')}>
    {#each sections as entry (entry.id)}
      <button
        class="nav-item"
        aria-current={section === entry.id ? 'page' : undefined}
        title={t(entry.label)}
        onclick={() => (section = entry.id)}
      >
        <entry.icon size={18} strokeWidth={1.8} aria-hidden="true" />
        <span class="label">{t(entry.label)}</span>
        {#if entry.expert}
          <span class="expert" title={t('nav-expert')}>
            <ShieldAlert size={14} aria-label={t('nav-expert')} />
          </span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="bottom">
    <button
      class="nav-item"
      aria-current={section === 'settings' ? 'page' : undefined}
      title={t('nav-settings')}
      onclick={() => (section = 'settings')}
    >
      <SettingsIcon size={18} strokeWidth={1.8} aria-hidden="true" />
      <span class="label">{t('nav-settings')}</span>
    </button>
    <button class="nav-item" title={t('nav-disconnect')} onclick={ondisconnect}>
      <ArrowLeftRight size={18} strokeWidth={1.8} aria-hidden="true" />
      <span class="label">{t('nav-disconnect')}</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px 10px;
    background: var(--sidebar);
    border-right: 1px solid var(--card-border);
    overflow-y: auto;
  }

  .device {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 8px 8px;
  }

  .device-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.25;
  }

  .device-text strong {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  nav,
  .bottom {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .bottom {
    margin-top: auto;
  }

  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: var(--control-height);
    padding: 0 10px;
    border: none;
    border-radius: var(--nav-radius);
    background: transparent;
    color: var(--text-2);
    text-align: left;
    cursor: pointer;
  }

  .nav-item:hover {
    background: var(--hover);
    color: var(--text);
  }

  .nav-item[aria-current='page'] {
    background: var(--selected);
    color: var(--text);
    font-weight: 600;
  }

  /* Fluent marks the selected item with an accent pill. */
  .nav-item[aria-current='page']::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    width: 3px;
    height: 16px;
    border-radius: 3px;
    background: var(--accent);
    transform: translateY(-50%);
    opacity: var(--nav-indicator);
  }

  :global([data-skin='macos']) .nav-item[aria-current='page'] {
    background: var(--accent);
    color: var(--accent-text);
  }

  .expert {
    display: flex;
    color: var(--warning);
  }

  .nav-item[aria-current='page'] .expert {
    color: inherit;
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (max-width: 720px) {
    .device-text,
    .label,
    .expert {
      display: none;
    }

    .device,
    .nav-item {
      justify-content: center;
      padding: 0;
    }
  }
</style>
