<script lang="ts">
  import ExpertDialog from './ExpertDialog.svelte';
  import icon from '../../../assets/branding/icon.svg';
  import { availableLocales, localeName, t } from '../lib/i18n.svelte';
  import {
    settings,
    systemSkin,
    type SchemeChoice,
    type Skin,
    type SkinChoice,
  } from '../lib/settings.svelte';

  // Platform names are proper names and need no translation.
  const skinNames: Record<Skin, string> = { linux: 'Linux', macos: 'macOS', windows: 'Windows' };

  const skins: { value: SkinChoice; label: () => string }[] = [
    { value: 'auto', label: () => t('settings-skin-auto', { skin: skinNames[systemSkin()] }) },
    { value: 'linux', label: () => skinNames.linux },
    { value: 'macos', label: () => skinNames.macos },
    { value: 'windows', label: () => skinNames.windows },
  ];

  const schemes: { value: SchemeChoice; label: string }[] = [
    { value: 'auto', label: 'settings-scheme-auto' },
    { value: 'light', label: 'settings-scheme-light' },
    { value: 'dark', label: 'settings-scheme-dark' },
  ];

  let askExpert = $state(false);

  function toggleExpert(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    if (input.checked) {
      // Only the dialog turns expert mode on.
      input.checked = false;
      askExpert = true;
    } else {
      settings.expert = false;
    }
  }
</script>

<div class="settings">
  <h1>{t('settings-title')}</h1>

  <section class="card">
    <h2>{t('settings-appearance')}</h2>

    <div class="row">
      <span id="skin-label">{t('settings-skin')}</span>
      <div class="segmented" role="radiogroup" aria-labelledby="skin-label">
        {#each skins as skin (skin.value)}
          <label class:active={settings.skin === skin.value}>
            <input type="radio" name="skin" value={skin.value} bind:group={settings.skin} />
            {skin.label()}
          </label>
        {/each}
      </div>
    </div>

    <div class="row">
      <span id="scheme-label">{t('settings-scheme')}</span>
      <div class="segmented" role="radiogroup" aria-labelledby="scheme-label">
        {#each schemes as scheme (scheme.value)}
          <label class:active={settings.scheme === scheme.value}>
            <input type="radio" name="scheme" value={scheme.value} bind:group={settings.scheme} />
            {t(scheme.label)}
          </label>
        {/each}
      </div>
    </div>

    <div class="row">
      <label for="language">{t('settings-language')}</label>
      <select id="language" class="select" bind:value={settings.language}>
        <option value={null}>{t('settings-language-auto')}</option>
        {#each availableLocales as locale (locale)}
          <option value={locale}>{localeName(locale)}</option>
        {/each}
      </select>
    </div>
  </section>

  <section class="card">
    <h2>{t('settings-advanced')}</h2>
    <label class="row switch-row">
      <span>
        {t('settings-expert')}
        <span class="subtle block">{t('settings-expert-hint')}</span>
      </span>
      <input type="checkbox" class="switch" checked={settings.expert} onchange={toggleExpert} />
    </label>
  </section>

  <section class="card about">
    <img src={icon} alt="" width="48" height="48" />
    <div>
      <strong>Syno Companion</strong>
      <p class="subtle">{t('settings-version', { version: __APP_VERSION__ })}</p>
      <p class="subtle">{t('settings-license')}</p>
      <p class="subtle">{t('unofficial')}</p>
    </div>
  </section>
</div>

<ExpertDialog bind:open={askExpert} />

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-width: 720px;
    margin: 0 auto;
  }

  h2 {
    margin-bottom: 6px;
    font-size: 1em;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 0;
  }

  .row + .row {
    border-top: 1px solid var(--card-border);
  }

  .segmented {
    display: inline-flex;
    flex-wrap: wrap;
    padding: 2px;
    border-radius: calc(var(--radius-control) + 2px);
    background: var(--track);
  }

  .segmented label {
    padding: 4px 12px;
    border-radius: var(--radius-control);
    color: var(--text-2);
    cursor: pointer;
  }

  .segmented label.active {
    background: var(--card);
    color: var(--text);
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.15);
  }

  .segmented label:has(input:focus-visible) {
    outline: 2px solid var(--accent);
  }

  .segmented input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .block {
    display: block;
  }

  .switch-row {
    cursor: pointer;
  }

  .switch {
    appearance: none;
    flex: none;
    position: relative;
    width: 42px;
    height: 24px;
    margin: 0;
    border-radius: 999px;
    background: var(--track);
    cursor: pointer;
    transition: background 150ms ease;
  }

  .switch::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #ffffff;
    box-shadow: 0 1px 2px rgb(0 0 0 / 0.3);
    transition: transform 150ms ease;
  }

  .switch:checked {
    background: var(--accent);
  }

  .switch:checked::after {
    transform: translateX(18px);
  }

  .about {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }

  @media (max-width: 640px) {
    .row {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
