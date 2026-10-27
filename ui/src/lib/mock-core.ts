// A stand-in for the Rust core in the browser preview, for working on the sign-in
// flow without a NAS. Only loaded by `npm run dev` with `?mock` in the address.
// Sign in with password "demo"; the NAS first shows an unknown certificate, then
// asks for a two-step code (any 6 digits). "vdsm-test" is not set up yet and
// shows the first-time setup.

import type { Channel } from '@tauri-apps/api/core';
import type {
  CompleteSetupRequest,
  ConnectOutcome,
  ConnectRequest,
  FoundNas,
  KnownDevice,
  LinkOutcome,
  LinkRequest,
  ProbeOutcome,
  ProbeRequest,
  SetupInfo,
  TerminalEvent,
} from './backend';
import {
  demoContainers,
  demoDiagnostics,
  demoDownloads,
  demoJournal,
  demoMemoryReport,
  demoNextcloud,
  demoOverview,
  demoPackages,
  demoRecipes,
  demoShares,
  demoUpdates,
} from './nas';

const FINGERPRINT = Array.from({ length: 32 }, (_, i) =>
  ((i * 37 + 11) % 256).toString(16).padStart(2, '0'),
)
  .join(':')
  .toUpperCase();

const found: FoundNas[] = [
  {
    name: 'nas-home',
    model: 'DS923+',
    host: '192.168.1.20',
    port: 5001,
    dsm: '7.4',
    mac: '02:00:00:00:00:20',
  },
  {
    name: 'vdsm-test',
    model: 'VirtualDSM',
    host: '192.168.1.42',
    port: 5001,
    dsm: '7.4',
    mac: null,
  },
];
const unconfigured = new Set(['192.168.1.42']);
let saved: KnownDevice[] = [];
let trusted = false;
/** Signed-in NAS ids, the current one last chosen. */
let sessions: string[] = [];
let current: string | null = null;
const snapshot = () => ({
  devices: saved.filter((device) => sessions.includes(device.id)),
  current,
});
const open = (id: string) => {
  sessions = [...sessions.filter((known) => known !== id), id];
  current = id;
};
let rememberedDevice = false;
let preferences = { language: null as string | null, closeToTray: false, notifications: true };

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function connect(request: ConnectRequest): Promise<ConnectOutcome> {
  await wait(500);
  if (!trusted && request.trustFingerprint !== FINGERPRINT) {
    return { status: 'untrustedCertificate', fingerprint: FINGERPRINT };
  }
  trusted = true;
  const password =
    request.password ??
    (saved.find((d) => d.host === request.host)?.rememberLogin ? 'demo' : undefined);
  if (password === undefined) return { status: 'passwordRequired' };
  if (password !== 'demo') return { status: 'failed', key: 'wrong-credentials' };
  if (!rememberedDevice) {
    if (!request.otp) return { status: 'otpRequired' };
    if (!/^\d{6}$/.test(request.otp)) return { status: 'failed', key: 'otp-wrong' };
    rememberedDevice = request.rememberDevice;
  }
  const device: KnownDevice = {
    id: `${request.host.toLowerCase()}:${request.port}`,
    name: request.name ?? request.host,
    model: request.model ?? '',
    host: request.host,
    port: request.port,
    account: request.account,
    fingerprint: FINGERPRINT,
    rememberLogin: request.rememberLogin,
    mac: request.mac,
  };
  saved = [device, ...saved.filter((known) => known.id !== device.id)];
  open(device.id);
  return { status: 'connected', device };
}

async function probe(request: ProbeRequest): Promise<ProbeOutcome> {
  await wait(400);
  if (!trusted && request.trustFingerprint !== FINGERPRINT) {
    return { status: 'untrustedCertificate', fingerprint: FINGERPRINT };
  }
  trusted = true;
  return { status: unconfigured.has(request.host) ? 'notConfigured' : 'configured' };
}

const setupInfo: SetupInfo = {
  deviceName: 'VirtualDSM',
  passwordRules: {
    minLength: 8,
    mixedCase: true,
    numeric: true,
    special: false,
    excludeUsername: true,
  },
  reservedNames: ['admin', 'guest', 'root'],
  timeZones: [
    { value: 'London', display: '(GMT) Dublin, Edinburgh, Lisbon, London', offset: 0 },
    {
      value: 'Amsterdam',
      display: '(GMT+01:00) Amsterdam, Berlin, Rome, Stockholm, Vienna',
      offset: 3600,
    },
    { value: 'Athens', display: '(GMT+02:00) Athens, Istanbul, Minsk', offset: 7200 },
  ],
};

async function completeSetup(request: CompleteSetupRequest) {
  await wait(1500);
  unconfigured.delete(request.host);
  rememberedDevice = true;
  const device: KnownDevice = {
    id: `${request.host.toLowerCase()}:${request.port}`,
    name: request.deviceName,
    model: request.model ?? '',
    host: request.host,
    port: request.port,
    account: request.account,
    fingerprint: FINGERPRINT,
    rememberLogin: request.rememberLogin,
  };
  saved = [device, ...saved.filter((known) => known.id !== device.id)];
  open(device.id);
  return { device, skipped: [], signedIn: true };
}

// The journal says applied, the check says no: as after a DSM update.
let mockRecipes = demoRecipes.map((state) => ({
  ...state,
  status: state.status && { ...state.status, applied: false },
  lastApplied: demoJournal[0] ?? null,
  removed: true,
}));
let mockJournal = [...demoJournal];
let mockBoot = false;

async function recipeRun(args: Record<string, unknown>, action: 'apply' | 'restore') {
  await wait(1500);
  if (args.password === undefined) throw { key: 'password-required' };
  if (args.password !== 'demo') throw { key: 'sudo' };
  const applied = action === 'apply';
  mockRecipes = mockRecipes.map((state) =>
    state.recipe.id === args.id
      ? { ...state, status: state.status && { ...state.status, applied } }
      : state,
  );
  const backup = applied ? '/var/services/homes/demo/.syno-companion/backups/mock.tar.gz' : null;
  mockJournal = [
    ...mockJournal,
    {
      time: Math.floor(Date.now() / 1000),
      recipe: String(args.id),
      version: 'v3.6.139',
      action,
      options: (args.options as string[] | undefined) ?? [],
      ok: true,
      dsm: '7.2.2-72806',
      backup: backup ?? undefined,
      by: 'Syno Companion (Linux)',
    },
  ];
  return { ok: true, output: `Mock ${action}: nothing changed.`, backup, applied };
}

let sshEnabled = false;
let mockDownloads = demoDownloads;
let mockContainers = demoContainers;
let mockPackages = demoPackages;
let mockSources = { feeds: [] as { name: string; url: string }[], trustLevel: 0 };
let terminalOutput: ((text: string) => void) | null = null;
const HOST_KEY = 'SHA256:mOcKhOsTkEyFoRtHePrEvIeWoNlY0123456789abcd';

/** SSH: first the host key question, then the password unless it is remembered. */
async function sshLink(request: LinkRequest): Promise<LinkOutcome> {
  await wait(700);
  if (!sshEnabled) return { status: 'failed', key: 'ssh-network' };
  if (request.trustFingerprint !== HOST_KEY) {
    return { status: 'untrustedHostKey', fingerprint: HOST_KEY };
  }
  const device = saved.find((known) => known.id === current);
  if (!device) throw { key: 'not-connected' };
  const password = request.password ?? (device.rememberLogin ? 'demo' : undefined);
  if (password === undefined) return { status: 'passwordRequired' };
  if (password !== 'demo') return { status: 'failed', key: 'ssh-auth' };
  device.ssh = { port: request.port, hostKey: HOST_KEY };
  return { status: 'linked', device };
}

const commands: Record<string, (args: Record<string, unknown>) => Promise<unknown>> = {
  probe: (args) => probe(args.request as ProbeRequest),
  begin_setup: async () => (await wait(600), setupInfo),
  complete_setup: (args) => completeSetup(args.request as CompleteSetupRequest),
  discover: async () => (await wait(1200), found),
  known_devices: async () => saved,
  connect: (args) => connect(args.request as ConnectRequest),
  overview: async () => {
    if (!current) throw { key: 'not-connected' };
    return { ...demoOverview, kernel: null };
  },
  updates: async () => (await wait(1500), demoUpdates),
  shares: async () => demoShares,
  stop_beep: async () => undefined,
  get_preferences: async () => preferences,
  set_preferences: async (args) => {
    preferences = args.preferences as typeof preferences;
  },
  power: async () => {
    sessions = sessions.filter((id) => id !== current);
    current = sessions[0] ?? null;
  },
  open_share: async () => undefined,
  wake: async () => undefined,
  disconnect: async () => {
    sessions = sessions.filter((id) => id !== current);
    current = sessions[0] ?? null;
    return snapshot();
  },
  connected: async () => snapshot(),
  switch_to: async (args) => {
    current = args.id as string;
    return snapshot();
  },
  ssh_service: async () => ({ enabled: sshEnabled, port: 22, homes: true }),
  enable_ssh: async () => {
    sshEnabled = true;
  },
  ssh_link: (args) => sshLink(args.request as LinkRequest),
  ssh_unlink: async () => {
    const device = saved.find((known) => known.id === current);
    if (device) delete device.ssh;
  },
  diagnostics: async () => (await wait(500), demoDiagnostics),
  memory_modules: async (args) => {
    await wait(500);
    if (args.password === undefined) throw { key: 'password-required' };
    if (args.password !== 'demo') throw { key: 'sudo' };
    return demoMemoryReport;
  },
  terminal_open: async (args) => {
    const events = args.events as Channel<TerminalEvent>;
    const say = (text: string) =>
      events.onmessage({ event: 'output', data: Array.from(new TextEncoder().encode(text)) });
    terminalOutput = say;
    say('Mock shell: everything you type is echoed.\r\n$ ');
    return 1;
  },
  terminal_write: async (args) => {
    const data = args.data as string;
    terminalOutput?.(data === '\r' ? '\r\n$ ' : data);
  },
  terminal_resize: async () => undefined,
  terminal_close: async () => {
    terminalOutput = null;
  },
  packages: async () => (await wait(600), mockPackages),
  package_sources: async () => mockSources,
  add_package_source: async (args) => {
    await wait(500);
    if (!String(args.url).startsWith('https://')) throw { key: 'feed-invalid' };
    mockSources = {
      feeds: [...mockSources.feeds, { name: String(args.name), url: String(args.url) }],
      trustLevel: args.trust ? Math.max(1, mockSources.trustLevel) : mockSources.trustLevel,
    };
    return mockSources;
  },
  remove_package_source: async (args) => {
    mockSources = { ...mockSources, feeds: mockSources.feeds.filter((f) => f.url !== args.url) };
    return mockSources;
  },
  install_package: async (args) => {
    await wait(1200);
    if (args.password === undefined) throw { key: 'password-required' };
    if (args.password !== 'demo') throw { key: 'sudo' };
    const pkg = mockPackages.find((known) => known.id === args.id);
    if (!pkg) throw { key: 'package-unavailable', detail: String(args.id) };
    mockPackages = mockPackages.map((known) =>
      known.id === pkg.id ? { ...known, installed: known.available, status: 'running' } : known,
    );
    return [pkg.name];
  },
  recipes: async () => (await wait(500), mockRecipes),
  boot_task: async () => mockBoot,
  set_boot_task: async (args) => {
    await wait(800);
    if (args.password === undefined) throw { key: 'password-required' };
    mockBoot = Boolean(args.enable);
    return mockBoot;
  },
  journal: async () => mockJournal,
  export_journal: async () => true,
  recipe_apply: async (args) => recipeRun(args, 'apply'),
  recipe_restore: async (args) => recipeRun(args, 'restore'),
  containers: async (args) => {
    await wait(args.pull ? 1500 : 500);
    if (args.password === undefined) throw { key: 'password-required' };
    if (args.password !== 'demo') throw { key: 'sudo' };
    return mockContainers;
  },
  update_project: async (args) => {
    await wait(1500);
    mockContainers = mockContainers.map((c) =>
      c.project === args.project ? { ...c, update: false } : c,
    );
    return 'Recreated';
  },
  nextcloud: async (args) => {
    await wait(800);
    if (args.password === undefined) throw { key: 'password-required' };
    return demoNextcloud;
  },
  nextcloud_check: async () => (
    await wait(900),
    { available: '32.0.1', requirements: { major: 32, phpMin: '8.1', phpBelow: '8.5' } }
  ),
  nextcloud_work_folder: async () => '/var/services/homes/demo/.syno-companion/nextcloud/1',
  nextcloud_step: async (args) => {
    await wait(700);
    return { ok: true, output: `Mock step ${String(args.step)}: done.` };
  },
  downloads: async () => mockDownloads,
  add_download: async (args) => {
    const uri = String(args.uri);
    if (!/^(https?|ftps?):\/\/|^magnet:/i.test(uri)) throw { key: 'download-link' };
    mockDownloads = [
      ...mockDownloads,
      {
        id: `dbid_${mockDownloads.length + 10}`,
        title: uri.split('/').pop() || uri,
        status: 'waiting',
        error: null,
        size: 0,
        downloaded: 0,
        speed: 0,
        destination: 'downloads',
      },
    ];
  },
  download_action: async (args) => {
    if (args.action === 'delete') mockDownloads = mockDownloads.filter((d) => d.id !== args.id);
    else
      mockDownloads = mockDownloads.map((d) =>
        d.id === args.id ? { ...d, status: args.action === 'pause' ? 'paused' : 'downloading' } : d,
      );
  },
  forget_device: async (args) => {
    saved = saved.filter((device) => device.id !== args.id);
  },
};

let callbacks = 0;
(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
  // Channels register a callback for the core; the mock calls `onmessage` directly.
  transformCallback: () => ++callbacks,
  invoke: (command: string, args: Record<string, unknown> = {}) => {
    const handler = commands[command];
    return handler ? handler(args) : Promise.reject({ key: 'unexpected', detail: command });
  },
};
