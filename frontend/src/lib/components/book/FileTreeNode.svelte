<script lang="ts">
	// A single node in the file tree (file or folder with children).
	import Icon from '$components/Icon.svelte';
	import FileTreeNode from './FileTreeNode.svelte';
	import type { FileEntry } from '$api/client';

	let {
		entry,
		depth = 0,
		isSelected = false,
		onSelect,
		loadChildren
	}: {
		entry: FileEntry;
		depth?: number;
		isSelected?: boolean;
		onSelect: (path: string) => void;
		loadChildren: (path: string) => Promise<FileEntry[]>;
	} = $props();

	let expanded = $state(false);
	let children = $state<FileEntry[]>([]);
	let isLoading = $state(false);

	async function toggle() {
		if (!entry.is_dir) {
			onSelect(entry.path);
			return;
		}

		expanded = !expanded;
		if (expanded && children.length === 0) {
			isLoading = true;
			try {
				children = await loadChildren(entry.path);
			} catch {
				children = [];
			} finally {
				isLoading = false;
			}
		}
	}
</script>

<div class="node" style:--depth={depth}>
	<button
		class="node-btn"
		class:dir={entry.is_dir}
		class:selected={isSelected}
		onclick={toggle}
		title={entry.path}
	>
		{#if entry.is_dir}
			<span class="chevron" class:expanded>
				<Icon name="chevron-right" size={12} />
			</span>
			<Icon name="folder" size={14} />
		{:else}
			<span class="spacer"></span>
			<Icon name="file" size={14} />
		{/if}
		<span class="name">{entry.name}</span>
		{#if isLoading}
			<span class="loading">...</span>
		{/if}
	</button>

	{#if expanded && children.length > 0}
		<div class="children">
			{#each children as child (child.path)}
				<FileTreeNode
					entry={child}
					depth={depth + 1}
					isSelected={false}
					{onSelect}
					{loadChildren}
				/>
			{/each}
		</div>
	{/if}
</div>

<style>
	.node {
		display: flex;
		flex-direction: column;
	}

	.node-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 5px 8px;
		padding-left: calc(8px + var(--depth) * 16px);
		border: 0;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text);
		font: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}

	.node-btn:hover {
		background: var(--surface-2);
	}

	.node-btn.selected {
		background: var(--accent-soft);
		color: var(--accent);
	}

	.node-btn.dir {
		font-weight: 500;
	}

	.chevron {
		display: inline-flex;
		transition: transform 0.15s ease;
		color: var(--faint);
	}

	.chevron.expanded {
		transform: rotate(90deg);
	}

	.spacer {
		width: 12px;
	}

	.name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.loading {
		font-size: 11px;
		color: var(--faint);
	}

	.children {
		display: flex;
		flex-direction: column;
	}
</style>
