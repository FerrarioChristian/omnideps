import RocketIcon from '@lucide/svelte/icons/rocket';
import TerminalIcon from '@lucide/svelte/icons/terminal';
import LayersIcon from '@lucide/svelte/icons/layers';
import SettingsIcon from '@lucide/svelte/icons/settings';
import type { DocsConfig } from './types.js';

export const docsConfig: DocsConfig = {
	site: {
		title: 'Omnideps Docs',
		description: 'A language-agnostic architectural dependency analyzer and visualizer based on Tree-sitter.',
		social: {
			github: 'https://github.com/FerrarioChristian',
			linkedin: 'https://www.linkedin.com/in/christian-ferrario/'
		}
	},
	sidebar: [
		{
			label: 'Getting Started',
			icon: RocketIcon,
			autogenerate: { directory: 'getting-started' }
		},
		{
			label: 'CLI Reference',
			icon: TerminalIcon,
			autogenerate: { directory: 'cli' }
		},
		{
			label: 'Architecture & Languages',
			icon: LayersIcon,
			autogenerate: { directory: 'architecture' }
		},
		{
			label: 'Guides',
			icon: SettingsIcon,
			autogenerate: { directory: 'guides' }
		}
	],
	toc: {
		minDepth: 2,
		maxDepth: 3
	},
	versions: {
		current: 'v1.0.0',
		versions: [
			{ label: 'v1.0.0 (latest)', href: '/docs' }
		]
	},
	i18n: {
		defaultLocale: 'en',
		locales: [
			{ code: 'en', label: 'English', flag: '🇺🇸' }
		]
	}
};
