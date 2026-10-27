import type { Component } from 'svelte';
import Cloud from '@lucide/svelte/icons/cloud';
import FolderOpen from '@lucide/svelte/icons/folder-open';
import LayoutDashboard from '@lucide/svelte/icons/layout-dashboard';
import Package from '@lucide/svelte/icons/package';
import Puzzle from '@lucide/svelte/icons/puzzle';
import SquareTerminal from '@lucide/svelte/icons/square-terminal';

export type SectionId = 'overview' | 'files' | 'packages' | 'ssh' | 'extensions' | 'nextcloud';

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
  { id: 'ssh', label: 'nav-ssh', icon: SquareTerminal },
  {
    id: 'extensions',
    label: 'nav-extensions',
    icon: Puzzle,
    planned: '0.4',
    description: 'planned-extensions',
    expert: true,
  },
  {
    id: 'nextcloud',
    label: 'nav-nextcloud',
    icon: Cloud,
    planned: '1.0',
    description: 'planned-nextcloud',
  },
];
