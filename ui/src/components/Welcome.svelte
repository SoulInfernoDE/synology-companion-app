<script lang="ts">
  import { onMount } from 'svelte';
  import Info from '@lucide/svelte/icons/info';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Server from '@lucide/svelte/icons/server';
  import Power from '@lucide/svelte/icons/power';
  import Trash from '@lucide/svelte/icons/trash-2';
  import icon from '../../../assets/branding/icon.svg';
  import SignIn, { type SignInTarget } from './SignIn.svelte';
  import {
    discover,
    forgetDevice,
    wake,
    hasCore,
    knownDevices,
    parseAddress,
    toFailure,
    type Failure,
    type FoundNas,
    type KnownDevice,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import { demoDevices } from '../lib/nas';

  let {
    onconnected,
    ondemo,
    notice = null,
    onback = undefined,
  }: {
    onconnected: (device: KnownDevice) => void;
    ondemo: () => void;
    /** A reason the previous connection ended, shown once. */
    notice?: Failure | null;
    /** Back to the signed-in NAS, when the screen was opened to add another one. */
    onback?: () => void;
  } = $props();

  let saved = $state<KnownDevice[]>([]);
  let found = $state<FoundNas[]>([]);
  let searching = $state(false);
  let searchFailure = $state<Failure | null>(null);
  let address = $state('');
  let addressInvalid = $state(false);
  let target = $state<SignInTarget | null>(null);

  const deviceId = (host: string, port: number) => `${host.trim().toLowerCase()}:${port}`;
  const unsaved = $derived(
    found.filter((nas) => !saved.some((device) => device.id === deviceId(nas.host, nas.port))),
  );

  async function search() {
    searching = true;
    searchFailure = null;
    try {
      found = await discover();
    } catch (error) {
      searchFailure = toFailure(error);
    } finally {
      searching = false;
    }
  }

  // Removal is confirmed inline; system dialogs do not work in every WebView.
  let forgetting = $state<string | null>(null);
  /** Feedback after a Wake-on-LAN packet, per device. */
  let woken = $state<Record<string, string>>({});

  async function wakeDevice(device: KnownDevice) {
    try {
      await wake(device.id);
      woken[device.id] = t('welcome-wake-sent');
    } catch (error) {
      woken[device.id] = t(`error-${toFailure(error).key}`);
    }
  }

  async function forget(device: KnownDevice) {
    forgetting = null;
    try {
      await forgetDevice(device.id);
    } finally {
      saved = await knownDevices();
    }
  }

  function connectAddress(event: SubmitEvent) {
    event.preventDefault();
    const parsed = parseAddress(address);
    addressInvalid = parsed === null;
    if (parsed) target = parsed;
  }

  onMount(async () => {
    if (!hasCore()) return;
    saved = await knownDevices().catch(() => []);
    await search();
  });
</script>

<div class="welcome">
  <header>
    <img src={icon} alt="" width="88" height="88" />
    <h1>{t('welcome-title')}</h1>
    <p class="muted">{t('welcome-subtitle')}</p>
  </header>

  {#if onback}
    <button class="button back" onclick={onback}>{t('welcome-back')}</button>
  {/if}

  {#if notice}
    <p class="notice-bar" role="status">{t(`error-${notice.key}`)}</p>
  {/if}

  {#if target}
    <SignIn {target} {onconnected} oncancel={() => (target = null)} />
  {:else if hasCore()}
    {#if saved.length > 0}
      <section class="card">
        <h2>{t('welcome-saved')}</h2>
        <ul>
          {#each saved as device (device.id)}
            {#if forgetting === device.id}
              <li class="confirm">
                <span class="device">{t('welcome-forget-confirm', { name: device.name })}</span>
                <button class="button" onclick={() => (forgetting = null)}
                  >{t('signin-cancel')}</button
                >
                <button class="button danger" onclick={() => forget(device)}>
                  {t('welcome-forget')}
                </button>
              </li>
            {:else}
              <li>
                <Server size={22} strokeWidth={1.6} aria-hidden="true" />
                <div class="device">
                  <strong>{device.name}</strong>
                  <span class="subtle">
                    {[device.model, `${device.host}:${device.port}`, device.account]
                      .filter(Boolean)
                      .join(' · ')}
                  </span>
                  {#if woken[device.id]}
                    <span class="subtle" role="status">{woken[device.id]}</span>
                  {/if}
                </div>
                {#if device.mac}
                  <button
                    class="icon-button wake"
                    title={t('welcome-wake')}
                    aria-label={t('welcome-wake')}
                    onclick={() => wakeDevice(device)}
                  >
                    <Power size={16} aria-hidden="true" />
                  </button>
                {/if}
                <button
                  class="icon-button"
                  title={t('welcome-forget')}
                  aria-label={t('welcome-forget')}
                  onclick={() => (forgetting = device.id)}
                >
                  <Trash size={16} aria-hidden="true" />
                </button>
                <button class="button primary" onclick={() => (target = { ...device })}>
                  {t('welcome-connect')}
                </button>
              </li>
            {/if}
          {/each}
        </ul>
      </section>
    {/if}

    <section class="card">
      <div class="title-row">
        <h2>{t('welcome-found')}</h2>
        <button class="button small" onclick={search} disabled={searching}>
          <span class:spin={searching}><RefreshCw size={14} aria-hidden="true" /></span>
          {searching ? t('welcome-searching') : t('welcome-search-again')}
        </button>
      </div>
      {#if unsaved.length > 0}
        <ul>
          {#each unsaved as nas (nas.host)}
            <li>
              <Server size={22} strokeWidth={1.6} aria-hidden="true" />
              <div class="device">
                <strong>{nas.name}</strong>
                <span class="subtle">
                  {[nas.model, nas.host, nas.dsm ? `DSM ${nas.dsm}` : null]
                    .filter(Boolean)
                    .join(' · ')}
                </span>
              </div>
              <button
                class="button"
                onclick={() =>
                  (target = {
                    host: nas.host,
                    port: nas.port,
                    name: nas.name,
                    model: nas.model,
                    mac: nas.mac,
                  })}
              >
                {t('welcome-connect')}
              </button>
            </li>
          {/each}
        </ul>
      {:else if searchFailure}
        <p class="subtle">{t(`error-${searchFailure.key}`)}</p>
      {:else if !searching && found.length === 0}
        <p class="subtle">{t('welcome-none-found')}</p>
      {/if}

      <form class="manual" onsubmit={connectAddress}>
        <label for="address">{t('welcome-manual')}</label>
        <div class="row">
          <input
            id="address"
            class="input"
            bind:value={address}
            placeholder={t('welcome-manual-placeholder')}
            autocomplete="off"
            spellcheck="false"
            aria-invalid={addressInvalid}
          />
          <button class="button" disabled={!address.trim()}>{t('welcome-connect')}</button>
        </div>
        {#if addressInvalid}<span class="invalid">{t('welcome-manual-invalid')}</span>{/if}
      </form>
    </section>
  {:else}
    <section class="card">
      <h2>{t('welcome-found')}</h2>
      <ul>
        {#each demoDevices as device (device.address)}
          <li>
            <Server size={22} strokeWidth={1.6} aria-hidden="true" />
            <div class="device">
              <strong>{device.name}</strong>
              <span class="subtle">{device.model} · {device.address} · DSM {device.dsm}</span>
            </div>
            <span class="badge">{t('welcome-sample')}</span>
            <button class="button" onclick={ondemo}>{t('welcome-connect')}</button>
          </li>
        {/each}
      </ul>
      <p class="notice subtle">
        <Info size={16} aria-hidden="true" />
        {t('welcome-browser-only')}
      </p>
    </section>
  {/if}

  {#if !target}
    <div class="demo">
      <button class="button" onclick={ondemo}>{t('welcome-demo')}</button>
      <span class="subtle">{t('welcome-demo-hint')}</span>
    </div>
  {/if}

  <footer class="subtle">{t('unofficial')}</footer>
</div>

<style>
  .welcome {
    display: flex;
    flex-direction: column;
    gap: 22px;
    max-width: 620px;
    margin: 0 auto;
    padding: 48px 16px 32px;
    min-height: 100%;
  }

  header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
  }

  header img {
    margin-bottom: 6px;
    filter: drop-shadow(0 6px 16px rgb(0 0 0 / 0.18));
  }

  .back {
    align-self: flex-start;
  }

  .notice-bar {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--warning) 16%, transparent);
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .button.small {
    min-height: calc(var(--control-height) - 6px);
    padding: 0 10px;
    font-size: 0.9em;
  }

  .spin {
    display: inline-flex;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
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
    padding: 10px 4px;
    color: var(--text-2);
  }

  li + li {
    border-top: 1px solid var(--card-border);
  }

  .confirm .device {
    display: block;
  }

  .device {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
    color: var(--text);
    line-height: 1.3;
  }

  .icon-button {
    display: grid;
    place-items: center;
    width: var(--control-height);
    height: var(--control-height);
    border: none;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
  }

  .icon-button:hover {
    background: var(--hover);
    color: var(--danger);
  }

  .icon-button.wake:hover {
    color: var(--success);
  }

  .manual {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row {
    display: flex;
    gap: 8px;
  }

  .row input {
    flex: 1;
    min-width: 0;
  }

  .invalid {
    color: var(--danger);
    font-size: 0.92em;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .notice {
    display: flex;
    gap: 8px;
    align-items: flex-start;
  }

  .notice :global(svg) {
    flex: none;
    margin-top: 2px;
  }

  .demo {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  footer {
    margin-top: auto;
    text-align: center;
  }

  @media (max-width: 480px) {
    li {
      flex-wrap: wrap;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .spin {
      animation: none;
    }
  }
</style>
