<script lang="ts" module>
	// Small stroke-icon set (Lucide-style, 24×24, currentColor) for the app
	// chrome. Type items use a monogram (see Nav) rather than a bespoke glyph;
	// per-type icons arrive with the detailed screens.
	export type IconName =
		| 'search'
		| 'link'
		| 'sun'
		| 'moon'
		| 'monitor'
		| 'plus'
		| 'folder'
		| 'x'
		| 'feather'
		| 'chevron-right'
		| 'chevron-left'
		| 'chevron-down'
		| 'square-plus';

	const PATHS: Record<IconName, string[]> = {
		search: ['M11 11m-8 0a8 8 0 1 0 16 0a8 8 0 1 0 -16 0', 'M21 21l-4.3-4.3'],
		link: ['M9 17H7A5 5 0 0 1 7 7h2', 'M15 7h2a5 5 0 1 1 0 10h-2', 'M8 12h8'],
		sun: [
			'M12 12m-4 0a4 4 0 1 0 8 0a4 4 0 1 0 -8 0',
			'M12 2v2',
			'M12 20v2',
			'M4.9 4.9l1.4 1.4',
			'M17.7 17.7l1.4 1.4',
			'M2 12h2',
			'M20 12h2',
			'M6.3 17.7l-1.4 1.4',
			'M19.1 4.9l-1.4 1.4'
		],
		moon: ['M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9z'],
		monitor: ['M3 4h18v12H3z', 'M8 20h8', 'M12 16v4'],
		plus: ['M5 12h14', 'M12 5v14'],
		folder: [
			'M4 5h5l2 2h9a1 1 0 0 1 1 1v9a2 2 0 0 1-2 2H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1z'
		],
		x: ['M18 6 6 18', 'M6 6l12 12'],
		feather: ['M20.2 12.2a6 6 0 0 0-8.5-8.5L5 10.5V19h8.5z', 'M16 8 2 22', 'M17.5 15H9'],
		'chevron-right': ['M9 6l6 6-6 6'],
		'chevron-left': ['M15 6l-6 6 6 6'],
		'chevron-down': ['M6 9l6 6 6-6'],
		'square-plus': ['M4 4h16v16H4z', 'M12 8v8', 'M8 12h8']
	};
</script>

<script lang="ts">
	let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#each PATHS[name] as d (d)}
		<path {d} />
	{/each}
</svg>
