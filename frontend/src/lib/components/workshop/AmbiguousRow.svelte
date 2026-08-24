<script lang="ts">
	// One ambiguous-link row (docs/ui/screens.md §5): the raw target text, its
	// candidates, and where it occurs, with a "Lever l'ambiguïté" action.
	import { t, typeLabel } from '$i18n/index.svelte';
	import type { AmbiguousGroup } from '$lib/workshop/link-rewrite';
	import type { EntrySummary } from '$api/types';

	let {
		group,
		candidates,
		ondisambiguate
	}: {
		group: AmbiguousGroup;
		candidates: EntrySummary[];
		ondisambiguate: (group: AmbiguousGroup) => void;
	} = $props();
</script>

<div class="arow">
	<div class="atop">
		<div class="aicon">⚟</div>
		<div class="abody">
			<div class="alink">[[{group.targetRaw}]]</div>
			<div class="smeta">
				{t('workshop.candidatesMatch', { count: candidates.length })} ·
				{t('workshop.occurrenceCount', { count: group.occurrences.length })}
				{#each group.occurrences.slice(0, 3) as occ (occ.slug + (occ.field ?? ''))}
					<span class="srcchip"
						>{occ.title}
						{#if occ.field}<span class="f">{occ.field}</span>{:else}<span class="f"
								>{t('entry.inBody')}</span
							>{/if}</span
					>
				{/each}
				{#if group.occurrences.length > 3}
					<span class="srcchip">+{group.occurrences.length - 3}</span>
				{/if}
			</div>
		</div>
		<button type="button" class="disamb-btn" onclick={() => ondisambiguate(group)}
			>{t('workshop.disambiguate')}</button
		>
	</div>
	<div class="acand">
		{t('workshop.candidatesLabel')}
		{#each candidates as candidate, i (candidate.slug)}
			{i > 0 ? ' · ' : ''}<b>{candidate.title}</b> ({typeLabel(candidate.type)})
		{/each}
	</div>
</div>

<style>
	.arow {
		padding: 14px 16px;
		border: 1px solid var(--warning-line);
		border-radius: var(--radius);
		background: var(--surface);
		margin-bottom: 8px;
	}
	.atop {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.aicon {
		width: 34px;
		height: 34px;
		border-radius: 8px;
		flex: none;
		display: grid;
		place-items: center;
		background: var(--warning-soft);
		color: var(--warning);
		border: 1px solid var(--warning-line);
	}
	.abody {
		flex: 1;
		min-width: 0;
	}
	.alink {
		font-family: var(--font-mono);
		font-size: 14px;
		color: var(--warning);
	}
	.smeta {
		color: var(--muted);
		font-size: 12.5px;
		margin-top: 3px;
		display: flex;
		align-items: center;
		gap: 7px;
		flex-wrap: wrap;
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
	.srcchip .f {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--faint);
	}
	.disamb-btn {
		height: 27px;
		padding: 0 10px;
		font-size: 12.5px;
		font: inherit;
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		border: 1px solid var(--border);
		color: var(--text);
		cursor: pointer;
		flex: none;
	}
	.disamb-btn:hover {
		border-color: var(--border-strong);
	}
	.acand {
		color: var(--muted);
		font-size: 12.5px;
		margin-top: 8px;
		padding-left: 46px;
	}
	.acand b {
		color: var(--text);
		font-weight: 600;
	}
</style>
