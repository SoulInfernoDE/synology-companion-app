// What the interface knows about a NAS. The Rust core fills these from the DSM Web API;
// in demo mode they come from `demoOverview`.

import type {
  Container,
  Diagnostics,
  Download,
  JournalEntry,
  Memory,
  MemoryModule,
  NextcloudEnvironment,
  Package,
  RecipeState,
  Shares,
  Updates,
} from './backend';

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

export const demoDiagnostics: Diagnostics = {
  dsmVersion: '7.4.1',
  dsmBuild: '86000',
  dsmUpdate: 1,
  kernel: '5.10.55+',
  machine: 'x86_64',
  ioUring: 'available',
  network: [
    { interface: 'eth0', driver: 'r8169', speed: 1000 },
    { interface: 'eth1', driver: 'r8169', speed: null },
  ],
  docker: 'Docker version 24.0.2, build 610b8d0',
};

export const demoMemory: MemoryModule[] = [
  {
    slot: 'ChannelA-DIMM0',
    sizeMb: 4096,
    kind: 'DDR4',
    speed: 2666,
    manufacturer: 'Samsung',
    partNumber: 'M471A5244CB0-CWE',
  },
  {
    slot: 'ChannelB-DIMM0',
    sizeMb: 16384,
    kind: 'DDR4',
    speed: 2666,
    manufacturer: 'Crucial',
    partNumber: 'CT16G4SFRA266',
  },
];

const synologyPackage = (
  id: string,
  name: string,
  description: string,
  installed: string | null,
  available: string,
): Package => ({
  id,
  name,
  description,
  installed,
  available,
  publisher: null,
  status: installed ? 'running' : null,
  beta: false,
  system: false,
  depends: [],
  download: null,
});

export const demoPackages: Package[] = [
  synologyPackage(
    'ContainerManager',
    'Container Manager',
    'Run containers and Compose projects on your NAS.',
    '24.0.2-1535',
    '24.0.2-1606',
  ),
  synologyPackage(
    'HyperBackup',
    'Hyper Backup',
    'Back up data to local shares, other NAS and cloud storage.',
    '4.1.2-4031',
    '4.1.2-4031',
  ),
  synologyPackage(
    'SynologyPhotos',
    'Synology Photos',
    'Organise and share photos and videos.',
    '1.7.1-10015',
    '1.8.1-10089',
  ),
  synologyPackage('TextEditor', 'Text Editor', 'Edit text files on the NAS.', null, '1.2.5-0254'),
  {
    ...synologyPackage(
      'synocli-file',
      'SynoCli File Tools',
      'Command-line tools such as nano, less and tree.',
      null,
      '4.0.3-32',
    ),
    publisher: 'SynoCommunity',
  },
];

export const demoRecipes: RecipeState[] = [
  {
    recipe: {
      id: 'third-party-drives',
      source: 'https://github.com/007revad/Synology_HDD_db',
      author: '007revad',
      license: 'MIT',
      version: 'v3.6.139',
      download:
        'https://raw.githubusercontent.com/007revad/Synology_HDD_db/v3.6.139/syno_hdd_db.sh',
      file: 'syno_hdd_db.sh',
      sha256: 'e8a392b9357142ec6d9de944653ba79d858030ba590d8cc25a287175a14fa079',
      dsm: '7.0',
      survivesUpdate: false,
      backup: ['/etc.defaults/synoinfo.conf', '/etc/synoinfo.conf', '/var/lib/disk-compatibility'],
      steps: { check: '', apply: 'bash {script} -e {options}', restore: '' },
      options: [
        { id: 'noupdate', flag: '--noupdate', default: true },
        { id: 'wdda', flag: '--wdda', default: false },
        { id: 'ram', flag: '--ram', default: false },
      ],
    },
    status: { applied: true, supported: true, dsm: '7.2.2-72806' },
    lastApplied: null,
    removed: false,
  },
];

export const demoJournal: JournalEntry[] = [
  {
    time: 1792500000,
    recipe: 'third-party-drives',
    version: 'v3.6.139',
    action: 'apply',
    options: ['noupdate'],
    ok: true,
    dsm: '7.2.2-72806',
    backup:
      '/var/services/homes/admin/.syno-companion/backups/third-party-drives-1792500000.tar.gz',
    by: 'Syno Companion (Linux)',
  },
];

export const demoContainers: Container[] = [
  {
    name: 'nextcloud-app',
    image: 'nextcloud:30-apache',
    state: 'running',
    project: 'nextcloud',
    projectDir: '/volume1/docker/nextcloud',
    update: true,
  },
  {
    name: 'nextcloud-db',
    image: 'mariadb:11.4',
    state: 'running',
    project: 'nextcloud',
    projectDir: '/volume1/docker/nextcloud',
    update: false,
  },
  {
    name: 'pihole',
    image: 'pihole/pihole:latest',
    state: 'running',
    project: null,
    projectDir: null,
    update: true,
  },
];

export const demoMemoryReport: Memory = {
  modules: demoMemory,
  tests: [{ time: '2026-03-14T10:12:40+01:00', passed: true, error: null }],
};

export const demoNextcloud: NextcloudEnvironment = {
  installs: [
    {
      path: '/volume1/web/nextcloud',
      owner: 'http',
      php: '/usr/local/bin/php82',
      phpVersion: '8.2.20',
      version: '31.0.9',
      maintenance: false,
      needsDbUpgrade: false,
      database: 'mysql',
      redis: true,
    },
  ],
  php: ['7.4.33', '8.1.29', '8.2.20'],
  mariadb: '10.11.6',
  redis: '7.2.5',
};

export const demoDownloads: Download[] = [
  {
    id: 'dbid_1',
    title: 'debian-13.1.0-amd64-netinst.iso',
    status: 'downloading',
    error: null,
    size: 754_974_720,
    downloaded: 301_989_888,
    speed: 4_200_000,
    destination: 'downloads',
  },
  {
    id: 'dbid_2',
    title: 'ubuntu-24.04.3-desktop-amd64.iso',
    status: 'paused',
    error: null,
    size: 6_203_355_136,
    downloaded: 1_240_000_000,
    speed: 0,
    destination: 'downloads',
  },
  {
    id: 'dbid_3',
    title: 'big-buck-bunny-1080p.mp4',
    status: 'finished',
    error: null,
    size: 725_106_140,
    downloaded: 725_106_140,
    speed: 0,
    destination: 'video',
  },
];
