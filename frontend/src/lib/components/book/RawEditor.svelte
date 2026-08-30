<script lang="ts">
	// Raw markdown editor textarea for book mode. Full-page monospace editing.
	let {
		content = $bindable(''),
		showLineBreaks = false,
		lineHeight = 'normal' as 'compact' | 'normal' | 'spacious',
		disabled = false,
		placeholder = ''
	}: {
		content?: string;
		showLineBreaks?: boolean;
		lineHeight?: 'compact' | 'normal' | 'spacious';
		disabled?: boolean;
		placeholder?: string;
	} = $props();

	const lineHeightValues = {
		compact: '1.4',
		normal: '1.6',
		spacious: '2.0'
	};

	// For showing line breaks, create an overlay that shows only pilcrow marks
	// at line endings. We replace each character with a space except newlines
	// which get a pilcrow before them.
	const breaksOverlay = $derived(
		showLineBreaks
			? content.replace(/[^\n]/g, ' ').replace(/\n/g, '\u00b6\n')
			: ''
	);
</script>

<div
	class="editor-wrap"
	style:--line-height={lineHeightValues[lineHeight]}
>
	{#if showLineBreaks}
		<div class="breaks-overlay" aria-hidden="true">{breaksOverlay}</div>
	{/if}
	<textarea
		class="editor mono"
		bind:value={content}
		{disabled}
		{placeholder}
		spellcheck="false"
	></textarea>
</div>

<style>
	.editor-wrap {
		position: relative;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.editor {
		width: 100%;
		height: 100%;
		padding: 24px 32px;
		border: 0;
		background: var(--bg);
		color: var(--text);
		font-size: 14px;
		line-height: var(--line-height, 1.6);
		resize: none;
		outline: none;
	}

	.editor::placeholder {
		color: var(--faint);
	}

	.editor:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Line breaks overlay - shows only pilcrow marks at line endings */
	.breaks-overlay {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		padding: 24px 32px;
		font-family: var(--font-mono);
		font-size: 14px;
		line-height: var(--line-height, 1.6);
		color: var(--accent);
		opacity: 0.5;
		white-space: pre-wrap;
		word-wrap: break-word;
		pointer-events: none;
		overflow: hidden;
	}

	@media (max-width: 640px) {
		.editor,
		.breaks-overlay {
			padding: 16px;
		}
	}
</style>
