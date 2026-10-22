// A stand-in for the Rust core in the browser preview, for working on the sign-in
// flow without a NAS. Only loaded by `npm run dev` with `?mock` in the address.
// Sign in with password "demo"; the NAS first shows an unknown certificate, then
// asks for a two-step code (any 6 digits).

import type { ConnectOutcome, ConnectRequest, FoundNas, KnownDevice } from './backend';
import { demoOverview } from './nas';

const FINGERPRINT = Array.from({ length: 32 }, (_, i) =>
  ((i * 37 + 11) % 256).toString(16).padStart(2, '0'),
)
  .join(':')
  .toUpperCase();

const found: FoundNas[] = [
  { name: 'nas-home', model: 'DS923+', host: '192.168.1.20', port: 5001, dsm: '7.3' },
];
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
  };
  saved = [device, ...saved.filter((known) => known.id !== device.id)];
  connected = true;
  return { status: 'connected', device };
}

const commands: Record<string, (args: Record<string, unknown>) => Promise<unknown>> = {
  discover: async () => (await wait(1200), found),
  known_devices: async () => saved,
  connect: (args) => connect(args.request as ConnectRequest),
  overview: async () => {
    if (!connected) throw { key: 'not-connected' };
    return { ...demoOverview, kernel: null, updates: null };
  },
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
