// A stand-in for the Rust core in the browser preview, for working on the sign-in
// flow without a NAS. Only loaded by `npm run dev` with `?mock` in the address.
// Sign in with password "demo"; the NAS first shows an unknown certificate, then
// asks for a two-step code (any 6 digits). "vdsm-test" is not set up yet and
// shows the first-time setup.

import type {
  CompleteSetupRequest,
  ConnectOutcome,
  ConnectRequest,
  FoundNas,
  KnownDevice,
  ProbeOutcome,
  ProbeRequest,
  SetupInfo,
} from './backend';
import { demoOverview, demoShares, demoUpdates } from './nas';

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
let connected = false;
let rememberedDevice = false;

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
  connected = true;
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
  connected = true;
  return { device, skipped: [] };
}

const commands: Record<string, (args: Record<string, unknown>) => Promise<unknown>> = {
  probe: (args) => probe(args.request as ProbeRequest),
  begin_setup: async () => (await wait(600), setupInfo),
  complete_setup: (args) => completeSetup(args.request as CompleteSetupRequest),
  discover: async () => (await wait(1200), found),
  known_devices: async () => saved,
  connect: (args) => connect(args.request as ConnectRequest),
  overview: async () => {
    if (!connected) throw { key: 'not-connected' };
    return { ...demoOverview, kernel: null };
  },
  updates: async () => (await wait(1500), demoUpdates),
  shares: async () => demoShares,
  stop_beep: async () => undefined,
  power: async () => {
    connected = false;
  },
  open_share: async () => undefined,
  wake: async () => undefined,
  disconnect: async () => {
    connected = false;
  },
  forget_device: async (args) => {
    saved = saved.filter((device) => device.id !== args.id);
  },
};

(window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
  invoke: (command: string, args: Record<string, unknown> = {}) => {
    const handler = commands[command];
    return handler ? handler(args) : Promise.reject({ key: 'unexpected', detail: command });
  },
};
