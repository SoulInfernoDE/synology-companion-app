// Locale-aware number formatting for sizes, rates and durations.

import { currentLocale } from './i18n.svelte';

const DECIMAL_UNITS = ['byte', 'kilobyte', 'megabyte', 'gigabyte', 'terabyte', 'petabyte'] as const;

/**
 * Decimal steps by default, as DSM and drive vendors label capacities. Memory uses
 * `binary` steps with the familiar labels, so 32 GiB of RAM reads "32 GB".
 */
export function bytes(value: number, steps: 'decimal' | 'binary' = 'decimal'): string {
  const base = steps === 'binary' ? 1024 : 1000;
  let unit = 0;
  while (value >= base && unit < DECIMAL_UNITS.length - 1) {
    value /= base;
    unit += 1;
  }
  return new Intl.NumberFormat(currentLocale(), {
    style: 'unit',
    unit: DECIMAL_UNITS[unit],
    unitDisplay: 'short',
    maximumFractionDigits: value < 10 && unit > 0 ? 1 : 0,
  }).format(value);
}

export function rate(bytesPerSecond: number): string {
  return `${bytes(bytesPerSecond)}/s`;
}

export function percent(value: number): string {
  return new Intl.NumberFormat(currentLocale(), {
    style: 'percent',
    maximumFractionDigits: 0,
  }).format(value / 100);
}

export function celsius(value: number): string {
  return new Intl.NumberFormat(currentLocale(), { style: 'unit', unit: 'celsius' }).format(value);
}

/** Days and hours, e.g. "19 days, 4 hours". */
export function duration(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const format = (value: number, unit: 'day' | 'hour') =>
    new Intl.NumberFormat(currentLocale(), { style: 'unit', unit, unitDisplay: 'long' }).format(
      value,
    );
  const parts = days > 0 ? [format(days, 'day'), format(hours, 'hour')] : [format(hours, 'hour')];
  return new Intl.ListFormat(currentLocale(), { style: 'narrow', type: 'unit' }).format(parts);
}
