<script lang="ts">
	// Links workshop / "chantier" screen (docs/ui/screens.md §5): stubs to
	// create and ambiguous links to resolve.
	import { t } from '$i18n/index.svelte';
	import StubRow from '$components/workshop/StubRow.svelte';
	import AmbiguousRow from '$components/workshop/AmbiguousRow.svelte';
	import CreateFromStubModal from '$components/workshop/CreateFromStubModal.svelte';
	import DisambiguateModal from '$components/workshop/DisambiguateModal.svelte';
	import type { Stub } from '$api/types';
	import type { AmbiguousGroup } from '$lib/workshop/link-rewrite';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	let pane = $state<'stubs' | 'ambiguous'>('stubs');

	const titlesBySlug = $derived(
		Object.fromEntries(data.entities.map((e) => [e.slug, e.title])) as Record<string, string>
	);

	let createOpen = $state(false);
	let createStub = $state<Stub | null>(null);

	function openCreate(stub: Stub) {
		createStub = stub;
		createOpen = true;
	}

	let disambOpen = $state(false);
	let disambGroup = $state<AmbiguousGroup | null>(null);

	function openDisambiguate(group: AmbiguousGroup) {
		disambGroup = group;
		disambOpen = true;
	}

	function candidatesFor(group: AmbiguousGroup) {
		return group.candidateSlugs
			.map((slug) => data.entities.find((e) => e.slug === slug))
			.filter((e) => e !== undefined);
	}
</script>

<div class="screen">
	<div class="head">
		<h1><span>⌦</span>{t('workshop.title')}</h1>
		<p class="sub">{t('workshop.subtitle')}</p>
	</div>

	<div class="seg">
		<button class:on={pane === 'stubs'} onclick={() => (pane = 'stubs')}>
			<span>{t('workshop.tabStubs')}</span><span class="b stub">{data.stubs.length}</span>
		</button>
		<button class:on={pane === 'ambiguous'} onclick={() => (pane = 'ambiguous')}>
			<span>{t('workshop.tabAmbiguous')}</span><span class="b amb">{data.ambiguous.length}</span>
		</button>
	</div>

	{#if pane === 'stubs'}
		{#if data.stubs.length === 0}
			<p class="empty">{t('dashboard.noStubs')}</p>
		{:else}
			{#each data.stubs as stub (stub.key)}
				<StubRow {stub} {titlesBySlug} oncreate={openCreate} />
			{/each}
		{/if}
	{:else if data.ambiguous.length === 0}
		<p class="empty">{t('workshop.noAmbiguous')}</p>
	{:else}
		{#each data.ambiguous as group (group.targetRaw)}
			<AmbiguousRow {group} candidates={candidatesFor(group)} ondisambiguate={openDisambiguate} />
		{/each}
	{/if}
</div>

<CreateFromStubModal bind:open={createOpen} stub={createStub} types={data.types} {titlesBySlug} />
<DisambiguateModal
	bind:open={disambOpen}
	group={disambGroup}
	candidates={disambGroup ? candidatesFor(disambGroup) : []}
	onresolved={() => {}}
/>

<style>
	.screen {
		max-width: 840px;
		margin: 0 auto;
		padding: 22px 26px 60px;
	}
	.head h1 {
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 21px;
		letter-spacing: -0.02em;
		margin: 0;
		font-weight: 660;
	}
	.head .sub {
		color: var(--muted);
		font-size: 13px;
		margin-top: 6px;
		max-width: 62ch;
	}
	.seg {
		display: inline-flex;
		background: var(--surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		padding: 2px;
		gap: 2px;
		margin: 20px 0 18px;
	}
	.seg button {
		height: 28px;
		padding: 0 13px;
		border: 0;
		background: transparent;
		color: var(--muted);
		border-radius: 5px;
		font-size: 13px;
		cursor: pointer;
		font: inherit;
		display: inline-flex;
		align-items: center;
		gap: 7px;
	}
	.seg button.on {
		background: var(--surface);
		color: var(--text);
		box-shadow: var(--shadow);
		font-weight: 500;
	}
	.seg button .b {
		font-family: var(--font-mono);
		font-size: 11px;
		border-radius: 20px;
		padding: 0 6px;
	}
	.seg button .b.stub {
		background: var(--stub-soft);
		color: var(--stub);
	}
	.seg button .b.amb {
		background: var(--warning-soft);
		color: var(--warning);
	}
	.empty {
		color: var(--muted);
		font-size: 13.5px;
		padding: 40px 0;
		text-align: center;
	}
</style>
