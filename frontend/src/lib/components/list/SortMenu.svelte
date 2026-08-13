<script lang="ts">
	// Toolbar "Sort" control (docs/ui/screens.md §3): a popover listing sortable
	// fields plus ascending/descending, mirroring the `sort=`/`-sort=` URL param.
	// Column headers in the table offer the same thing inline; this is the
	// toolbar-level entry point for fields that have no visible column.
	import Popover from '$components/Popover.svelte';
	import Icon from '$components/Icon.svelte';
	import { t } from '$i18n/index.svelte';

	let {
		options,
		field,
		descending,
		onchange
	}: {
		options: { key: string; label: string }[];
		field: string;
		descending: boolean;
		onchange: (field: string, descending: boolean) => void;
	} = $props();

	const current = $derived(options.find((o) => o.key === field) ?? options[0]);

	let open = $state(false);

	function pickField(key: string) {
		onchange(key, descending);
		open = false;
	}

	function pickDirection(next: boolean) {
		onchange(field, next);
		open = false;
	}
</script>

<Popover bind:open align="left">
	{#snippet trigger({ toggle })}
		<button class="selectish" onclick={toggle}>
			<span class="k">{t('list.sort')}:</span>
			<span class="v">{current?.label}</span>
			<Icon name="chevron-down" size={12} />
		</button>
	{/snippet}
	<p class="mlabel">{t('list.sortBy')}</p>
	{#each options as option (option.key)}
		<button class="mi" onclick={() => pickField(option.key)}>
			<span class="mtext">{option.label}</span>
			{#if option.key === field}<span class="ck">✓</span>{/if}
		</button>
	{/each}
	<p class="mlabel">{t('list.order')}</p>
	<button class="mi" onclick={() => pickDirection(false)}>
		<span class="mtext">{t('list.ascending')}</span>
		{#if !descending}<span class="ck">✓</span>{/if}
	</button>
	<button class="mi" onclick={() => pickDirection(true)}>
		<span class="mtext">{t('list.descending')}</span>
		{#if descending}<span class="ck">✓</span>{/if}
	</button>
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
	.k {
		color: var(--faint);
	}
	.v {
		font-weight: 500;
	}
	.mlabel {
		margin: 0;
		padding: 7px 9px 4px;
		font-size: 10.5px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--faint);
	}
	.mi {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 9px;
		padding: 7px 9px;
		border: 0;
		background: transparent;
		border-radius: 5px;
		font-size: 13px;
		font: inherit;
		color: var(--text);
		text-align: left;
		cursor: pointer;
	}
	.mi:hover {
		background: var(--surface-2);
	}
	.mtext {
		flex: 1;
	}
	.ck {
		color: var(--accent);
	}
</style>
