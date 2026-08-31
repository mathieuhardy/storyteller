// Undo/redo stack for text editors. Stores snapshots of content with cursor position.
// Max 100 entries to bound memory usage.

export interface UndoState {
	content: string;
	selectionStart: number;
	selectionEnd: number;
}

export class UndoStack {
	private stack: UndoState[] = [];
	private index = -1;
	private readonly maxSize: number;

	constructor(maxSize = 100) {
		this.maxSize = maxSize;
	}

	/**
	 * Push a new state onto the stack. Clears any redo history.
	 */
	push(state: UndoState): void {
		// Remove any redo entries
		if (this.index < this.stack.length - 1) {
			this.stack = this.stack.slice(0, this.index + 1);
		}

		// Don't push if identical to current state
		const current = this.stack[this.index];
		if (current && current.content === state.content) {
			return;
		}

		this.stack.push(state);
		this.index = this.stack.length - 1;

		// Trim oldest entries if exceeding max size
		if (this.stack.length > this.maxSize) {
			const excess = this.stack.length - this.maxSize;
			this.stack = this.stack.slice(excess);
			this.index -= excess;
		}
	}

	/**
	 * Undo: returns the previous state, or null if at the beginning.
	 */
	undo(): UndoState | null {
		if (this.index <= 0) {
			return null;
		}
		this.index--;
		return this.stack[this.index];
	}

	/**
	 * Redo: returns the next state, or null if at the end.
	 */
	redo(): UndoState | null {
		if (this.index >= this.stack.length - 1) {
			return null;
		}
		this.index++;
		return this.stack[this.index];
	}

	/**
	 * Check if undo is available.
	 */
	canUndo(): boolean {
		return this.index > 0;
	}

	/**
	 * Check if redo is available.
	 */
	canRedo(): boolean {
		return this.index < this.stack.length - 1;
	}

	/**
	 * Clear the stack and reset.
	 */
	clear(): void {
		this.stack = [];
		this.index = -1;
	}

	/**
	 * Initialize with an initial state (e.g., when loading a file).
	 */
	init(state: UndoState): void {
		this.clear();
		this.push(state);
	}

	/**
	 * Get the current stack size (for debugging).
	 */
	get size(): number {
		return this.stack.length;
	}
}
