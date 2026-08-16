<script lang="ts">
	// Link token (docs/ui/components.md §3): renders a wikilink as a clickable
	// token with visual state (resolved, stub, ambiguous). Clicking a resolved
	// link navigates; clicking a stub opens creation; ambiguous opens disambiguation.
	import type { OutgoingLink } from '$api/types';

	let {
		target,
		resolution
	}: {
		target: string;
		resolution?: OutgoingLink;
	} = $props();

	const state = $derived(resolution?.resolution ?? 'stub');
	const slug = $derived(resolution?.target_slug);

	const href = $derived(
		state === 'resolved' && slug ? `/entry/${slug}` : undefined
	);

	// Extract display label from target (handle [[slug|Label]] format)
	const label = $derived(() => {
		const pipeIndex = target.indexOf('|');
		return pipeIndex > 0 ? target.slice(pipeIndex + 1) : target;
	});
</script>

{#if href}
	<a class="link-token resolved" {href}>
		{label()}
	</a>
{:else if state === 'stub'}
	<button class="link-token stub" type="button" title="Create this entry">
		{label()}
		<span class="indicator">+</span>
	</button>
{:else}
	<button class="link-token ambiguous" type="button" title="Resolve ambiguity">
		{label()}
		<span class="indicator">?</span>
	</button>
{/if}

<style>
	.link-token {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		padding: 3px 10px;
		border-radius: 4px;
		font-size: 13px;
		font-family: inherit;
		text-decoration: none;
		cursor: pointer;
		transition:
			background 0.1s ease,
			color 0.1s ease;
	}

	.resolved {
		background: var(--accent-soft);
		color: var(--accent);
		border: none;
	}
	.resolved:hover {
		background: var(--accent);
		color: white;
	}

	.stub {
		background: var(--stub-soft);
		color: var(--stub);
		border: 1px dashed var(--stub-line);
	}
	.stub:hover {
		background: var(--stub);
		color: white;
		border-style: solid;
	}

	.ambiguous {
		background: var(--warning-soft);
		color: var(--warning);
		border: 1px dashed var(--warning-line);
	}
	.ambiguous:hover {
		background: var(--warning);
		color: white;
		border-style: solid;
	}

	.indicator {
		font-size: 10px;
		font-weight: 700;
		margin-left: 2px;
	}
</style>
