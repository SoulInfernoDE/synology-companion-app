// Points the Flatpak manifest and the AUR package at a GitHub release:
// sets the version and the SHA-256 of both .deb files from the release's
// SHA256SUMS.txt.
//
//   node tools/update-packaging.mjs 0.3.0

import { readFileSync, writeFileSync } from 'node:fs';

const version = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(version ?? '')) {
  console.error('usage: node tools/update-packaging.mjs <version>');
  process.exit(2);
}

const base = 'https://github.com/SoulInfernoDE/synology-companion-app/releases/download';
const response = await fetch(`${base}/v${version}/SHA256SUMS.txt`);
if (!response.ok) {
  console.error(`no SHA256SUMS.txt for v${version}: HTTP ${response.status}`);
  process.exit(1);
}
const sums = Object.fromEntries(
  (await response.text())
    .split('\n')
    .map((line) => line.trim().split(/\s+/))
    .filter((parts) => parts.length === 2)
    .map(([sum, name]) => [name, sum]),
);
const sum = (arch) => {
  const name = `syno-companion_${version}_${arch}.deb`;
  if (!sums[name]) throw new Error(`${name} is not in SHA256SUMS.txt`);
  return sums[name];
};
const [amd64, arm64] = [sum('amd64'), sum('arm64')];
const today = new Date().toISOString().slice(0, 10);

function update(path, edit) {
  const before = readFileSync(path, 'utf8');
  const after = edit(before);
  writeFileSync(path, after);
  console.log(`${path}: ${before === after ? 'unchanged' : 'updated'}`);
}

update('packaging/flatpak/io.github.soulinfernode.synocompanion.yml', (text) =>
  text
    .replace(
      /download\/v[\d.]+\/syno-companion_[\d.]+_amd64\.deb\n(\s+)sha256: \w+/,
      (_, space) =>
        `download/v${version}/syno-companion_${version}_amd64.deb\n${space}sha256: ${amd64}`,
    )
    .replace(
      /download\/v[\d.]+\/syno-companion_[\d.]+_arm64\.deb\n(\s+)sha256: \w+/,
      (_, space) =>
        `download/v${version}/syno-companion_${version}_arm64.deb\n${space}sha256: ${arm64}`,
    ),
);
update('packaging/flatpak/io.github.soulinfernode.synocompanion.metainfo.xml', (text) =>
  text.includes(`version="${version}"`)
    ? text
    : text.replace('<releases>', `<releases>\n    <release version="${version}" date="${today}"/>`),
);
update('packaging/aur/PKGBUILD', (text) =>
  text
    .replace(/^pkgver=.*$/m, `pkgver=${version}`)
    .replace(/^pkgrel=.*$/m, 'pkgrel=1')
    .replace(/^sha256sums_x86_64=.*$/m, `sha256sums_x86_64=('${amd64}')`)
    .replace(/^sha256sums_aarch64=.*$/m, `sha256sums_aarch64=('${arm64}')`),
);
update('packaging/aur/.SRCINFO', (text) =>
  text
    .replace(/pkgver = .*/, `pkgver = ${version}`)
    .replace(/pkgrel = .*/, 'pkgrel = 1')
    .replace(
      /download\/v[\d.]+\/syno-companion_[\d.]+_amd64\.deb/,
      `download/v${version}/syno-companion_${version}_amd64.deb`,
    )
    .replace(
      /download\/v[\d.]+\/syno-companion_[\d.]+_arm64\.deb/,
      `download/v${version}/syno-companion_${version}_arm64.deb`,
    )
    .replace(/sha256sums_x86_64 = \w+/, `sha256sums_x86_64 = ${amd64}`)
    .replace(/sha256sums_aarch64 = \w+/, `sha256sums_aarch64 = ${arm64}`),
);
