<script lang="ts">
	// "Add a filter" control (docs/ui/screens.md §3 / components.md §4): pick a
	// field, then supply a value — an equality filter appended as `<field>=value`
	// to the URL (`tag=` for the free-form tag dimension). Enum/boolean fields
	// skip the value step and offer their possible values directly.
	import Popover from '$components/Popover.svelte';
	import { t } from '$i18n/index.svelte';
	import type { FieldKind } from '$api/types';

	interface FilterableField {
		name: string;
		label: string;
		kind: FieldKind;
		enum_values?: string[];
	}

	let {
		fields,
		onadd
	}: {
		fields: FilterableField[];
		onadd: (field: string, value: string) => void;
	} = $props();

	let open = $state(false);
	let step = $state<'field' | 'value'>('field');
	let selected = $state<FilterableField | null>(null);
	let value = $state('');

	// The panel always restarts at the field-picker step next time it opens.
	$effect(() => {
		if (!open) {
			step = 'field';
			selected = null;
			value = '';
		}
	});

	function pickField(field: FilterableField) {
		if (field.kind === 'boolean') {
			onadd(field.name, 'true');
			open = false;
			return;
		}
		selected = field;
		step = 'value';
	}

	function pickEnumValue(v: string) {
		if (!selected) return;
		onadd(selected.name, v);
		open = false;
	}

	function submitValue() {
		if (!selected || !value.trim()) return;
		onadd(selected.name, value.trim());
		open = false;
	}

	// Focuses the value input as soon as it mounts, without the `autofocus`
	// attribute (flagged for stealing focus outside the user's control on a
	// page load — here it is scoped to a popover the user just opened).
	function focusOnMount(node: HTMLInputElement) {
		node.focus();
	}
</script>

<Popover bind:open align="left">
	{#snippet trigger({ toggle })}
		<button class="filterbtn" onclick={toggle}>
			<span class="plus">+</span>
			{t('list.addFilter')}
		</button>
	{/snippet}
	{#if step === 'field'}
		<p class="mlabel">{t('list.filterByField')}</p>
		{#each fields as field (field.name)}
			<button class="mi" onclick={() => pickField(field)}>{field.label}</button>
		{/each}
	{:else if selected?.kind === 'enum'}
		<p class="mlabel">{selected.label}</p>
		{#each selected.enum_values ?? [] as ev (ev)}
			<button class="mi" onclick={() => pickEnumValue(ev)}>{ev}</button>
		{/each}
	{:else if selected}
		<p class="mlabel">{selected.label}</p>
		<form class="value-form" onsubmit={(e) => (e.preventDefault(), submitValue())}>
			<input
				class="value-input"
				type="text"
				placeholder={t('list.filterValue')}
				bind:value
				use:focusOnMount
			/>
		</form>
	{/if}
</Popover>

<style>
	.filterbtn {
		height: 28px;
		padding: 0 10px;
		display: inline-flex;
		align-items: center;
		gap: 7px;
		background: var(--surface);
		border: 1px dashed var(--border);
		border-radius: var(--radius-sm);
		color: var(--muted);
		font-size: 12.5px;
		font: inherit;
		cursor: pointer;
	}
	.filterbtn:hover {
		border-color: var(--border-strong);
	}
	.plus {
		color: var(--faint);
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
		display: block;
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
	.value-form {
		padding: 4px 6px 6px;
	}
	.value-input {
		width: 100%;
		height: 30px;
		padding: 0 9px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--text);
		font: inherit;
		font-size: 13px;
	}
	.value-input:focus {
		outline: none;
		border-color: var(--accent-line);
	}
</style>
