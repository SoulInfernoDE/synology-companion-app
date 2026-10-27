<script lang="ts">
  import { FitAddon } from '@xterm/addon-fit';
  import { Terminal } from '@xterm/xterm';
  import '@xterm/xterm/css/xterm.css';
  import { onMount } from 'svelte';
  import {
    terminalClose,
    terminalOpen,
    terminalResize,
    terminalWrite,
    toFailure,
    type Failure,
  } from '../lib/backend';
  import { t } from '../lib/i18n.svelte';

  let { onclose }: { onclose: () => void } = $props();

  let host: HTMLDivElement;
  let failure = $state<Failure | null>(null);
  let ended = $state(false);

  /** xterm draws on a canvas, so it gets the skin's colors as values. */
  function theme() {
    const style = getComputedStyle(document.documentElement);
    const token = (name: string) => style.getPropertyValue(name).trim();
    return {
      background: token('--card'),
      foreground: token('--text'),
      cursor: token('--accent'),
      selectionBackground: token('--accent-soft'),
    };
  }

  onMount(() => {
    const terminal = new Terminal({
      fontFamily:
        "'JetBrains Mono', 'Cascadia Mono', 'SF Mono', Menlo, 'DejaVu Sans Mono', monospace",
      fontSize: 13,
      cursorBlink: true,
      theme: theme(),
    });
    const fit = new FitAddon();
    terminal.loadAddon(fit);
    terminal.open(host);
    fit.fit();

    let id: number | null = null;
    terminalOpen(terminal.cols, terminal.rows, (event) => {
      if (event.event === 'output') {
        terminal.write(new Uint8Array(event.data));
      } else {
        ended = true;
      }
    })
      .then((opened) => {
        id = opened;
        terminal.focus();
      })
      .catch((error) => (failure = toFailure(error)));

    const input = terminal.onData((data) => {
      if (id !== null) void terminalWrite(id, data).catch(() => (ended = true));
    });
    const resized = new ResizeObserver(() => {
      fit.fit();
      if (id !== null) void terminalResize(id, terminal.cols, terminal.rows).catch(() => {});
    });
    resized.observe(host);
    // The skin or color scheme can change while the terminal is open.
    const restyle = new MutationObserver(() => (terminal.options.theme = theme()));
    restyle.observe(document.documentElement, { attributes: true });

    return () => {
      resized.disconnect();
      restyle.disconnect();
      input.dispose();
      if (id !== null) void terminalClose(id);
      terminal.dispose();
    };
  });
</script>

<section class="card terminal">
  <div class="bar">
    <span class="subtle">{ended ? t('ssh-terminal-ended') : t('ssh-terminal-hint')}</span>
    <button class="button" onclick={onclose}>{t('ssh-terminal-close')}</button>
  </div>
  {#if failure}
    <p class="failure" role="alert">
      {t(`error-${failure.key}`)}
      {#if failure.detail}<span class="subtle block">{failure.detail}</span>{/if}
    </p>
  {/if}
  <div class="screen" bind:this={host}></div>
</section>

<style>
  .terminal {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .screen {
    height: 420px;
    min-height: 0;
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
</style>
