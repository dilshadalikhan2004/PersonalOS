import { Bot, FileText, House, LayoutDashboard, Waypoints } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

export type PageKey = 'home' | 'dashboard' | 'chat' | 'documents' | 'timeline' | 'settings';

export interface NavigationItem {
  key: PageKey;
  label: string;
  icon: LucideIcon;
}

export const navigationItems: NavigationItem[] = [
  { key: 'home', label: 'Home', icon: House },
  { key: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
  { key: 'chat', label: 'Chat', icon: Bot },
  { key: 'documents', label: 'Documents', icon: FileText },
  { key: 'timeline', label: 'Timeline', icon: Waypoints },
];

export const pageMetadata: Record<PageKey, { title: string; eyebrow: string }> = {
  home: { title: 'Good morning, Alex', eyebrow: 'Your space' },
  dashboard: { title: 'Dashboard', eyebrow: 'Overview' },
  chat: { title: 'Chat', eyebrow: 'Your private assistant' },
  documents: { title: 'Documents', eyebrow: 'Your library' },
  timeline: { title: 'Timeline', eyebrow: 'Your life in context' },
  settings: { title: 'Settings', eyebrow: 'Personalize LifeOS' },
};
