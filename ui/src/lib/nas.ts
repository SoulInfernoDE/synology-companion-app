// What the interface knows about a NAS. The Rust core fills these from the DSM Web API;
// in demo mode they come from `demoOverview`.

import type { Shares, Updates } from './backend';

export type Health = 'normal' | 'warning' | 'critical';

export interface DemoNas {
  name: string;
  model: string;
  address: string;
  dsm: string;
}

export interface Volume {
  id: string;
  /** 1 for /volume1; the interface names it in the user's language. */
  number: number | null;
  raid: string | null;
  /** "Btrfs" or "ext4". */
  fileSystem: string | null;
  totalBytes: number;
  usedBytes: number;
  health: Health;
}

/** A wired network port; speeds in Mbit/s. */
export interface NetworkPort {
  name: string;
  connected: boolean;
  linkMbps: number | null;
  maxMbps: number | null;
}

export interface Drive {
  id: string;
  slot: string;
  model: string;
  kind: 'hdd' | 'ssd' | 'nvme';
  sizeBytes: number;
  temperatureC: number | null;
  health: Health;
  /** False when the drive is not on Synology's compatibility list; null when DSM does not say. */
  listed: boolean | null;
}

/** Mirrors `companion_dsm::Overview`; `kernel` comes with a later version. */
export interface Overview {
  model: string;
  dsm: string;
  kernel?: string | null;
  uptimeSeconds: number;
  temperatureC: number | null;
  cpuPercent: number;
  /** e.g. "Intel Celeron J4125"; a Virtual DSM reports its host's processor. */
  cpuModel: string | null;
  cpuCores: number | null;
  cpuClockMhz: number | null;
  /** Microarchitecture, e.g. "Gemini Lake Refresh". */
  cpuArchitecture: string | null;
  memoryPercent: number;
  memoryBytes: number;
  networkRxBytesPerSecond: number;
  networkTxBytesPerSecond: number;
  networkPorts: NetworkPort[];
  volumes: Volume[];
  drives: Drive[];
}

const TB = 1000 ** 4;
const GiB = 1024 ** 3;

export const demoDevices: DemoNas[] = [
  { name: 'nas-home', model: 'DS923+', address: '192.168.1.20', dsm: '7.3.1' },
  { name: 'vdsm-test', model: 'Virtual DSM', address: '192.168.1.42', dsm: '7.3.1' },
];

export const demoOverview: Overview = {
  model: 'DS923+',
  dsm: '7.3.1-86003',
  kernel: '5.10.55+',
  uptimeSeconds: 19 * 86400 + 4 * 3600 + 12 * 60,
  temperatureC: 41,
  cpuPercent: 12,
  cpuModel: 'AMD Ryzen R1600',
  cpuCores: 2,
  cpuClockMhz: 2600,
  cpuArchitecture: 'Zen',
  memoryPercent: 46,
  memoryBytes: 32 * GiB,
  networkRxBytesPerSecond: 3_400_000,
  networkTxBytesPerSecond: 820_000,
  networkPorts: [
    { name: 'eth0', connected: true, linkMbps: 1000, maxMbps: 1000 },
    { name: 'eth1', connected: false, linkMbps: null, maxMbps: 1000 },
    { name: 'eth2', connected: true, linkMbps: 10_000, maxMbps: 10_000 },
  ],
  volumes: [
    {
      id: 'volume_1',
      number: 1,
      raid: 'SHR-1',
      fileSystem: 'Btrfs',
      totalBytes: 21.8 * TB,
      usedBytes: 13.1 * TB,
      health: 'normal',
    },
    {
      id: 'volume_2',
      number: 2,
      raid: 'Basic (NVMe)',
      fileSystem: 'Btrfs',
      totalBytes: 0.98 * TB,
      usedBytes: 0.21 * TB,
      health: 'normal',
    },
  ],
  drives: [
    {
      id: 'sata1',
      slot: 'Drive 1',
      model: 'WD Red Plus 12TB',
      kind: 'hdd',
      sizeBytes: 12 * TB,
      temperatureC: 36,
      health: 'normal',
      listed: false,
    },
    {
      id: 'sata2',
      slot: 'Drive 2',
      model: 'WD Red Plus 12TB',
      kind: 'hdd',
      sizeBytes: 12 * TB,
      temperatureC: 37,
      health: 'normal',
      listed: false,
    },
    {
      id: 'sata3',
      slot: 'Drive 3',
      model: 'Seagate IronWolf 12TB',
      kind: 'hdd',
      sizeBytes: 12 * TB,
      temperatureC: 39,
      health: 'warning',
      listed: false,
    },
    {
      id: 'nvme0n1',
      slot: 'M.2 Drive 1',
      model: 'Samsung 990 Pro 1TB',
      kind: 'nvme',
      sizeBytes: 1 * TB,
      temperatureC: 44,
      health: 'normal',
      listed: false,
    },
  ],
};

export const demoUpdates: Updates = {
  dsmChecked: true,
  dsm: { version: '7.4.1-90080 Update 1', needsReboot: true },
  packages: [
    { id: 'SMBService', name: 'SMB Service', installed: '4.15.13-0871', available: '4.15.13-2503' },
    {
      id: 'SecureSignIn',
      name: 'Secure SignIn Service',
      installed: '1.1.5-0362',
      available: '1.1.7-0455',
    },
  ],
};

export const demoShares: Shares = {
  smbEnabled: true,
  shares: [
    {
      name: 'homes',
      description: "homes contains all users' home",
      smbName: 'home',
      encrypted: false,
    },
    { name: 'media', description: 'Photos, music and videos', smbName: 'media', encrypted: false },
    { name: 'backup', description: '', smbName: 'backup', encrypted: true },
  ],
};
