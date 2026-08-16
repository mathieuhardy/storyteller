<script lang="ts">
	// Entry detail screen (docs/ui/screens.md §2): displays a full entry with
	// header, diagnostic banner, attributes (frontmatter), and body. The rail
	// panel shows backlinks, outgoing links, and metadata.
	import { onMount, onDestroy } from 'svelte';
	import type { PageData } from './$types';
	import { t, typeLabel } from '$i18n/index.svelte';
	import { setRailEntry, clearRailEntry } from '$stores/rail.svelte';
	import EntryHeader from '$components/entry/EntryHeader.svelte';
	import DiagnosticBanner from '$components/entry/DiagnosticBanner.svelte';
	import AttributesSection from '$components/entry/AttributesSection.svelte';
	import BodySection from '$components/entry/BodySection.svelte';

	let { data }: { data: PageData } = $props();

	const entry = $derived(data.entry);
	const links = $derived(data.links);
	const schema = $derived(data.schema);

	// Populate the rail with entry data
	$effect(() => {
		setRailEntry(entry, links);
	});

	onDestroy(() => {
		clearRailEntry();
	});
</script>

<div class="entry-screen">
	<EntryHeader {entry} />

	{#if entry.errors.length > 0}
		<DiagnosticBanner errors={entry.errors} />
	{/if}

	<AttributesSection frontmatter={entry.frontmatter} fields={schema.fields} {links} />

	<BodySection body={entry.body} {links} />
</div>

<style>
	.entry-screen {
		max-width: 780px;
		margin: 0 auto;
		padding: 24px 26px 60px;
	}
</style>
