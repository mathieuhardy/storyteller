<script lang="ts">
	// File tree sidebar for book mode. Shows configurable folders.
	import { onMount } from 'svelte';
	import { listFiles, type FileEntry } from '$api/client';
	import { t } from '$i18n/index.svelte';
	import Icon from '$components/Icon.svelte';
	import Popover from '$components/Popover.svelte';
	import FileTreeNode from './FileTreeNode.svelte';

	let {
		folders = ['chapters'],
		currentFile = null,
		onSelect,
		onFoldersChange
	}: {
		folders?: string[];
		currentFile?: string | null;
		onSelect: (path: string) => void;
		onFoldersChange?: (folders: string[]) => void;
	} = $props();

	let rootEntries = $state<FileEntry[]>([]);
	let folderEntries = $state<Map<string, FileEntry[]>>(new Map());
	let isLoading = $state(true);
	let folderPickerOpen = $state(false);
	let availableFolders = $state<string[]>([]);

	onMount(async () => {
		await loadRoot();
	});

	async function loadRoot() {
		isLoading = true;
		try {
			rootEntries = await listFiles();
			// Extract available folders from root
			availableFolders = rootEntries.filter((e) => e.is_dir).map((e) => e.name);

			// Load contents of selected folders
			for (const folder of folders) {
				if (availableFolders.includes(folder)) {
					const entries = await listFiles(folder);
					folderEntries.set(folder, entries);
				}
			}
			folderEntries = new Map(folderEntries);
		} catch {
			rootEntries = [];
		} finally {
			isLoading = false;
		}
	}

	async function loadChildren(path: string): Promise<FileEntry[]> {
		return await listFiles(path);
	}

	function toggleFolder(folder: string) {
		const newFolders = folders.includes(folder)
			? folders.filter((f) => f !== folder)
			: [...folders, folder];
		onFoldersChange?.(newFolders);

		// Load folder contents if newly added
		if (!folders.includes(folder)) {
			listFiles(folder).then((entries) => {
				folderEntries.set(folder, entries);
				folderEntries = new Map(folderEntries);
			});
		}
	}

	// Get entries to display based on selected folders
	const displayEntries = $derived.by(() => {
		const entries: FileEntry[] = [];
		// If no folders selected, show all root entries
		if (folders.length === 0) {
			return rootEntries;
		}
		// Otherwise, show only selected folders as top-level
		for (const folder of folders) {
			const folderEntry = rootEntries.find((e) => e.is_dir && e.name === folder);
			if (folderEntry) {
				entries.push(folderEntry);
			}
		}
		return entries;
	});
</script>

<div class="file-tree">
	<div class="header">
		<span class="title">{t('book.files')}</span>
		<Popover bind:open={folderPickerOpen} align="left">
			{#snippet trigger({ toggle })}
				<button class="folder-btn" onclick={toggle} title={t('book.selectFolders')}>
					<Icon name="folder" size={14} />
				</button>
			{/snippet}
			<div class="folder-picker">
				<p class="picker-label">{t('book.selectFolders')}</p>
				{#each availableFolders as folder (folder)}
					<label class="folder-option">
						<input
							type="checkbox"
							checked={folders.includes(folder)}
							onchange={() => toggleFolder(folder)}
						/>
						<Icon name="folder" size={14} />
						<span>{folder}</span>
					</label>
				{/each}
				{#if availableFolders.length === 0}
					<p class="empty">{t('book.noFolders')}</p>
				{/if}
			</div>
		</Popover>
	</div>

	<div class="tree-content">
		{#if isLoading}
			<p class="loading">{t('book.loading')}</p>
		{:else if displayEntries.length === 0}
			<p class="empty">{t('book.noFiles')}</p>
		{:else}
			{#each displayEntries as entry (entry.path)}
				<FileTreeNode
					{entry}
					depth={0}
					isSelected={currentFile === entry.path}
					onSelect={(path) => onSelect(path)}
					{loadChildren}
				/>
			{/each}
		{/if}
	</div>
</div>

<style>
	.file-tree {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px;
		border-bottom: 1px solid var(--border);
	}

	.title {
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
	}

	.folder-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border: 0;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--muted);
		cursor: pointer;
	}

	.folder-btn:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.folder-picker {
		padding: 8px;
		min-width: 160px;
	}

	.picker-label {
		margin: 0 0 8px;
		padding: 0 4px;
		font-size: 10.5px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
	}

	.folder-option {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 4px;
		font-size: 13px;
		cursor: pointer;
	}

	.folder-option input {
		margin: 0;
	}

	.tree-content {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
	}

	.loading,
	.empty {
		padding: 12px;
		font-size: 13px;
		color: var(--muted);
		text-align: center;
	}
</style>
