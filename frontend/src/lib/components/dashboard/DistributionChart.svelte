<script lang="ts">
	// Distribution by type bar chart (docs/ui/screens.md §1). Shows horizontal
	// bars proportional to entry count per type. Only shows types with entries.
	import { t, typeLabel } from '$i18n/index.svelte';

	let {
		byType
	}: {
		byType: Record<string, number>;
	} = $props();

	// Sort types by count descending, filter out zeros
	const sorted = $derived(
		Object.entries(byType)
			.filter(([_, count]) => count > 0)
			.sort((a, b) => b[1] - a[1])
	);

	// Max count for percentage calculation
	const maxCount = $derived(
		sorted.length > 0 ? sorted[0][1] : 1
	);
</script>

<div class="card">
	<header class="header">
		<h2 class="header-title">{t('dashboard.distribution')}</h2>
	</header>

	{#if sorted.length === 0}
		<div class="empty">{t('dashboard.noEntries')}</div>
	{:else}
		<div class="bars">
			{#each sorted as [typeName, count]}
				<div class="bar-row">
					<span class="type-name">{typeLabel(typeName)}</span>
					<div class="track">
						<div
							class="fill"
							style="width: {(count / maxCount) * 100}%"
						></div>
					</div>
					<span class="count">{count}</span>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.card {
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		overflow: hidden;
	}

	.header {
		padding: 12px 15px;
		border-bottom: 1px solid var(--border);
	}
	.header-title {
		margin: 0;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--muted);
	}

	.empty {
		padding: 24px 15px;
		text-align: center;
		font-size: 13px;
		color: var(--muted);
	}

	.bars {
		padding: 12px 15px;
	}

	.bar-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 0;
		font-size: 12.5px;
	}

	.type-name {
		width: 88px;
		flex-shrink: 0;
		color: var(--muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.track {
		flex: 1;
		height: 7px;
		background: var(--surface-3);
		border-radius: 4px;
		overflow: hidden;
	}

	.fill {
		height: 100%;
		background: var(--accent);
		border-radius: 4px;
		transition: width 0.3s ease;
	}

	.count {
		width: 24px;
		text-align: right;
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		color: var(--text);
	}
</style>
