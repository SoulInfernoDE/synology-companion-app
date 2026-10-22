<script lang="ts">
  /** A usage bar that turns to warning and danger colors as it fills. */
  let { value, label }: { value: number; label: string } = $props();

  const level = $derived(value >= 90 ? 'danger' : value >= 75 ? 'warning' : 'normal');
</script>

<div
  class="meter {level}"
  role="meter"
  aria-label={label}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-valuenow={Math.round(value)}
>
  <div class="fill" style:width="{Math.min(100, Math.max(0, value))}%"></div>
</div>

<style>
  .meter {
    height: 6px;
    border-radius: 999px;
    background: var(--track);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    border-radius: inherit;
    background: var(--accent);
    transition: width 300ms ease;
  }

  .warning .fill {
    background: var(--warning);
  }

  .danger .fill {
    background: var(--danger);
  }
</style>
