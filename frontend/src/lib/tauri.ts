// Detects the Tauri desktop shell and provides native folder picker.
// Uses the backend zenity/kdialog endpoint to avoid Tauri dialog plugin
// crashes on Linux with webkit2gtk.
import { isTauri } from '@tauri-apps/api/core';

export { isTauri };

export async function pickFolder(): Promise<string | null> {
	// Use backend endpoint that calls zenity/kdialog directly,
	// bypassing the broken Tauri dialog plugin on Linux.
	const res = await fetch('/api/v1/pick-folder');
	if (!res.ok) return null;
	const data = await res.json();
	return data.path ?? null;
}
