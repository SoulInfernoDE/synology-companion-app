import type { Component } from 'svelte';
import Cloud from '@lucide/svelte/icons/cloud';
import ContainerIcon from '@lucide/svelte/icons/container';
import Download from '@lucide/svelte/icons/download';
import FolderOpen from '@lucide/svelte/icons/folder-open';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import Package from '@lucide/svelte/icons/package';
import Puzzle from '@lucide/svelte/icons/puzzle';
import SquareTerminal from '@lucide/svelte/icons/square-terminal';

export type SectionId =
  | 'overview'
  | 'files'
  | 'packages'
  | 'containers'
  | 'downloads'
  | 'ssh'
  | 'extensions'
  | 'nextcloud';

export interface Section {
  id: SectionId;
  /** Fluent message ID of the navigation label. */
  label: string;
  icon: Component;
  /** Roadmap version that brings the section; absent once it exists. */
  planned?: string;
  /** Fluent message ID describing a planned section. */
  description?: string;
  /** Needs expert mode. */
  expert?: boolean;
}

export const sections: readonly Section[] = [
  { id: 'overview', label: 'nav-overview', icon: LayoutDashboard },
  { id: 'files', label: 'nav-files', icon: FolderOpen },
  { id: 'packages', label: 'nav-packages', icon: Package },
  { id: 'containers', label: 'nav-containers', icon: ContainerIcon },
  { id: 'downloads', label: 'nav-downloads', icon: Download },
  { id: 'ssh', label: 'nav-ssh', icon: SquareTerminal },
  { id: 'extensions', label: 'nav-extensions', icon: Puzzle, expert: true },
  { id: 'nextcloud', label: 'nav-nextcloud', icon: Cloud },
];
