<script lang="ts">
	// One stub row (docs/ui/components.md §5): the link text(s) encountered,
	// mention count, and the entries citing it (as chips), plus a "Créer" action.
	import type { Stub } from '$api/types';
	import { t } from '$i18n/index.svelte';

	let {
		stub,
		titlesBySlug,
		oncreate
	}: {
		stub: Stub;
		titlesBySlug: Record<string, string>;
		oncreate: (stub: Stub) => void;
	} = $props();

	const altLabels = $derived(stub.labels.slice(1));
</script>

<div class="srow">
	<div class="sicon">＋</div>
	<div class="sbody">
		<div class="sname">
			{stub.labels[0] ?? stub.key}
			{#if altLabels.length > 0}
				<span class="alt">{t('workshop.alsoWritten', { labels: altLabels.join(', ') })}</span>
			{/if}
		</div>
		<div class="smeta">
			<span class="cnt">{t('workshop.mentions', { count: stub.count })}</span>
			{#each stub.sources as sourceSlug (sourceSlug)}
				<span class="srcchip">{titlesBySlug[sourceSlug] ?? sourceSlug}</span>
			{/each}
		</div>
	</div>
	<button type="button" class="create-btn" onclick={() => oncreate(stub)}>{t('action.create')}</button>
</div>

<style>
	.srow {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 14px 16px;
		border: 1px solid var(--border);
		border-radius: var(--radius);
		background: var(--surface);
		margin-bottom: 8px;
	}
	.srow:hover {
		border-color: var(--border-strong);
	}
	.sicon {
		width: 34px;
		height: 34px;
		border-radius: 8px;
		flex: none;
		display: grid;
		place-items: center;
		background: var(--stub-soft);
		color: var(--stub);
		border: 1px solid var(--stub-line);
	}
	.sbody {
		flex: 1;
		min-width: 0;
	}
	.sname {
		font-weight: 650;
		font-size: 15px;
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.sname .alt {
		font-weight: 400;
		color: var(--faint);
		font-size: 12.5px;
	}
	.smeta {
		color: var(--muted);
		font-size: 12.5px;
		margin-top: 4px;
		display: flex;
		align-items: center;
		gap: 7px;
		flex-wrap: wrap;
	}
	.smeta .cnt {
		font-family: var(--font-mono);
		color: var(--stub);
	}
	.srcchip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 21px;
		padding: 0 8px;
		border-radius: 20px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		font-size: 11.5px;
		color: var(--text);
	}
	.create-btn {
		height: 27px;
		padding: 0 10px;
		font-size: 12.5px;
		font: inherit;
		border-radius: var(--radius-sm);
		background: var(--stub-soft);
		border: 1px solid var(--stub-line);
		color: var(--stub);
		cursor: pointer;
		flex: none;
	}
	.create-btn:hover {
		filter: brightness(0.97);
	}
</style>
