<script lang="ts">
	// Stubs preview card (docs/ui/screens.md §1). Shows top N unresolved links
	// (stubs) with count indicator. Links to full workshop/chantier view.
	import type { Stub } from '$api/types';
	import { t } from '$i18n/index.svelte';

	let {
		stubs,
		maxItems = 4
	}: {
		stubs: Stub[];
		maxItems?: number;
	} = $props();

	// Take top N stubs sorted by count
	const displayed = $derived(
		[...stubs]
			.sort((a, b) => b.count - a.count)
			.slice(0, maxItems)
	);
</script>

<div class="card">
	<header class="header">
		<h2 class="header-title">{t('dashboard.workshop')}</h2>
		<a class="view-all" href="/stubs">{t('dashboard.open')} →</a>
	</header>

	{#if stubs.length === 0}
		<div class="empty">{t('dashboard.noStubs')}</div>
	{:else}
		<ul class="list">
			{#each displayed as stub}
				<li class="stub-line">
					<a class="stub-link" href="/stubs">
						<span class="stub-icon">+</span>
						<span class="stub-name">{stub.labels[0] ?? stub.key}</span>
						<span class="stub-count">{stub.count}×</span>
					</a>
				</li>
			{/each}
		</ul>
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
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 15px;
		border-bottom: 1px solid var(--border);
	}
	.header-title {
		margin: 0;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--stub);
	}
	.view-all {
		font-size: 12px;
		color: var(--accent);
		text-decoration: none;
	}
	.view-all:hover {
		text-decoration: underline;
	}

	.empty {
		padding: 24px 15px;
		text-align: center;
		font-size: 13px;
		color: var(--muted);
	}

	.list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.stub-line {
		border-bottom: 1px solid var(--border);
	}
	.stub-line:last-child {
		border-bottom: none;
	}

	.stub-link {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 9px 15px;
		text-decoration: none;
		color: inherit;
		transition: background 0.1s ease;
	}
	.stub-link:hover {
		background: var(--surface-2);
	}

	.stub-icon {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: var(--stub-soft);
		border: 1px solid var(--stub-line);
		border-radius: 6px;
		color: var(--stub);
		font-size: 14px;
		font-weight: 600;
	}

	.stub-name {
		flex: 1;
		font-size: 13.5px;
		font-weight: 500;
		color: var(--text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.stub-count {
		font-family: var(--font-mono);
		font-size: 11.5px;
		font-variant-numeric: tabular-nums;
		color: var(--stub);
	}
</style>
