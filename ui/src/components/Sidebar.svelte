<script lang="ts">
  import Check from '@lucide/svelte/icons/check';
  import ChevronDown from '@lucide/svelte/icons/chevron-down';
  import LogOut from '@lucide/svelte/icons/log-out';
  import Plus from '@lucide/svelte/icons/plus';
  import SettingsIcon from '@lucide/svelte/icons/settings';
  import ShieldAlert from '@lucide/svelte/icons/shield-alert';
  import icon from '../../../assets/branding/icon.svg';
  import { t } from '../lib/i18n.svelte';
  import { sections, type SectionId } from '../lib/sections';

  interface Device {
    id: string;
    name: string;
    model: string;
  }

  let {
    device,
    devices = [],
    section = $bindable(),
    ondisconnect,
    onswitch,
    onadd,
  }: {
    device: Device;
    /** Every signed-in NAS; the switcher appears when there is more than the demo. */
    devices?: Device[];
    section: SectionId | 'settings';
    ondisconnect: () => void;
    onswitch?: (id: string) => void;
    onadd?: () => void;
  } = $props();

  let menuOpen = $state(false);
</script>

<aside class="sidebar">
  <div class="switcher">
    <button
      class="device"
      aria-expanded={menuOpen}
      aria-haspopup="menu"
      title={t('nav-switch')}
      disabled={!onadd}
      onclick={() => (menuOpen = !menuOpen)}
    >
      <img src={icon} alt="" width="36" height="36" />
      <span class="device-text">
        <strong>{device.name}</strong>
        <span class="subtle">{device.model}</span>
      </span>
      {#if onadd}<span class="chevron"><ChevronDown size={16} aria-hidden="true" /></span>{/if}
    </button>
    {#if menuOpen && onadd}
      <div class="menu" role="menu">
        {#each devices as entry (entry.id)}
          <button
            class="menu-item"
            role="menuitemradio"
            aria-checked={entry.id === device.id}
            onclick={() => {
              menuOpen = false;
              if (entry.id !== device.id) onswitch?.(entry.id);
            }}
          >
            <span class="mark"
              >{#if entry.id === device.id}<Check size={14} aria-hidden="true" />{/if}</span
            >
            <span class="menu-text">
              <strong>{entry.name}</strong>
              <span class="subtle">{entry.model}</span>
            </span>
          </button>
        {/each}
        <button
          class="menu-item"
          role="menuitem"
          onclick={() => {
            menuOpen = false;
            onadd();
          }}
        >
          <span class="mark"><Plus size={14} aria-hidden="true" /></span>
          <span class="menu-text">{t('nav-add')}</span>
        </button>
      </div>
    {/if}
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
      <LogOut size={18} strokeWidth={1.8} aria-hidden="true" />
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

  .switcher {
    position: relative;
  }

  .device {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 6px 8px;
    border: none;
    border-radius: var(--nav-radius);
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .device:hover:not(:disabled) {
    background: var(--hover);
  }

  .device:disabled {
    cursor: default;
  }

  .chevron {
    display: flex;
    margin-left: auto;
    color: var(--text-3);
  }

  .menu {
    position: absolute;
    z-index: 10;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    display: flex;
    flex-direction: column;
    padding: 4px;
    border-radius: var(--radius-card);
    background: var(--card);
    box-shadow:
      0 0 0 1px var(--card-border),
      0 12px 30px rgb(0 0 0 / 0.25);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .menu-item:hover {
    background: var(--hover);
  }

  .mark {
    display: flex;
    flex: none;
    width: 16px;
    color: var(--accent);
  }

  .menu-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.25;
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
