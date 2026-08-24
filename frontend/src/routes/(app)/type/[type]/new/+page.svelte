<script lang="ts">
	import type { PageData } from './$types';
	import EntryEditor from '$components/editor/EntryEditor.svelte';

	let { data }: { data: PageData } = $props();
</script>

{#key data.type}
	<EntryEditor mode="create" type={data.type} schema={data.schema} />
{/key}
