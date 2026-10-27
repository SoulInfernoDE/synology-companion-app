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
  /** MAC address of the answering port, for Wake-on-LAN. */
  mac: string | null;
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
  mac?: string;
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
  mac?: string;
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

export type ProbeOutcome =
  | { status: 'configured' }
  | { status: 'notConfigured' }
  | { status: 'untrustedCertificate'; fingerprint: string }
  | { status: 'certificateChanged'; fingerprint: string }
  | ({ status: 'failed' } & Failure);

export interface ProbeRequest {
  host: string;
  port: number;
  trustFingerprint?: string;
}

export interface PasswordRules {
  minLength: number | null;
  mixedCase: boolean;
  numeric: boolean;
  special: boolean;
  excludeUsername: boolean;
}

export interface TimeZone {
  value: string;
  display: string;
  offset: number;
}

export interface SetupInfo {
  deviceName: string;
  passwordRules: PasswordRules;
  reservedNames: string[];
  timeZones: TimeZone[];
}

export type UpdatePolicy = 'important' | 'all' | 'notifyOnly';

export interface CompleteSetupRequest {
  host: string;
  port: number;
  model?: string;
  deviceName: string;
  account: string;
  password: string;
  timeZone: string;
  updates: UpdatePolicy;
  rememberLogin: boolean;
  mac?: string;
}

export interface SetupFinished {
  device: KnownDevice;
  skipped: string[];
}

export interface DsmUpdate {
  version: string;
  needsReboot: boolean;
}

export interface PackageUpdate {
  id: string;
  name: string;
  installed: string;
  available: string;
}

/** `dsmChecked: false` or `packages: null` mean DSM could not be asked. */
export interface Updates {
  dsmChecked: boolean;
  /** `null` after a successful check: DSM is up to date. */
  dsm: DsmUpdate | null;
  packages: PackageUpdate[] | null;
}

export interface Share {
  name: string;
  description: string;
  smbName: string;
  encrypted: boolean;
}

export interface Shares {
  shares: Share[];
  smbEnabled: boolean | null;
}

export type PowerAction = 'reboot' | 'shutdown';

/** Settings the Rust core keeps: tray, notifications, language of both. */
export interface CorePreferences {
  language: string | null;
  closeToTray: boolean;
  notifications: boolean;
}

export const getPreferences = () => invoke<CorePreferences>('get_preferences');
export const setPreferences = (preferences: CorePreferences) =>
  invoke<void>('set_preferences', { preferences });

/** Hands the interface language to the core, for tray menu and notifications. */
export async function syncLanguage(language: string | null): Promise<void> {
  if (!hasCore()) return;
  const current = await getPreferences();
  if (current.language !== language) await setPreferences({ ...current, language });
}

export const discover = () => invoke<FoundNas[]>('discover');
export const updates = () => invoke<Updates>('updates');
export const shares = () => invoke<Shares>('shares');
export const stopBeep = () => invoke<void>('stop_beep');
export const power = (action: PowerAction) => invoke<void>('power', { action });
export const openShare = (name: string) => invoke<void>('open_share', { name });
export const wake = (id: string) => invoke<void>('wake', { id });
export const probe = (request: ProbeRequest) => invoke<ProbeOutcome>('probe', { request });
export const beginSetup = (request: ProbeRequest) => invoke<SetupInfo>('begin_setup', { request });
export const completeSetup = (request: CompleteSetupRequest) =>
  invoke<SetupFinished>('complete_setup', { request });
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
