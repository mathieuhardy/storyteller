<script lang="ts">
	// One frontmatter field control, generated from `FieldSchema.kind`
	// (docs/ui/components.md §4). The value shape depends on the kind:
	// text/number-or-text/image → string, number → number|null, boolean →
	// boolean, enum → string, list/list-or-text/link/link-list/image-list →
	// string[] (a single `link` is a 0-or-1-item array for a uniform contract).
	import type { FieldSchema } from '$api/types';
	import { t } from '$i18n/index.svelte';
	import TokenInput from './TokenInput.svelte';
	import LinkFieldEditor from './LinkFieldEditor.svelte';
	import ImageFieldEditor from './ImageFieldEditor.svelte';

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let {
		field,
		value = $bindable()
	}: {
		field: FieldSchema;
		value: any;
	} = $props();
</script>

{#if field.kind === 'text'}
	<textarea class="ctrl-textarea" bind:value rows="2"></textarea>
{:else if field.kind === 'number'}
	<input
		class="ctrl-input num"
		type="number"
		value={value ?? ''}
		oninput={(e) => (value = e.currentTarget.value === '' ? null : Number(e.currentTarget.value))}
	/>
{:else if field.kind === 'number-or-text'}
	<input class="ctrl-input" type="text" bind:value />
{:else if field.kind === 'boolean'}
	<button
		type="button"
		class="toggle"
		class:on={value === true}
		role="switch"
		aria-checked={value === true}
		aria-label={field.label}
		onclick={() => (value = !value)}
	>
		<span class="knob"></span>
	</button>
{:else if field.kind === 'enum'}
	<select class="ctrl-input" bind:value>
		{#if !field.required}
			<option value="">—</option>
		{/if}
		{#each field.enum_values ?? [] as option (option)}
			<option value={option}>{option}</option>
		{/each}
	</select>
{:else if field.kind === 'list' || field.kind === 'list-or-text' || field.kind === 'image-list'}
	<TokenInput bind:tokens={value} placeholder={t('editor.addToken')} />
{:else if field.kind === 'link'}
	<LinkFieldEditor
		bind:tokens={value}
		linkTargets={field.link_targets ?? []}
		multi={false}
		addLabel={t('editor.linkOne', { field: field.label })}
	/>
{:else if field.kind === 'link-list'}
	<LinkFieldEditor
		bind:tokens={value}
		linkTargets={field.link_targets ?? []}
		multi={true}
		addLabel={t('editor.linkMany', { field: field.label })}
	/>
{:else if field.kind === 'image'}
	<ImageFieldEditor bind:paths={value} multi={false} />
{:else if field.kind === 'image-list'}
	<ImageFieldEditor bind:paths={value} multi={true} />
{:else}
	<input class="ctrl-input" type="text" bind:value />
{/if}

<style>
	.ctrl-input,
	.ctrl-textarea {
		width: 100%;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		color: var(--text);
		font-family: inherit;
		font-size: 13.5px;
		padding: 7px 10px;
		outline: none;
	}
	.ctrl-input:focus,
	.ctrl-textarea:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-soft);
	}
	.ctrl-textarea {
		resize: vertical;
		min-height: 34px;
		line-height: 1.5;
		font-family: inherit;
	}
	.ctrl-input.num {
		width: 110px;
		font-family: var(--font-mono);
	}
	select.ctrl-input {
		cursor: pointer;
	}
	.toggle {
		width: 40px;
		height: 23px;
		border-radius: 20px;
		background: var(--surface-3);
		border: 1px solid var(--border);
		position: relative;
		cursor: pointer;
		padding: 0;
	}
	.toggle .knob {
		position: absolute;
		top: 1.5px;
		left: 2px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: #fff;
		box-shadow: var(--shadow);
		transition: left 0.16s ease;
	}
	.toggle.on {
		background: var(--accent);
		border-color: var(--accent);
	}
	.toggle.on .knob {
		left: 19px;
	}
</style>
