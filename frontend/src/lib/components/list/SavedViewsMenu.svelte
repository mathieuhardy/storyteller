<script lang="ts">
	// Saved views (docs/api.md §4, M5): name the current filter/sort
	// combination, reload it later. Persisted client-side (see
	// `stores/savedViews.svelte.ts`) — a view is a UI preference, not project
	// data, so nothing here ever touches the API.
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';
	import { t } from '$i18n/index.svelte';
	import { viewsFor, saveView, deleteView, type SavedView } from '$stores/savedViews.svelte';

	let {
		type,
		currentQuery,
		onapply
	}: {
		type: string;
		/** The current query string (no leading `?`), to save as-is. */
		currentQuery: string;
		onapply: (query: string) => void;
	} = $props();

	let open = $state(false);
	let nameInput = $state('');

	const views = $derived.by((): SavedView[] => {
		// `open` is read only to make this re-run each time the popover opens,
		// since the localStorage-backed store isn't itself reactive state.
		void open;
		return viewsFor(type);
	});

	function apply(view: SavedView) {
		onapply(view.query);
		open = false;
	}

	function save() {
		const name = nameInput.trim();
		if (!name) return;
		saveView(name, type, currentQuery);
		nameInput = '';
	}

	function remove(id: string, event: MouseEvent) {
		event.stopPropagation();
		deleteView(id);
	}
</script>

<Popover bind:open align="left">
	{#snippet trigger({ toggle })}
		<button class="selectish" onclick={toggle}>
			<Icon name="square-plus" size={13} />
			<span>{t('list.views')}</span>
			{#if views.length > 0}<span class="count">{views.length}</span>{/if}
		</button>
	{/snippet}
	<div class="panel">
		<p class="mlabel">{t('list.savedViews')}</p>
		{#if views.length === 0}
			<p class="empty">{t('list.noSavedViews')}</p>
		{:else}
			{#each views as view (view.id)}
				<div class="mi">
					<button type="button" class="mtext" onclick={() => apply(view)}>{view.name}</button>
					<button
						type="button"
						class="del"
						onclick={(e) => remove(view.id, e)}
						aria-label={t('list.deleteView', { name: view.name })}>✕</button
					>
				</div>
			{/each}
		{/if}
		<form class="saveform" onsubmit={(e) => (e.preventDefault(), save())}>
			<input
				class="nameinput"
				type="text"
				placeholder={t('list.saveViewPlaceholder')}
				bind:value={nameInput}
			/>
			<button class="savebtn" type="submit" disabled={!nameInput.trim()}>{t('action.save')}</button>
		</form>
	</div>
</Popover>

<style>
	.selectish {
		height: 28px;
		padding: 0 10px;
		display: inline-flex;
		align-items: center;
		gap: 7px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		color: var(--text);
		font-size: 12.5px;
		font: inherit;
		cursor: pointer;
	}
	.selectish:hover {
		border-color: var(--border-strong);
	}
	.count {
		font-family: var(--font-mono);
		font-size: 10.5px;
		color: var(--muted);
		background: var(--surface-2);
		border-radius: 20px;
		padding: 0 5px;
	}
	.panel {
		width: 220px;
	}
	.mlabel {
		margin: 0;
		padding: 7px 9px 4px;
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
	}
	.empty {
		margin: 0;
		padding: 4px 9px 8px;
		font-size: 12px;
		color: var(--faint);
	}
	.mi {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 4px;
		border-radius: 5px;
	}
	.mi:hover {
		background: var(--surface-2);
	}
	.mi:hover .del {
		opacity: 1;
	}
	.mtext {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding: 7px 0 7px 9px;
		border: 0;
		background: transparent;
		font-size: 13px;
		font: inherit;
		color: var(--text);
		text-align: left;
		cursor: pointer;
	}
	.del {
		opacity: 0;
		flex: none;
		border: 0;
		background: transparent;
		color: var(--faint);
		font-size: 11px;
		padding: 6px 9px 6px 2px;
		cursor: pointer;
	}
	.del:hover {
		color: var(--danger);
	}
	.saveform {
		display: flex;
		gap: 6px;
		padding: 8px 6px 4px;
		border-top: 1px solid var(--border);
		margin-top: 4px;
	}
	.nameinput {
		flex: 1;
		height: 28px;
		padding: 0 8px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--text);
		font: inherit;
		font-size: 12.5px;
	}
	.nameinput:focus {
		outline: none;
		border-color: var(--accent-line);
	}
	.savebtn {
		height: 28px;
		padding: 0 10px;
		border: 0;
		border-radius: var(--radius-sm);
		background: var(--accent);
		color: var(--accent-fg);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.savebtn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
