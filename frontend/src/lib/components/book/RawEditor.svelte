<script lang="ts">
	import { onMount } from 'svelte';
	import { UndoStack, type UndoState } from '$lib/utils/undoStack';
	import SearchBar from '$components/SearchBar.svelte';

	// Raw markdown editor textarea for book mode. Full-page monospace editing.
	let {
		content = $bindable(''),
		showLineBreaks = false,
		lineHeight = 'normal' as 'compact' | 'normal' | 'spacious',
		editorWidth = 'full' as 'medium' | 'wide' | 'full',
		disabled = false,
		placeholder = ''
	}: {
		content?: string;
		showLineBreaks?: boolean;
		lineHeight?: 'compact' | 'normal' | 'spacious';
		editorWidth?: 'medium' | 'wide' | 'full';
		disabled?: boolean;
		placeholder?: string;
	} = $props();

	// Undo/redo stack
	const undoStack = new UndoStack(100);
	let lastContent = '';
	let isUndoRedo = false;

	// Search state
	let searchVisible = $state(false);

	const lineHeightValues = {
		compact: '1.4',
		normal: '1.6',
		spacious: '2.0'
	};

	const editorWidthValues = {
		medium: '90ch',
		wide: '110ch',
		full: 'none'
	};

	let textareaEl: HTMLTextAreaElement | undefined = $state();
	let overlayEl: HTMLDivElement | undefined = $state();
	let mirrorEl: HTMLDivElement | undefined = $state();
	let cursorEl: HTMLDivElement | undefined = $state();

	// Custom cursor state
	let cursorVisible = $state(false);
	let cursorX = $state(0);
	let cursorY = $state(0);

	// Sync overlay scroll with textarea
	function onScroll() {
		if (overlayEl && textareaEl) {
			overlayEl.scrollTop = textareaEl.scrollTop;
			overlayEl.scrollLeft = textareaEl.scrollLeft;
		}
		updateCursorPosition();
	}

	// Update custom cursor position using mirror element technique
	function updateCursorPosition() {
		if (!textareaEl || !mirrorEl) return;

		const selStart = textareaEl.selectionStart;
		const selEnd = textareaEl.selectionEnd;

		// Hide cursor if there's a selection or textarea not focused
		if (selStart !== selEnd || document.activeElement !== textareaEl) {
			cursorVisible = false;
			return;
		}

		cursorVisible = true;

		// Get text before cursor
		const textBefore = textareaEl.value.substring(0, selStart);

		// Update mirror content with a marker span at cursor position
		mirrorEl.innerHTML = '';
		const textNode = document.createTextNode(textBefore);
		const marker = document.createElement('span');
		marker.id = 'cursor-marker';
		marker.textContent = '\u200b'; // Zero-width space to ensure proper line positioning
		mirrorEl.appendChild(textNode);
		mirrorEl.appendChild(marker);

		// Sync mirror scroll with textarea
		mirrorEl.scrollTop = textareaEl.scrollTop;
		mirrorEl.scrollLeft = textareaEl.scrollLeft;

		// Get marker position relative to mirror
		const markerRect = marker.getBoundingClientRect();
		const mirrorRect = mirrorEl.getBoundingClientRect();

		cursorX = markerRect.left - mirrorRect.left;
		cursorY = markerRect.top - mirrorRect.top;
	}

	function onFocus() {
		updateCursorPosition();
	}

	function onBlur() {
		cursorVisible = false;
	}

	function onInput() {
		updateCursorPosition();
		// Push to undo stack after changes (debounced by checking if content actually changed)
		if (!isUndoRedo && textareaEl && content !== lastContent) {
			undoStack.push({
				content,
				selectionStart: textareaEl.selectionStart,
				selectionEnd: textareaEl.selectionEnd
			});
			lastContent = content;
		}
	}

	function onKeydown(e: KeyboardEvent) {
		const isMod = e.ctrlKey || e.metaKey;

		// Ctrl+F: open search
		if (isMod && e.key === 'f') {
			e.preventDefault();
			searchVisible = true;
			return;
		}

		// Ctrl+Z: undo
		if (isMod && e.key === 'z' && !e.shiftKey) {
			e.preventDefault();
			const state = undoStack.undo();
			if (state) {
				applyUndoState(state);
			}
			return;
		}

		// Ctrl+Shift+Z or Ctrl+Y: redo
		if ((isMod && e.key === 'z' && e.shiftKey) || (isMod && e.key === 'y')) {
			e.preventDefault();
			const state = undoStack.redo();
			if (state) {
				applyUndoState(state);
			}
			return;
		}

		// Defer to allow selection to update
		requestAnimationFrame(updateCursorPosition);
	}

	function applyUndoState(state: UndoState) {
		isUndoRedo = true;
		content = state.content;
		lastContent = state.content;
		queueMicrotask(() => {
			textareaEl?.setSelectionRange(state.selectionStart, state.selectionEnd);
			updateCursorPosition();
			isUndoRedo = false;
		});
	}

	function handleSearchNavigate(_index: number, start: number, end: number) {
		if (!textareaEl) return;
		textareaEl.focus();
		textareaEl.setSelectionRange(start, end);
		// Scroll the selection into view
		// We need to calculate scroll position based on character offset
		const textBefore = content.substring(0, start);
		const lines = textBefore.split('\n');
		const lineNumber = lines.length - 1;
		const lineHeightPx = parseFloat(getComputedStyle(textareaEl).lineHeight) || 22;
		const targetScroll = lineNumber * lineHeightPx - textareaEl.clientHeight / 2;
		textareaEl.scrollTop = Math.max(0, targetScroll);
		updateCursorPosition();
	}

	function onClick() {
		requestAnimationFrame(updateCursorPosition);
	}

	onMount(() => {
		if (textareaEl && document.activeElement === textareaEl) {
			updateCursorPosition();
		}
		// Initialize undo stack with initial content
		if (content) {
			undoStack.init({ content, selectionStart: 0, selectionEnd: 0 });
			lastContent = content;
		}
	});

	// Re-init undo stack when content is loaded externally (file change)
	$effect(() => {
		if (content !== lastContent && !isUndoRedo) {
			// Content changed externally (e.g., file loaded), reinit the stack
			undoStack.init({ content, selectionStart: 0, selectionEnd: 0 });
			lastContent = content;
		}
	});

	// For showing line breaks, create an overlay with only pilcrow marks.
	// Replace all non-newline characters with spaces to maintain positioning,
	// then add pilcrow before each newline.
	const breaksOverlay = $derived(
		showLineBreaks ? content.replace(/[^\n]/g, ' ').replace(/\n/g, '\u00b6\n') : ''
	);
</script>

<div
	class="editor-outer"
	style:--line-height={lineHeightValues[lineHeight]}
	style:--max-width={editorWidthValues[editorWidth]}
>
	<SearchBar {content} bind:visible={searchVisible} onNavigate={handleSearchNavigate} />
	<div class="editor-wrap">
		<textarea
			class="editor"
			bind:this={textareaEl}
			bind:value={content}
			onscroll={onScroll}
			onfocus={onFocus}
			onblur={onBlur}
			oninput={onInput}
			onkeydown={onKeydown}
			onkeyup={updateCursorPosition}
			onclick={onClick}
			onselect={updateCursorPosition}
			{disabled}
			{placeholder}
			spellcheck="false"
		></textarea>
		<!-- Mirror element for cursor position calculation -->
		<div class="cursor-mirror" bind:this={mirrorEl} aria-hidden="true"></div>
		<!-- Custom cursor -->
		{#if cursorVisible}
			<div
				class="custom-cursor"
				bind:this={cursorEl}
				style:left="{cursorX}px"
				style:top="{cursorY}px"
			></div>
		{/if}
		{#if showLineBreaks}
			<div class="breaks-overlay" bind:this={overlayEl} aria-hidden="true">{breaksOverlay}</div>
		{/if}
	</div>
</div>

<style>
	.editor-outer {
		flex: 1;
		min-height: 0;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.editor-wrap {
		position: relative;
		width: 100%;
		max-width: var(--max-width, none);
		height: 100%;
		margin: 0 auto;
		flex: 1;
		min-height: 0;
	}

	.editor {
		position: relative;
		width: 100%;
		height: 100%;
		padding: 24px 32px;
		border: 0;
		background: var(--bg);
		color: var(--text);
		caret-color: transparent; /* Hide native cursor, we use custom one */
		font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'Consolas', monospace;
		font-size: 14px;
		line-height: var(--line-height, 1.6);
		resize: none;
		outline: none;
		white-space: pre-wrap;
		word-wrap: break-word;
	}

	/* Mirror element - invisible copy of textarea for cursor positioning */
	.cursor-mirror {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		padding: 24px 32px;
		font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'Consolas', monospace;
		font-size: 14px;
		line-height: var(--line-height, 1.6);
		white-space: pre-wrap;
		word-wrap: break-word;
		overflow: hidden;
		visibility: hidden;
		pointer-events: none;
		box-sizing: border-box;
	}

	/* Custom blinking cursor - height matches font size, not line height */
	.custom-cursor {
		position: absolute;
		width: 1.5px;
		font-size: 14px;
		height: 1em; /* Matches font-size, not line-height */
		background: var(--muted);
		pointer-events: none;
		z-index: 2;
		animation: blink 1s step-end infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0; }
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
		font-family: ui-monospace, 'SF Mono', 'Cascadia Mono', 'Consolas', monospace;
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
		.breaks-overlay,
		.cursor-mirror {
			padding: 16px;
		}
	}
</style>
