<script lang="ts">
  import ArrowDown from '@lucide/svelte/icons/arrow-down';
  import ArrowUp from '@lucide/svelte/icons/arrow-up';
  import Cpu from '@lucide/svelte/icons/cpu';
  import Database from '@lucide/svelte/icons/database';
  import FlaskConical from '@lucide/svelte/icons/flask-conical';
  import HardDrive from '@lucide/svelte/icons/hard-drive';
  import MemoryStick from '@lucide/svelte/icons/memory-stick';
  import Network from '@lucide/svelte/icons/network';
  import RefreshCw from '@lucide/svelte/icons/refresh-cw';
  import Thermometer from '@lucide/svelte/icons/thermometer';
  import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
  import Meter from './Meter.svelte';
  import { bytes, celsius, duration, percent, rate } from '../lib/format';
  import { t } from '../lib/i18n.svelte';
  import type { Health, Overview } from '../lib/nas';

  let { overview, name, demo }: { overview: Overview; name: string; demo: boolean } = $props();

  const badgeClass: Record<Health, string> = {
    normal: 'success',
    warning: 'warning',
    critical: 'danger',
  };

  const overall = $derived<Health>(
    [...overview.volumes, ...overview.drives].reduce<Health>(
      (worst, { health }) =>
        health === 'critical' || worst === 'critical'
          ? 'critical'
          : health === 'warning' || worst === 'warning'
            ? 'warning'
            : 'normal',
      'normal',
    ),
  );
</script>

<div class="overview">
  <header>
    <div>
      <h1>{name}</h1>
      <p class="muted">
        {[
          overview.model,
          overview.dsm ? t('overview-dsm', { version: overview.dsm }) : null,
          overview.kernel ? t('overview-kernel', { version: overview.kernel }) : null,
        ]
          .filter(Boolean)
          .join(' · ')}
      </p>
      <p class="subtle">{t('overview-uptime', { duration: duration(overview.uptimeSeconds) })}</p>
    </div>
    <span class="badge {badgeClass[overall]}">{t(`health-${overall}`)}</span>
  </header>

  {#if demo}
    <p class="demo-note">
      <FlaskConical size={16} aria-hidden="true" />
      {t('overview-demo')}
    </p>
  {/if}

  <section class="tiles">
    <article class="card tile">
      <h2><Cpu size={16} aria-hidden="true" /> {t('overview-cpu')}</h2>
      <p class="value">{percent(overview.cpuPercent)}</p>
      <Meter value={overview.cpuPercent} label={t('overview-cpu')} />
    </article>
    <article class="card tile">
      <h2><MemoryStick size={16} aria-hidden="true" /> {t('overview-memory')}</h2>
      <p class="value">
        {percent(overview.memoryPercent)}
        <span class="subtle"
          >{t('overview-memory-of', { total: bytes(overview.memoryBytes, 'binary') })}</span
        >
      </p>
      <Meter value={overview.memoryPercent} label={t('overview-memory')} />
    </article>
    <article class="card tile">
      <h2><Network size={16} aria-hidden="true" /> {t('overview-network')}</h2>
      <p class="value rates">
        <span
          ><ArrowDown size={15} aria-hidden="true" />{rate(overview.networkRxBytesPerSecond)}</span
        >
        <span><ArrowUp size={15} aria-hidden="true" />{rate(overview.networkTxBytesPerSecond)}</span
        >
      </p>
    </article>
    {#if overview.temperatureC !== null}
      <article class="card tile">
        <h2><Thermometer size={16} aria-hidden="true" /> {t('overview-temperature')}</h2>
        <p class="value">{celsius(overview.temperatureC)}</p>
      </article>
    {/if}
  </section>

  <div class="columns">
    <section class="card">
      <h2><Database size={16} aria-hidden="true" /> {t('overview-storage')}</h2>
      <ul class="list">
        {#each overview.volumes as volume (volume.id)}
          {@const used = volume.totalBytes > 0 ? (volume.usedBytes / volume.totalBytes) * 100 : 0}
          {@const volumeName =
            volume.number !== null ? t('volume-name', { number: volume.number }) : volume.id}
          <li class="volume">
            <div class="line">
              <strong>{volumeName}</strong>
              {#if volume.raid}<span class="subtle">{volume.raid}</span>{/if}
              <span class="badge {badgeClass[volume.health]}">{t(`health-${volume.health}`)}</span>
            </div>
            <Meter value={used} label={volumeName} />
            <span class="subtle">
              {t('overview-volume-used', {
                used: bytes(volume.usedBytes),
                total: bytes(volume.totalBytes),
              })} · {percent(used)}
            </span>
          </li>
        {/each}
      </ul>
    </section>

    {#if overview.updates}
      <section class="card">
        <h2><RefreshCw size={16} aria-hidden="true" /> {t('overview-updates')}</h2>
        <ul class="list">
          <li>
            {overview.updates.dsm
              ? t('overview-update-dsm', { version: overview.updates.dsm })
              : t('overview-update-dsm-none')}
          </li>
          <li>{t('overview-update-packages', { count: overview.updates.packages })}</li>
        </ul>
      </section>
    {/if}
  </div>

  <section class="card">
    <h2><HardDrive size={16} aria-hidden="true" /> {t('overview-drives')}</h2>
    <ul class="list">
      {#each overview.drives as drive (drive.id)}
        <li class="drive">
          <div class="drive-main">
            <strong>{drive.model}</strong>
            <span class="subtle">
              {[
                drive.slot,
                t(`drive-${drive.kind}`),
                bytes(drive.sizeBytes),
                drive.temperatureC !== null ? celsius(drive.temperatureC) : null,
              ]
                .filter(Boolean)
                .join(' · ')}
            </span>
          </div>
          {#if drive.listed === false}
            <span class="badge warning" title={t('overview-drive-unlisted-hint')}>
              <TriangleAlert size={13} aria-hidden="true" />
              {t('overview-drive-unlisted')}
            </span>
          {/if}
          <span class="badge {badgeClass[drive.health]}">{t(`health-${drive.health}`)}</span>
        </li>
      {/each}
    </ul>
    {#if overview.drives.some((drive) => drive.listed === false)}
      <p class="subtle hint">{t('overview-drive-unlisted-hint')}</p>
    {/if}
  </section>
</div>

<style>
  .overview {
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-width: 1080px;
    margin: 0 auto;
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }

  .demo-note {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: var(--radius-control);
    background: var(--accent-soft);
    color: var(--text);
  }

  h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    color: var(--text-2);
    font-size: 0.95em;
  }

  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 14px;
  }

  /* Without a temperature sensor there are three tiles. */
  .tiles:has(> :nth-child(3):last-child) {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .tile {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .tile h2 {
    margin-bottom: 0;
  }

  .value {
    font-size: 1.6em;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .value .subtle {
    font-size: 0.5em;
    font-weight: 400;
  }

  .rates {
    display: flex;
    flex-direction: column;
    font-size: 1.05em;
  }

  .rates span {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .columns {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 14px;
  }

  .columns:has(> :only-child) {
    grid-template-columns: 1fr;
  }

  .list {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .list li {
    padding: 10px 0;
  }

  .list li + li {
    border-top: 1px solid var(--card-border);
  }

  .volume {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .line .badge {
    margin-left: auto;
  }

  .drive {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .drive-main {
    display: flex;
    /* Wraps the badges below the model before squeezing it. */
    flex: 1 1 220px;
    flex-direction: column;
    min-width: 0;
    line-height: 1.3;
  }

  .hint {
    margin-top: 8px;
  }

  @media (max-width: 1040px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 860px) {
    .columns {
      grid-template-columns: 1fr;
    }

    .drive {
      flex-wrap: wrap;
    }
  }
</style>
