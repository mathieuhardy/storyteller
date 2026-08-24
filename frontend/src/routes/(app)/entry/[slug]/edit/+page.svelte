<script lang="ts">
	import type { PageData } from './$types';
	import EntryEditor from '$components/editor/EntryEditor.svelte';

	let { data }: { data: PageData } = $props();
</script>

{#key data.entry.slug}
	<EntryEditor mode="edit" type={data.entry.type} schema={data.schema} entry={data.entry} links={data.links} />
{/key}
