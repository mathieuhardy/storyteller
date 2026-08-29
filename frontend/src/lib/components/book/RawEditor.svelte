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

	// For showing line breaks, we overlay a div with the same content but with
	// visible line break markers. This is purely visual.
	const contentWithBreaks = $derived(
		showLineBreaks
			? content.replace(/\n/g, '\u00b6\n') // Pilcrow at end of each line
			: ''
	);
</script>

<div
	class="editor-wrap"
	class:show-breaks={showLineBreaks}
	style:--line-height={lineHeightValues[lineHeight]}
>
	{#if showLineBreaks}
		<div class="breaks-overlay" aria-hidden="true">
			{contentWithBreaks}
		</div>
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
		background: var(--book-bg, var(--bg));
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

	/* Line breaks overlay */
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
		color: transparent;
		white-space: pre-wrap;
		word-wrap: break-word;
		pointer-events: none;
		overflow: hidden;
	}

	.show-breaks .editor {
		/* Make textarea text invisible so we see the overlay */
		color: transparent;
		caret-color: var(--text);
	}

	/* Style the pilcrow marks */
	.show-breaks .breaks-overlay {
		color: var(--faint);
		opacity: 0.4;
	}

	@media (max-width: 640px) {
		.editor,
		.breaks-overlay {
			padding: 16px;
		}
	}
</style>
