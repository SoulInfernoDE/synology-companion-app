// Calls into the Rust core. In a plain browser (`npm run dev`) there is no core;
// the interface then offers only the sample data.

import { invoke } from '@tauri-apps/api/core';
import type { Overview } from './nas';

/** True inside the app; false in the plain browser preview. */
export const hasCore = () => '__TAURI_INTERNALS__' in window;

export interface FoundNas {
  name: string;
  model: string;
  host: string;
  port: number;
  dsm: string | null;
}

export interface KnownDevice {
  id: string;
  name: string;
  model: string;
  host: string;
  port: number;
  account: string;
  fingerprint?: string;
  rememberLogin: boolean;
}

/** An error from the core: `key` selects the message `error-<key>`. */
export interface Failure {
  key: string;
  detail?: string;
}

export interface ConnectRequest {
  host: string;
  port: number;
  name?: string;
  model?: string;
  account: string;
  password?: string;
  otp?: string;
  rememberDevice: boolean;
  rememberLogin: boolean;
  trustFingerprint?: string;
}

export type ConnectOutcome =
  | { status: 'connected'; device: KnownDevice }
  | { status: 'passwordRequired' }
  | { status: 'otpRequired' }
  | { status: 'untrustedCertificate'; fingerprint: string }
  | { status: 'certificateChanged'; fingerprint: string }
  | ({ status: 'failed' } & Failure);

export function isFailure(value: unknown): value is Failure {
  return typeof value === 'object' && value !== null && 'key' in value;
}

/** Turns anything thrown by `invoke` into a `Failure`. */
export function toFailure(error: unknown): Failure {
  return isFailure(error) ? error : { key: 'unexpected', detail: String(error) };
}

export const discover = () => invoke<FoundNas[]>('discover');
export const knownDevices = () => invoke<KnownDevice[]>('known_devices');
export const connect = (request: ConnectRequest) => invoke<ConnectOutcome>('connect', { request });
export const overview = () => invoke<Overview>('overview');
export const disconnect = () => invoke<void>('disconnect');
export const forgetDevice = (id: string) => invoke<void>('forget_device', { id });

/** "nas.local:5001", "192.168.1.20" or "[fe80::1]:5001" → host and port (default 5001). */
export function parseAddress(input: string): { host: string; port: number } | null {
  let text = input
    .trim()
    .replace(/^https?:\/\//i, '')
    .replace(/\/.*$/, '');
  if (!text) return null;
  let port = 5001;
  const bracketed = /^\[([^\]]+)\](?::(\d+))?$/.exec(text);
  if (bracketed) {
    text = bracketed[1]!;
    if (bracketed[2]) port = Number(bracketed[2]);
  } else if ((text.match(/:/g) ?? []).length === 1) {
    const [host, portText] = text.split(':');
    text = host!;
    port = Number(portText);
  }
  if (!text || !Number.isInteger(port) || port < 1 || port > 65535) return null;
  return { host: text, port };
}
