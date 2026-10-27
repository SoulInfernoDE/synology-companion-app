<script lang="ts" module>
  /** The NAS to sign in to: found on the network, saved, or typed in. */
  export interface SignInTarget {
    host: string;
    port: number;
    name?: string;
    model?: string;
    account?: string;
    rememberLogin?: boolean;
    /** MAC address from discovery, kept for Wake-on-LAN. */
    mac?: string | null;
  }
</script>

<script lang="ts">
  import { onMount } from 'svelte';
  import KeyRound from '@lucide/svelte/icons/key-round';
  import ShieldAlert from '@lucide/svelte/icons/shield-alert';
  import ShieldQuestion from '@lucide/svelte/icons/shield-question';
  import Setup from './Setup.svelte';
  import { connect, probe, toFailure, type Failure, type KnownDevice } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';

  let {
    target,
    onconnected,
    oncancel,
  }: {
    target: SignInTarget;
    onconnected: (device: KnownDevice) => void;
    oncancel: () => void;
  } = $props();

  type Step = 'checking' | 'credentials' | 'otp' | 'certificate' | 'setup';

  // The form starts from the target once; later edits belong to the user.
  // svelte-ignore state_referenced_locally
  let account = $state(target.account ?? '');
  let password = $state('');
  // svelte-ignore state_referenced_locally
  let rememberLogin = $state(target.rememberLogin ?? false);
  let otp = $state('');
  let rememberDevice = $state(true);
  let trustFingerprint = $state<string | undefined>();

  let step = $state<Step>('checking');
  /** The certificate question came from the setup check, not from signing in. */
  let certificateFromCheck = false;
  let busy = $state(false);
  let failure = $state<Failure | null>(null);
  let certificate = $state<{ fingerprint: string; changed: boolean } | null>(null);

  const title = $derived(target.name ?? target.host);

  async function send(usePassword: boolean) {
    busy = true;
    failure = null;
    try {
      const outcome = await connect({
        host: target.host,
        port: target.port,
        name: target.name,
        model: target.model,
        account,
        password: usePassword ? password : undefined,
        otp: step === 'otp' ? otp : undefined,
        rememberDevice,
        rememberLogin,
        trustFingerprint,
        mac: target.mac ?? undefined,
      });
      switch (outcome.status) {
        case 'connected':
          onconnected(outcome.device);
          return;
        case 'passwordRequired':
          step = 'credentials';
          break;
        case 'otpRequired':
          step = 'otp';
          break;
        case 'untrustedCertificate':
        case 'certificateChanged':
          certificate = {
            fingerprint: outcome.fingerprint,
            changed: outcome.status === 'certificateChanged',
          };
          step = 'certificate';
          break;
        case 'failed':
          failure = { key: outcome.key, detail: outcome.detail };
          break;
      }
    } catch (error) {
      failure = toFailure(error);
    } finally {
      busy = false;
    }
  }

  /** Before the first sign-in: is DSM's welcome wizard still open? */
  async function check() {
    step = 'checking';
    failure = null;
    try {
      const outcome = await probe({ host: target.host, port: target.port, trustFingerprint });
      switch (outcome.status) {
        case 'configured':
          step = 'credentials';
          break;
        case 'notConfigured':
          step = 'setup';
          break;
        case 'untrustedCertificate':
        case 'certificateChanged':
          certificate = {
            fingerprint: outcome.fingerprint,
            changed: outcome.status === 'certificateChanged',
          };
          certificateFromCheck = true;
          step = 'certificate';
          break;
        case 'failed':
          failure = { key: outcome.key, detail: outcome.detail };
          step = 'credentials';
          break;
      }
    } catch (error) {
      failure = toFailure(error);
      step = 'credentials';
    }
  }

  function trust() {
    trustFingerprint = certificate?.fingerprint;
    certificate = null;
    if (certificateFromCheck) {
      certificateFromCheck = false;
      void check();
      return;
    }
    step = 'credentials';
    void send(password !== '');
  }

  // A saved NAS with a remembered password connects without asking.
  onMount(() => {
    if (target.account && target.rememberLogin) {
      step = 'credentials';
      void send(false);
    } else {
      void check();
    }
  });
</script>

{#if step === 'setup'}
  <Setup
    {target}
    {trustFingerprint}
    {onconnected}
    {oncancel}
    onsignin={(setUpAccount) => {
      account = setUpAccount;
      failure = { key: 'setup-sign-in' };
      step = 'credentials';
    }}
  />
{:else}
  <section class="card signin" aria-busy={busy || step === 'checking'}>
    <header>
      <div class="icon" class:warning={certificate?.changed}>
        {#if step === 'certificate'}
          {#if certificate?.changed}
            <ShieldAlert size={22} aria-hidden="true" />
          {:else}
            <ShieldQuestion size={22} aria-hidden="true" />
          {/if}
        {:else}
          <KeyRound size={22} aria-hidden="true" />
        {/if}
      </div>
      <div>
        <h2>{t('signin-title', { name: title })}</h2>
        <p class="subtle">
          {t('signin-address', { host: target.host, port: String(target.port) })}
        </p>
      </div>
    </header>

    {#if step === 'checking'}
      <p class="muted checking">{t('signin-checking')}</p>
    {:else if step === 'certificate' && certificate}
      <h3>{t(certificate.changed ? 'signin-cert-changed-title' : 'signin-cert-title')}</h3>
      <p class="muted">
        {t(certificate.changed ? 'signin-cert-changed-text' : 'signin-cert-text')}
      </p>
      <div class="fingerprint">
        <span class="subtle">{t('signin-cert-fingerprint')}</span>
        <code>{certificate.fingerprint}</code>
      </div>
      <div class="actions">
        <button class="button" onclick={oncancel}>{t('signin-cancel')}</button>
        <button class="button {certificate.changed ? 'danger' : 'primary'}" onclick={trust}>
          {t(certificate.changed ? 'signin-cert-trust-changed' : 'signin-cert-trust')}
        </button>
      </div>
    {:else if step === 'otp'}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void send(password !== '');
        }}
      >
        <h3>{t('signin-otp-title')}</h3>
        <p class="muted">{t('signin-otp-text')}</p>
        <label class="field">
          <span>{t('signin-otp-code')}</span>
          <input
            class="input code"
            bind:value={otp}
            inputmode="numeric"
            autocomplete="one-time-code"
            maxlength="8"
            required
          />
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={rememberDevice} />
          {t('signin-remember-device')}
        </label>
        {@render problem()}
        <div class="actions">
          <button type="button" class="button" onclick={oncancel}>{t('signin-cancel')}</button>
          <button class="button primary" disabled={busy}>
            {busy ? t('signin-busy') : t('signin-otp-submit')}
          </button>
        </div>
      </form>
    {:else}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void send(true);
        }}
      >
        <label class="field">
          <span>{t('signin-account')}</span>
          <input
            class="input"
            bind:value={account}
            autocomplete="username"
            spellcheck="false"
            required
          />
        </label>
        <label class="field">
          <span>{t('signin-password')}</span>
          <input
            class="input"
            type="password"
            bind:value={password}
            autocomplete="current-password"
            required
          />
        </label>
        <label class="check">
          <input type="checkbox" bind:checked={rememberLogin} />
          <span>
            {t('signin-remember-login')}
            <span class="subtle block">{t('signin-remember-login-hint')}</span>
          </span>
        </label>
        {@render problem()}
        <div class="actions">
          <button type="button" class="button" onclick={oncancel}>{t('signin-cancel')}</button>
          <button class="button primary" disabled={busy}>
            {busy ? t('signin-busy') : t('signin-submit')}
          </button>
        </div>
      </form>
    {/if}
  </section>
{/if}

{#snippet problem()}
  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}
{/snippet}

<style>
  .signin,
  form {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon {
    display: grid;
    flex: none;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 12px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  .icon.warning {
    background: color-mix(in srgb, var(--danger) 16%, transparent);
    color: var(--danger);
  }

  h3 {
    font-size: 1em;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .code {
    max-width: 10em;
    font-size: 1.3em;
    letter-spacing: 0.2em;
    font-variant-numeric: tabular-nums;
  }

  .check {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    cursor: pointer;
  }

  .check input {
    margin-top: 3px;
    accent-color: var(--accent);
  }

  .block {
    display: block;
  }

  .fingerprint {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: var(--track);
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
    word-break: break-all;
    user-select: text;
  }

  .failure {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    user-select: text;
  }

  .checking {
    padding: 12px 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .button:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
