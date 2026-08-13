<script module lang="ts">
	// Builds display chips from a URL's search params, skipping view-state keys.
	export function chipsFromParams(
		params: URLSearchParams,
		fieldLabelOf: (name: string) => string
	): { key: string; value: string; label: string }[] {
		const reserved = new Set(['sort', 'page', 'per_page']);
		const chips: { key: string; value: string; label: string }[] = [];
		for (const [key, value] of params.entries()) {
			if (reserved.has(key)) continue;
			chips.push({ key, value, label: fieldLabelOf(key) });
		}
		return chips;
	}
</script>

<script lang="ts">
	// Active-filter chips (docs/ui/screens.md §3): one removable chip per
	// `tag=`/`<field>=` pair currently in the URL. `sort`/`page`/`per_page` are
	// view state, not filters, and never show up here.
	import { t } from '$i18n/index.svelte';

	interface Chip {
		key: string;
		value: string;
		label: string;
	}

	let {
		chips,
		onremove,
		onclearall
	}: {
		chips: Chip[];
		onremove: (key: string, value: string) => void;
		onclearall: () => void;
	} = $props();
</script>

{#if chips.length > 0}
	<div class="filters">
		<span class="lbl">{t('list.filters')}</span>
		{#each chips as chip (chip.key + ':' + chip.value)}
			<span class="fchip">
				<span class="fk">{chip.label}</span>
				{chip.value}
				<button class="x" onclick={() => onremove(chip.key, chip.value)} aria-label="✕">✕</button>
			</span>
		{/each}
		<button class="clearall" onclick={onclearall}>{t('list.clearAll')}</button>
	</div>
{/if}

<style>
	.filters {
		display: flex;
		align-items: center;
		gap: 7px;
		padding: 0 0 12px;
		flex-wrap: wrap;
	}
	.lbl {
		font-size: 11px;
		color: var(--faint);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.fchip {
		height: 24px;
		padding: 0 6px 0 9px;
		display: inline-flex;
		align-items: center;
		gap: 7px;
		background: var(--accent-soft);
		border: 1px solid var(--accent-line);
		color: var(--accent);
		border-radius: 20px;
		font-size: 12px;
	}
	.fk {
		opacity: 0.75;
		font-family: var(--font-mono);
		font-size: 11px;
	}
	.x {
		border: 0;
		background: transparent;
		color: inherit;
		cursor: pointer;
		opacity: 0.7;
		padding: 0;
		font-size: 11px;
		line-height: 1;
	}
	.x:hover {
		opacity: 1;
	}
	.clearall {
		border: 0;
		background: transparent;
		font-size: 12px;
		color: var(--muted);
		cursor: pointer;
		padding: 0;
	}
	.clearall:hover {
		color: var(--text);
	}
</style>
