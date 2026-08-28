// Detects the Tauri desktop shell and wraps its native folder picker
// (ADR 0019). Kept out of `$api/client.ts` so the API client stays unaware
// of which shell it runs in — only the launcher screen needs to know.
import { isTauri } from '@tauri-apps/api/core';

export { isTauri };

export async function pickFolder(): Promise<string | null> {
	const { open } = await import('@tauri-apps/plugin-dialog');
	const selected = await open({ directory: true, multiple: false });
	return typeof selected === 'string' ? selected : null;
}
