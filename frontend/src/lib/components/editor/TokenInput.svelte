<script lang="ts">
	// Token field (docs/ui/components.md §4): editable chips for aliases, tags,
	// and `list`/`list-or-text` fields. Enter or comma commits the pending text
	// as a new token; the × on a chip removes it.
	let {
		tokens = $bindable([]),
		placeholder = ''
	}: {
		tokens?: string[];
		placeholder?: string;
	} = $props();

	let pending = $state('');

	function commit() {
		const value = pending.trim();
		if (value && !tokens.includes(value)) tokens = [...tokens, value];
		pending = '';
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ',') {
			event.preventDefault();
			commit();
		} else if (event.key === 'Backspace' && pending === '' && tokens.length > 0) {
			tokens = tokens.slice(0, -1);
		}
	}

	function remove(index: number) {
		tokens = tokens.filter((_, i) => i !== index);
	}
</script>

<div class="token-field">
	{#each tokens as tokenValue, i (i)}
		<span class="token">
			{tokenValue}
			<button type="button" class="remove" onclick={() => remove(i)} aria-label="remove">✕</button
			>
		</span>
	{/each}
	<input
		class="token-input"
		type="text"
		bind:value={pending}
		{placeholder}
		onkeydown={onKeydown}
		onblur={commit}
	/>
</div>

<style>
	.token-field {
		flex: 1;
		display: flex;
		flex-wrap: wrap;
		gap: 5px;
		align-items: center;
		min-height: 34px;
		padding: 4px 6px;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
	}
	.token-field:focus-within {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
	}
	.token {
		height: 22px;
		padding: 0 4px 0 9px;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		border-radius: 6px;
		font-size: 12px;
		background: var(--surface-2);
		border: 1px solid var(--border);
		color: var(--text);
	}
	.remove {
		border: 0;
		background: transparent;
		color: var(--faint);
		cursor: pointer;
		font-size: 11px;
		padding: 0;
		line-height: 1;
	}
	.remove:hover {
		color: var(--danger);
	}
	.token-input {
		border: 0;
		background: transparent;
		outline: none;
		color: var(--text);
		font-family: inherit;
		font-size: 13px;
		min-width: 90px;
		flex: 1;
		height: 24px;
	}
</style>
