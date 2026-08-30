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

	let textareaEl: HTMLTextAreaElement | undefined = $state();
	let overlayEl: HTMLDivElement | undefined = $state();

	// Sync overlay scroll with textarea
	function onScroll() {
		if (overlayEl && textareaEl) {
			overlayEl.scrollTop = textareaEl.scrollTop;
			overlayEl.scrollLeft = textareaEl.scrollLeft;
		}
	}

	// For showing line breaks, create an overlay with only pilcrow marks.
	// Replace all non-newline characters with spaces to maintain positioning,
	// then add pilcrow before each newline.
	const breaksOverlay = $derived(
		showLineBreaks ? content.replace(/[^\n]/g, '\u00A0').replace(/\n/g, '\u00b6\n') : ''
	);
</script>

<div
	class="editor-wrap"
	style:--line-height={lineHeightValues[lineHeight]}
>
	<textarea
		class="editor mono"
		bind:this={textareaEl}
		bind:value={content}
		onscroll={onScroll}
		{disabled}
		{placeholder}
		spellcheck="false"
	></textarea>
	{#if showLineBreaks}
		<div class="breaks-overlay mono" bind:this={overlayEl} aria-hidden="true">{breaksOverlay}</div>
	{/if}
</div>

<style>
	.editor-wrap {
		position: relative;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.editor {
		position: relative;
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

	/* Line breaks overlay - shows only pilcrow marks positioned to match text */
	.breaks-overlay {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		padding: 24px 32px;
		font-size: 14px;
		line-height: var(--line-height, 1.6);
		color: var(--faint);
		white-space: pre-wrap;
		word-wrap: break-word;
		overflow: auto;
		pointer-events: none;
		box-sizing: border-box;
		z-index: 1;
		/* Hide scrollbars but allow scroll sync */
		scrollbar-width: none;
		-ms-overflow-style: none;
	}

	.breaks-overlay::-webkit-scrollbar {
		display: none;
	}

	@media (max-width: 640px) {
		.editor,
		.breaks-overlay {
			padding: 16px;
		}
	}
</style>
