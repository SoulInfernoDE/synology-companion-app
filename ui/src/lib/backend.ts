// Calls into the Rust core. In a plain browser (`npm run dev`) there is no core;
// the interface then offers only the sample data.

import { Channel, invoke } from '@tauri-apps/api/core';
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
  /** SSH with a key of this app. */
  ssh?: SshLink;
}

export interface SshLink {
  port: number;
  /** SHA-256 fingerprint of the trusted host key. */
  hostKey: string;
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
  /** False when DSM did not let the new account in right away; sign in as usual. */
  signedIn: boolean;
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
/** Every signed-in NAS and the one the interface shows. */
export interface Connected {
  devices: KnownDevice[];
  current: string | null;
}

export const disconnect = () => invoke<Connected>('disconnect');
export const connected = () => invoke<Connected>('connected');
export const switchTo = (id: string) => invoke<Connected>('switch_to', { id });
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

export interface SshService {
  enabled: boolean;
  port: number;
  /** The user home service, which key sign-in needs. */
  homes: boolean;
}

export interface LinkRequest {
  port: number;
  password?: string;
  trustFingerprint?: string;
}

export type LinkOutcome =
  | { status: 'linked'; device: KnownDevice }
  | { status: 'passwordRequired' }
  | { status: 'untrustedHostKey'; fingerprint: string }
  | { status: 'hostKeyChanged'; fingerprint: string }
  | ({ status: 'failed' } & Failure);

export interface NetworkDriver {
  interface: string;
  driver: string;
  speed: number | null;
}

export interface Diagnostics {
  dsmVersion: string | null;
  dsmBuild: string | null;
  dsmUpdate: number | null;
  kernel: string;
  machine: string;
  ioUring: 'available' | 'disabled' | 'missing';
  network: NetworkDriver[];
  docker: string | null;
}

export interface MemoryModule {
  slot: string;
  sizeMb: number | null;
  kind: string | null;
  speed: number | null;
  manufacturer: string | null;
  partNumber: string | null;
}

export type TerminalEvent = { event: 'output'; data: number[] } | { event: 'closed' };

export const sshService = () => invoke<SshService>('ssh_service');
export const enableSsh = (port: number) => invoke<void>('enable_ssh', { port });
export const sshLink = (request: LinkRequest) => invoke<LinkOutcome>('ssh_link', { request });
export const sshUnlink = () => invoke<void>('ssh_unlink');
export const diagnostics = () => invoke<Diagnostics>('diagnostics');
export interface MemoryTest {
  time: string;
  passed: boolean;
  error: number | null;
}

export interface Memory {
  modules: MemoryModule[];
  /** Memory tests of Synology Assistant found in the system log. */
  tests: MemoryTest[];
}

export const memoryModules = (password?: string) => invoke<Memory>('memory_modules', { password });

/** Opens a shell on the current NAS; returns its id. */
export function terminalOpen(
  cols: number,
  rows: number,
  onevent: (event: TerminalEvent) => void,
): Promise<number> {
  const events = new Channel<TerminalEvent>();
  events.onmessage = onevent;
  return invoke<number>('terminal_open', { cols, rows, events });
}
export const terminalWrite = (id: number, data: string) =>
  invoke<void>('terminal_write', { id, data });
export const terminalResize = (id: number, cols: number, rows: number) =>
  invoke<void>('terminal_resize', { id, cols, rows });
export const terminalClose = (id: number) => invoke<void>('terminal_close', { id });

export interface Package {
  id: string;
  name: string;
  description: string;
  installed: string | null;
  available: string | null;
  /** "SynoCommunity"; null for Synology's own packages. */
  publisher: string | null;
  status: string | null;
  beta: boolean;
  system: boolean;
  depends: string[];
  download: { url: string; md5: string } | null;
}

export interface PackageSources {
  feeds: { name: string; url: string }[];
  /** 0: Synology only, 1: also trusted publishers, 2: any publisher. */
  trustLevel: number;
}

export const packages = () => invoke<Package[]>('packages');
export const packageSources = () => invoke<PackageSources>('package_sources');
export const addPackageSource = (name: string, url: string, trust: boolean) =>
  invoke<PackageSources>('add_package_source', { name, url, trust });
export const removePackageSource = (url: string) =>
  invoke<PackageSources>('remove_package_source', { url });
/** Installs or updates a package with what it needs; returns the names installed. */
export const installPackage = (id: string, password?: string) =>
  invoke<string[]>('install_package', { id, password });

/** Whether Package Center offers a newer version than the installed one. */
export function hasUpdate(pkg: Package): boolean {
  if (!pkg.installed || !pkg.available) return false;
  const parts = (version: string) =>
    version
      .split(/\D+/)
      .filter(Boolean)
      .map((part) => Number(part));
  const [a, b] = [parts(pkg.available), parts(pkg.installed)];
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const [x, y] = [a[i] ?? -1, b[i] ?? -1];
    if (x !== y) return x > y;
  }
  return false;
}

export interface Recipe {
  id: string;
  source: string;
  author: string;
  license: string;
  version: string;
  download: string;
  file: string;
  sha256: string;
  dsm: string;
  survivesUpdate: boolean;
  backup: string[];
  steps: { check: string; apply: string; restore: string };
  options: { id: string; flag: string; default: boolean }[];
}

export interface RecipeStatus {
  applied: boolean;
  supported: boolean;
  dsm: string;
}

export interface RecipeState {
  recipe: Recipe;
  status: RecipeStatus | null;
  /** The last apply in the journal that no restore undid. */
  lastApplied: JournalEntry | null;
  /** Applied according to the journal but gone now, usually after a DSM update. */
  removed: boolean;
}

export interface RecipeRun {
  ok: boolean;
  output: string;
  backup: string | null;
  applied: boolean;
}

export interface JournalEntry {
  time: number;
  recipe: string;
  version: string;
  action: 'apply' | 'restore';
  options: string[];
  ok: boolean;
  dsm: string;
  backup?: string;
  by: string;
}

export const recipes = () => invoke<RecipeState[]>('recipes');
export const recipeApply = (id: string, options: string[], password?: string) =>
  invoke<RecipeRun>('recipe_apply', { id, options, password });
export const recipeRestore = (id: string, password?: string) =>
  invoke<RecipeRun>('recipe_restore', { id, password });
export const journal = () => invoke<JournalEntry[]>('journal');
export const exportJournal = () => invoke<boolean>('export_journal');

export interface Container {
  name: string;
  image: string;
  state: string;
  project: string | null;
  projectDir: string | null;
  /** The tag names a newer image than the one the container runs. */
  update: boolean;
}

export const containers = (pull: boolean, password?: string) =>
  invoke<Container[]>('containers', { pull, password });
export const updateProject = (project: string, dir: string, password?: string) =>
  invoke<string>('update_project', { project, dir, password });
export const bootTask = () => invoke<boolean>('boot_task');
export const setBootTask = (enable: boolean, password?: string) =>
  invoke<boolean>('set_boot_task', { enable, password });

export interface NextcloudInstall {
  path: string;
  owner: string;
  php: string;
  phpVersion: string;
  version: string;
  maintenance: boolean;
  needsDbUpgrade: boolean;
  database: string | null;
  redis: boolean;
}

export interface NextcloudEnvironment {
  installs: NextcloudInstall[];
  php: string[];
  mariadb: string | null;
  redis: string | null;
}

export interface NextcloudCheck {
  available: string | null;
  requirements: { major: number; phpMin: string; phpBelow: string | null } | null;
}

export type NextcloudStep =
  | 'backup'
  | 'maintenanceOn'
  | 'openPermissions'
  | 'update'
  | 'upgrade'
  | 'repair'
  | 'closePermissions'
  | 'maintenanceOff';

export const nextcloudSteps: NextcloudStep[] = [
  'backup',
  'maintenanceOn',
  'openPermissions',
  'update',
  'upgrade',
  'repair',
  'closePermissions',
  'maintenanceOff',
];

export const nextcloud = (password?: string) =>
  invoke<NextcloudEnvironment>('nextcloud', { password });
export const nextcloudCheck = (install: NextcloudInstall, password?: string) =>
  invoke<NextcloudCheck>('nextcloud_check', { install, password });
export const nextcloudWorkFolder = () => invoke<string>('nextcloud_work_folder');
export const nextcloudStep = (
  install: NextcloudInstall,
  step: NextcloudStep,
  work: string,
  password?: string,
) => invoke<{ ok: boolean; output: string }>('nextcloud_step', { install, step, work, password });

/** Whether PHP `version` is within the bounds of a Nextcloud release. */
export function phpFits(
  version: string,
  requirements: { phpMin: string; phpBelow: string | null },
): boolean {
  const parts = (text: string) => text.split('.').slice(0, 2).map(Number);
  const compare = (a: number[], b: number[]) => (a[0]! - b[0]!) * 100 + (a[1]! - b[1]!);
  const php = parts(version);
  return (
    compare(php, parts(requirements.phpMin)) >= 0 &&
    (requirements.phpBelow === null || compare(php, parts(requirements.phpBelow)) < 0)
  );
}

export interface Download {
  id: string;
  title: string;
  status: string;
  error: string | null;
  size: number;
  downloaded: number;
  speed: number;
  destination: string | null;
}

export type DownloadAction = 'pause' | 'resume' | 'delete';

export const downloads = () => invoke<Download[]>('downloads');
export const addDownload = (uri: string) => invoke<void>('add_download', { uri });
export const downloadAction = (id: string, action: DownloadAction) =>
  invoke<void>('download_action', { id, action });
