<script lang="ts">
  import Check from '@lucide/svelte/icons/check';
  import Sparkles from '@lucide/svelte/icons/sparkles';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import X from '@lucide/svelte/icons/x';
  import type { SignInTarget } from './SignIn.svelte';
  import {
    beginSetup,
    completeSetup,
    toFailure,
    type Failure,
    type KnownDevice,
    type SetupInfo,
    type TimeZone,
    type UpdatePolicy,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';

  let {
    target,
    trustFingerprint,
    onconnected,
    oncancel,
  }: {
    target: SignInTarget;
    trustFingerprint?: string;
    onconnected: (device: KnownDevice) => void;
    oncancel: () => void;
  } = $props();

  type Step = 'intro' | 'form' | 'confirm' | 'running' | 'skipped';

  let step = $state<Step>('intro');
  let busy = $state(false);
  let failure = $state<Failure | null>(null);
  let info = $state<SetupInfo | null>(null);

  let deviceName = $state('');
  let account = $state('');
  let password = $state('');
  let repeat = $state('');
  let timeZone = $state('');
  let updates = $state<UpdatePolicy>('important');
  let rememberLogin = $state(false);
  let finished = $state<{ device: KnownDevice; skipped: string[] } | null>(null);

  /** DSM's own choice: the zone with the computer's standard-time offset, by city if possible. */
  function suggestZone(zones: TimeZone[]): string {
    const year = new Date().getFullYear();
    const offset =
      -Math.max(
        new Date(year, 0, 1).getTimezoneOffset(),
        new Date(year, 6, 1).getTimezoneOffset(),
      ) * 60;
    const city = Intl.DateTimeFormat().resolvedOptions().timeZone.split('/').pop() ?? '';
    const matching = zones.filter((zone) => zone.offset === offset);
    return (
      matching.find((zone) => city && zone.display.includes(city.replace('_', ' ')))?.value ??
      matching[0]?.value ??
      zones[0]?.value ??
      ''
    );
  }

  async function start() {
    busy = true;
    failure = null;
    try {
      info = await beginSetup({ host: target.host, port: target.port, trustFingerprint });
      deviceName = info.deviceName || target.name || '';
      timeZone = suggestZone(info.timeZones);
      step = 'form';
    } catch (error) {
      failure = toFailure(error);
    } finally {
      busy = false;
    }
  }

  const rules = $derived(info?.passwordRules);
  const checks = $derived.by(() => {
    const list: { label: string; ok: boolean }[] = [];
    if (!rules) return list;
    if (rules.minLength)
      list.push({
        label: t('setup-rule-length', { count: rules.minLength }),
        ok: password.length >= rules.minLength,
      });
    if (rules.mixedCase)
      list.push({
        label: t('setup-rule-case'),
        ok: /[a-z]/.test(password) && /[A-Z]/.test(password),
      });
    if (rules.numeric) list.push({ label: t('setup-rule-number'), ok: /\d/.test(password) });
    if (rules.special)
      list.push({ label: t('setup-rule-special'), ok: /[^A-Za-z0-9]/.test(password) });
    if (rules.excludeUsername)
      list.push({
        label: t('setup-rule-name'),
        ok: !account.trim() || !password.toLowerCase().includes(account.trim().toLowerCase()),
      });
    list.push({ label: t('setup-rule-repeat'), ok: password !== '' && password === repeat });
    return list;
  });

  const accountProblem = $derived.by(() => {
    const name = account.trim().toLowerCase();
    if (!name) return null;
    if (name === 'admin' || info?.reservedNames.includes(name)) return t('setup-account-reserved');
    return null;
  });

  const formValid = $derived(
    deviceName.trim() !== '' &&
      account.trim() !== '' &&
      accountProblem === null &&
      timeZone !== '' &&
      checks.every((check) => check.ok),
  );

  const zoneLabel = $derived(
    info?.timeZones.find((zone) => zone.value === timeZone)?.display ?? timeZone,
  );

  async function run() {
    step = 'running';
    failure = null;
    try {
      finished = await completeSetup({
        host: target.host,
        port: target.port,
        model: target.model,
        deviceName: deviceName.trim(),
        account: account.trim(),
        password,
        timeZone,
        updates,
        rememberLogin,
      });
      password = '';
      repeat = '';
      if (finished.skipped.length === 0) onconnected(finished.device);
      else step = 'skipped';
    } catch (error) {
      failure = toFailure(error);
      // DSM's setup session may be gone after a failed step; the form starts over.
      step = 'intro';
    }
  }
</script>

<section class="card setup" aria-busy={busy || step === 'running'}>
  <header>
    <div class="icon"><Sparkles size={22} aria-hidden="true" /></div>
    <div>
      <h2>{t('setup-title', { name: target.name ?? target.host })}</h2>
      <p class="subtle">{t('signin-address', { host: target.host, port: String(target.port) })}</p>
    </div>
  </header>

  {#if step === 'intro'}
    <p class="warning">
      <TriangleAlert size={18} aria-hidden="true" />
      <span>{t('setup-open-warning')}</span>
    </p>
    <p class="muted">{t('setup-intro')}</p>
    {@render problem()}
    <div class="actions">
      <button class="button" onclick={oncancel}>{t('signin-cancel')}</button>
      <button class="button primary" onclick={start} disabled={busy}>
        {busy ? t('signin-busy') : t('setup-start')}
      </button>
    </div>
  {:else if step === 'form' && info}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        if (formValid) step = 'confirm';
      }}
    >
      <label class="field">
        <span>{t('setup-device-name')}</span>
        <input class="input" bind:value={deviceName} spellcheck="false" maxlength="15" required />
        <span class="subtle">{t('setup-device-name-hint')}</span>
      </label>

      <fieldset>
        <legend>{t('setup-account-title')}</legend>
        <p class="subtle">{t('setup-account-hint')}</p>
        <label class="field">
          <span>{t('signin-account')}</span>
          <input
            class="input"
            bind:value={account}
            autocomplete="username"
            spellcheck="false"
            aria-invalid={accountProblem !== null}
            required
          />
          {#if accountProblem}<span class="invalid">{accountProblem}</span>{/if}
        </label>
        <label class="field">
          <span>{t('signin-password')}</span>
          <input
            class="input"
            type="password"
            bind:value={password}
            autocomplete="new-password"
            required
          />
        </label>
        <label class="field">
          <span>{t('setup-password-repeat')}</span>
          <input
            class="input"
            type="password"
            bind:value={repeat}
            autocomplete="new-password"
            required
          />
        </label>
        <ul class="checks">
          {#each checks as check (check.label)}
            <li class:ok={check.ok}>
              {#if check.ok}<Check size={14} aria-hidden="true" />{:else}<X
                  size={14}
                  aria-hidden="true"
                />{/if}
              {check.label}
            </li>
          {/each}
        </ul>
      </fieldset>

      <label class="field">
        <span>{t('setup-time-zone')}</span>
        <select class="select" bind:value={timeZone}>
          {#each info.timeZones as zone (zone.value)}
            <option value={zone.value}>{zone.display}</option>
          {/each}
        </select>
      </label>

      <fieldset>
        <legend>{t('setup-updates')}</legend>
        {#each ['important', 'all', 'notifyOnly'] as const as policy (policy)}
          <label class="check">
            <input type="radio" name="updates" value={policy} bind:group={updates} />
            {t(`setup-updates-${policy}`)}
          </label>
        {/each}
      </fieldset>

      <label class="check">
        <input type="checkbox" bind:checked={rememberLogin} />
        <span>
          {t('signin-remember-login')}
          <span class="subtle block">{t('signin-remember-login-hint')}</span>
        </span>
      </label>

      <div class="actions">
        <button type="button" class="button" onclick={oncancel}>{t('signin-cancel')}</button>
        <button class="button primary" disabled={!formValid}>{t('setup-review')}</button>
      </div>
    </form>
  {:else if step === 'confirm'}
    <h3>{t('setup-confirm-title')}</h3>
    <ul class="summary">
      <li>{t('setup-do-name', { name: deviceName.trim() })}</li>
      <li>{t('setup-do-account', { account: account.trim() })}</li>
      <li>{t('setup-do-admin')}</li>
      <li>{t('setup-do-zone', { zone: zoneLabel })}</li>
      <li>{t(`setup-do-updates-${updates}`)}</li>
      <li>{t('setup-do-privacy')}</li>
      <li>{t('setup-do-autoblock')}</li>
    </ul>
    <p class="subtle">{t('setup-confirm-hint')}</p>
    <div class="actions">
      <button class="button" onclick={() => (step = 'form')}>{t('signin-cancel')}</button>
      <button class="button primary" onclick={run}>{t('setup-run')}</button>
    </div>
  {:else if step === 'running'}
    <p class="muted running">{t('setup-running')}</p>
  {:else if step === 'skipped' && finished}
    <h3>{t('setup-done')}</h3>
    <p class="muted">{t('setup-skipped')}</p>
    <ul class="summary">
      {#each finished.skipped as api (api)}<li><code>{api}</code></li>{/each}
    </ul>
    <div class="actions">
      <button class="button primary" onclick={() => finished && onconnected(finished.device)}>
        {t('setup-continue')}
      </button>
    </div>
  {/if}
</section>

{#snippet problem()}
  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}
{/snippet}

<style>
  .setup,
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

  h3 {
    font-size: 1em;
  }

  .warning {
    display: flex;
    gap: 10px;
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

  fieldset {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 12px 14px;
    border: 1px solid var(--card-border);
    border-radius: var(--radius-control);
  }

  legend {
    padding: 0 4px;
    font-weight: 600;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
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

  .checks,
  .summary {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .checks li {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-3);
    font-size: 0.92em;
  }

  .checks li.ok {
    color: var(--success);
  }

  .summary li {
    padding-left: 16px;
    text-indent: -12px;
  }

  .summary li::before {
    content: '•';
    display: inline-block;
    width: 12px;
    text-indent: 0;
    color: var(--accent);
  }

  .invalid {
    color: var(--danger);
    font-size: 0.92em;
  }

  .block {
    display: block;
  }

  .failure {
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
    user-select: text;
  }

  .running {
    padding: 24px 0;
    text-align: center;
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
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
