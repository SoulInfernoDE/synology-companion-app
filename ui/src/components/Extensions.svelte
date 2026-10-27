<script lang="ts">
  import CircleCheck from '@lucide/svelte/icons/circle-check';
  import CircleX from '@lucide/svelte/icons/circle-x';
  import Download from '@lucide/svelte/icons/download';
  import Puzzle from '@lucide/svelte/icons/puzzle';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import {
    bootTask,
    exportJournal,
    journal as fetchJournal,
    recipeApply,
    recipeRestore,
    recipes as fetchRecipes,
    setBootTask,
    toFailure,
    type Failure,
    type JournalEntry,
    type KnownDevice,
    type RecipeRun,
    type RecipeState,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';
  import { demoJournal, demoRecipes } from '../lib/nas';

  let {
    device,
    demo,
    onssh,
  }: {
    device: KnownDevice | null;
    demo: boolean;
    onssh: () => void;
  } = $props();

  type Action = 'apply' | 'restore';

  let states = $state<RecipeState[] | null>(null);
  let entries = $state<JournalEntry[] | null>(null);
  let failure = $state<Failure | null>(null);
  /** Chosen options per recipe. */
  let chosen = $state<Record<string, string[]>>({});
  /** The recipe and action waiting for confirmation. */
  let confirming = $state<{ id: string; action: Action } | null>(null);
  let needsPassword = $state(false);
  let password = $state('');
  let busy = $state<string | null>(null);
  let result = $state<{ id: string; action: Action; run: RecipeRun } | null>(null);
  let exported = $state(false);
  /** The boot task that applies removed recipes again; `null` while unknown. */
  let boot = $state<boolean | null>(null);
  let bootBusy = $state(false);
  let bootPassword = $state(false);

  const linked = $derived(demo || device?.ssh !== undefined);

  async function load() {
    failure = null;
    if (demo) {
      states = demoRecipes;
      entries = demoJournal;
      boot = false;
    } else {
      try {
        states = await fetchRecipes();
        entries = linked ? await fetchJournal() : [];
        boot = linked ? await bootTask() : null;
      } catch (error) {
        failure = toFailure(error);
      }
    }
    // After a DSM update, the options of the last run are the ones to use again.
    for (const { recipe, lastApplied } of states ?? []) {
      chosen[recipe.id] ??=
        lastApplied?.options ?? recipe.options.filter((option) => option.default).map((o) => o.id);
    }
  }

  $effect(() => {
    void load();
  });

  function toggle(id: string, option: string, on: boolean) {
    const current = chosen[id] ?? [];
    chosen[id] = on ? [...current, option] : current.filter((known) => known !== option);
  }

  async function run() {
    if (!confirming) return;
    const { id, action } = confirming;
    busy = id;
    failure = null;
    result = null;
    try {
      const secret = password === '' ? undefined : password;
      const outcome =
        action === 'apply'
          ? await recipeApply(id, chosen[id] ?? [], secret)
          : await recipeRestore(id, secret);
      result = { id, action, run: outcome };
      confirming = null;
      needsPassword = false;
      // A boot task that is set up learns about the new run right away.
      if (boot) await setBootTask(true, secret).catch(() => undefined);
      await load();
    } catch (error) {
      const reason = toFailure(error);
      if (reason.key === 'password-required') needsPassword = true;
      else {
        if (reason.key === 'sudo') password = '';
        failure = reason;
      }
    } finally {
      busy = null;
    }
  }

  async function toggleBoot() {
    bootBusy = true;
    failure = null;
    try {
      boot = await setBootTask(!boot, password === '' ? undefined : password);
      bootPassword = false;
    } catch (error) {
      const reason = toFailure(error);
      if (reason.key === 'password-required') bootPassword = true;
      else {
        if (reason.key === 'sudo') password = '';
        failure = reason;
      }
    } finally {
      bootBusy = false;
    }
  }

  async function saveJournal() {
    exported = false;
    try {
      exported = await exportJournal();
    } catch (error) {
      failure = toFailure(error);
    }
  }

  const date = (seconds: number) =>
    new Date(seconds * 1000).toLocaleString(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short',
    });
  const recipeTitle = (id: string) => t(`recipe-${id}-title`);
</script>

<div class="extensions">
  <header>
    <h1>{t('nav-extensions')}</h1>
    <p class="muted">{t('extensions-intro')}</p>
  </header>

  <p class="warning">
    <TriangleAlert size={16} aria-hidden="true" />
    <span>{t('extensions-warning')}</span>
  </p>

  {#if !linked}
    <p class="hint">
      <span>{t('extensions-needs-ssh')}</span>
      <button class="button" onclick={onssh}>{t('packages-set-up-ssh')}</button>
    </p>
  {/if}

  {#if states?.some((state) => state.removed)}
    <p class="hint">
      <span>{t('extensions-removed-hint')}</span>
    </p>
  {/if}

  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}

  {#each states ?? [] as { recipe, status, removed } (recipe.id)}
    <section class="card recipe">
      <div class="head">
        <span class="icon"><Puzzle size={20} aria-hidden="true" /></span>
        <div class="grow">
          <h2>{recipeTitle(recipe.id)}</h2>
          <span class="subtle">
            {t('extensions-by', { author: recipe.author, license: recipe.license })} ·
            {recipe.version}
          </span>
        </div>
        {#if removed}
          <span class="badge warning">{t('extensions-removed')}</span>
        {:else if status?.applied}
          <span class="badge success">{t('extensions-applied')}</span>
        {:else if status && !status.supported}
          <span class="badge danger">{t('extensions-unsupported', { dsm: recipe.dsm })}</span>
        {:else if status}
          <span class="badge">{t('extensions-not-applied')}</span>
        {/if}
      </div>
      <p class="muted">{t(`recipe-${recipe.id}-text`)}</p>
      <code class="subtle">{recipe.source}</code>

      {#if recipe.options.length > 0 && !status?.applied}
        <fieldset>
          <legend class="subtle">{t('extensions-options')}</legend>
          {#each recipe.options as option (option.id)}
            <label class="check">
              <input
                type="checkbox"
                checked={(chosen[recipe.id] ?? []).includes(option.id)}
                onchange={(event) => toggle(recipe.id, option.id, event.currentTarget.checked)}
              />
              <span>
                {t(`recipe-${recipe.id}-option-${option.id}`)}
                <code class="subtle">{option.flag}</code>
              </span>
            </label>
          {/each}
        </fieldset>
      {/if}

      {#if !recipe.survivesUpdate}
        <p class="subtle">{t('extensions-update-note')}</p>
      {/if}

      {#if confirming?.id === recipe.id}
        <div class="plan">
          <strong>{t('extensions-plan-title')}</strong>
          <ol>
            {#if confirming.action === 'apply'}
              <li>
                {t('extensions-plan-download', { file: recipe.file, version: recipe.version })}
              </li>
              <li>
                {t('extensions-plan-backup')}
                <code class="block">{recipe.backup.join('  ')}</code>
              </li>
              <li>
                {t('extensions-plan-apply')}
                <code class="block">
                  {recipe.steps.apply.replace('{script}', recipe.file).replace(
                    '{options}',
                    recipe.options
                      .filter((o) => (chosen[recipe.id] ?? []).includes(o.id))
                      .map((o) => o.flag)
                      .join(' '),
                  )}
                </code>
              </li>
              <li>{t('extensions-plan-check')}</li>
            {:else}
              <li>{t('extensions-plan-restore')}</li>
              <li>{t('extensions-plan-check')}</li>
            {/if}
            <li>{t('extensions-plan-journal')}</li>
          </ol>
          <span class="subtle">SHA-256 <code>{recipe.sha256}</code></span>
          {#if needsPassword}
            <label class="field">
              <span>{t('ssh-diag-memory-password')}</span>
              <input
                class="input"
                type="password"
                bind:value={password}
                autocomplete="current-password"
              />
            </label>
          {/if}
          <div class="actions">
            <button class="button" disabled={busy !== null} onclick={() => (confirming = null)}>
              {t('signin-cancel')}
            </button>
            <button class="button danger" disabled={busy !== null || demo} onclick={run}>
              {#if busy === recipe.id}
                {t('extensions-running')}
              {:else}
                {t(confirming.action === 'apply' ? 'extensions-apply' : 'extensions-restore')}
              {/if}
            </button>
          </div>
        </div>
      {:else}
        <div class="actions">
          {#if status?.applied}
            <button
              class="button"
              disabled={!linked || busy !== null}
              onclick={() => (confirming = { id: recipe.id, action: 'restore' })}
            >
              {t('extensions-restore')}
            </button>
          {/if}
          <button
            class="button {status?.applied ? '' : 'primary'}"
            disabled={!linked || busy !== null || status?.supported === false}
            onclick={() => (confirming = { id: recipe.id, action: 'apply' })}
          >
            {t(status?.applied ? 'extensions-apply-again' : 'extensions-apply')}
          </button>
        </div>
      {/if}

      {#if result?.id === recipe.id}
        <div class="result" class:ok={result.run.ok}>
          {#if result.run.ok}
            <CircleCheck size={18} aria-hidden="true" />
          {:else}
            <CircleX size={18} aria-hidden="true" />
          {/if}
          <span>
            {t(result.run.ok ? 'extensions-result-ok' : 'extensions-result-failed')}
            {#if result.run.backup}
              <span class="subtle block"
                >{t('extensions-backup-at')} <code>{result.run.backup}</code></span
              >
            {/if}
          </span>
        </div>
        {#if result.run.output}
          <details open={!result.run.ok}>
            <summary class="subtle">{t('extensions-output')}</summary>
            <pre>{result.run.output}</pre>
          </details>
        {/if}
      {/if}
    </section>
  {:else}
    {#if !failure}<p class="muted">{t('overview-loading')}</p>{/if}
  {/each}

  <section class="card stack">
    <div class="head">
      <div class="grow">
        <h2>{t('extensions-boot-title')}</h2>
        <span class="muted">{t('extensions-boot-text')}</span>
      </div>
      {#if boot !== null}
        <span class="badge" class:success={boot}>
          {t(boot ? 'extensions-boot-on' : 'extensions-boot-off')}
        </span>
      {/if}
    </div>
    {#if bootPassword}
      <label class="field">
        <span>{t('ssh-diag-memory-password')}</span>
        <input
          class="input"
          type="password"
          bind:value={password}
          autocomplete="current-password"
        />
      </label>
    {/if}
    <div class="actions">
      <button
        class="button"
        disabled={demo || !linked || boot === null || bootBusy}
        onclick={toggleBoot}
      >
        {#if bootBusy}
          {t('extensions-running')}
        {:else}
          {t(boot ? 'extensions-boot-remove' : 'extensions-boot-add')}
        {/if}
      </button>
    </div>
  </section>

  <section class="card">
    <div class="head">
      <h2 class="grow">{t('extensions-journal')}</h2>
      <button class="button" disabled={demo || !linked || !entries?.length} onclick={saveJournal}>
        <Download size={15} aria-hidden="true" />
        {t('extensions-export')}
      </button>
    </div>
    {#if exported}<p class="subtle">{t('extensions-exported')}</p>{/if}
    {#if entries && entries.length > 0}
      <ul>
        {#each [...entries].reverse() as entry (`${entry.time}-${entry.recipe}-${entry.action}`)}
          <li>
            {#if entry.ok}
              <CircleCheck size={16} aria-hidden="true" class="good" />
            {:else}
              <CircleX size={16} aria-hidden="true" class="bad" />
            {/if}
            <div class="grow">
              <strong>
                {recipeTitle(entry.recipe)} ·
                {t(entry.action === 'apply' ? 'extensions-applied-action' : 'extensions-restored')}
              </strong>
              <span class="subtle block">
                {date(entry.time)} · DSM {entry.dsm} · {entry.version}
                {#if entry.options.length > 0}· {entry.options.join(', ')}{/if} · {entry.by}
              </span>
            </div>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="subtle">{t('extensions-journal-empty')}</p>
    {/if}
  </section>
  {#if demo}<p class="subtle">{t('extensions-demo')}</p>{/if}
</div>

<style>
  .extensions {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 820px;
    margin: 0 auto;
  }

  header,
  .recipe,
  .stack,
  .plan {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  header {
    gap: 6px;
  }

  .head {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .grow {
    flex: 1;
    min-width: 0;
  }

  h2 {
    font-size: 1.05em;
  }

  .icon {
    display: grid;
    flex: none;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: 10px;
    background: var(--accent-soft);
    color: var(--accent);
  }

  fieldset {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 0;
    padding: 0;
    border: 0;
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

  .plan {
    padding: 12px 14px;
    border-radius: var(--radius-control);
    background: var(--track);
  }

  ol {
    margin: 0;
    padding-left: 1.4em;
    line-height: 1.6;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-width: 18em;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .result {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--danger) 14%, transparent);
  }

  .result.ok {
    background: color-mix(in srgb, var(--success) 16%, transparent);
  }

  pre {
    max-height: 320px;
    overflow: auto;
    margin: 6px 0 0;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: var(--track);
    font-size: 0.82em;
    white-space: pre-wrap;
    user-select: text;
  }

  code {
    font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'DejaVu Sans Mono', monospace;
    font-size: 0.86em;
    word-break: break-all;
    user-select: text;
  }

  ul {
    margin: 8px 0 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 0;
  }

  li + li {
    border-top: 1px solid var(--card-border);
  }

  li :global(.good) {
    flex: none;
    margin-top: 2px;
    color: var(--success);
  }

  li :global(.bad) {
    flex: none;
    margin-top: 2px;
    color: var(--danger);
  }

  .hint,
  .warning {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    background: color-mix(in srgb, var(--warning) 16%, transparent);
  }

  .hint span {
    flex: 1;
  }

  .warning {
    align-items: flex-start;
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

  .button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .button:disabled {
    opacity: 0.5;
    cursor: default;
  }
</style>
