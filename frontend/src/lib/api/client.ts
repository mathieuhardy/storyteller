// Abstract client over the HTTP API (docs/api.md §1). Screens call these
// functions and never build URLs or touch `fetch` directly, so the transport can
// change (a Tauri command layer, M7) without touching the UI. Every call takes
// an optional `fetch` so it works inside SvelteKit `load` (which supplies one)
// and in components (where it defaults to the global).

import type {
	AssetInfo,
	BookResponse,
	Entry,
	EntrySummary,
	Frontmatter,
	Graph,
	OutgoingLink,
	Backlink,
	Page,
	ProjectResponse,
	ProjectsResponse,
	SearchResult,
	Stub,
	TypeResponse,
	VersionResponse
} from './types';

/** Base path of the versioned API; same origin (dev proxy / prod server). */
const BASE = '/api/v1';

type Fetch = typeof fetch;

/** A request that the server refused, carrying its normalized error body. */
export class ApiError extends Error {
	constructor(
		readonly status: number,
		readonly code: string,
		message: string
	) {
		super(message);
		this.name = 'ApiError';
	}
}

async function request<T>(
	path: string,
	init: RequestInit | undefined,
	fetchImpl: Fetch
): Promise<T> {
	let response: Response;
	try {
		response = await fetchImpl(`${BASE}${path}`, init);
	} catch (cause) {
		// A dead server / network drop never reaches an HTTP status.
		throw new ApiError(0, 'network_error', `cannot reach the server: ${String(cause)}`);
	}

	const body = await response.text();
	const parsed = body ? JSON.parse(body) : null;

	if (!response.ok) {
		const error = parsed?.error ?? {};
		throw new ApiError(
			response.status,
			error.code ?? 'error',
			error.message ?? response.statusText
		);
	}
	return parsed as T;
}

function json(payload: unknown): RequestInit {
	return {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(payload)
	};
}

// --- Projects -------------------------------------------------------------

export const listProjects = (fetchImpl: Fetch = fetch): Promise<ProjectsResponse> =>
	request('/projects', undefined, fetchImpl);

export const openProject = (path: string, fetchImpl: Fetch = fetch): Promise<ProjectResponse> =>
	request('/projects/open', json({ path }), fetchImpl);

/** Removes a project or book from the recent list (does not delete files). */
export const removeRecent = (path: string, fetchImpl: Fetch = fetch): Promise<void> =>
	request('/projects/recent', { ...json({ path }), method: 'DELETE' }, fetchImpl);

export const getProject = (fetchImpl: Fetch = fetch): Promise<ProjectResponse> =>
	request('/project', undefined, fetchImpl);

export const getVersion = (fetchImpl: Fetch = fetch): Promise<VersionResponse> =>
	request('/version', undefined, fetchImpl);

// --- Types ----------------------------------------------------------------

export const getTypes = (
	opts: { all?: boolean } = {},
	fetchImpl: Fetch = fetch
): Promise<TypeResponse[]> => {
	const params = new URLSearchParams();
	if (opts.all) params.set('all', 'true');
	const query = params.toString();
	return request(`/types${query ? `?${query}` : ''}`, undefined, fetchImpl);
};

export const getType = (type: string, fetchImpl: Fetch = fetch): Promise<TypeResponse> =>
	request(`/types/${encodeURIComponent(type)}`, undefined, fetchImpl);

export const setTypeEnabled = (
	type: string,
	enabled: boolean,
	fetchImpl: Fetch = fetch
): Promise<TypeResponse> =>
	request(
		`/types/${encodeURIComponent(type)}`,
		{ ...json({ enabled }), method: 'PATCH' },
		fetchImpl
	);

// --- Entities (wired as the screens land) ---------------------------------

export const getEntities = (
	query = '',
	fetchImpl: Fetch = fetch
): Promise<Page<EntrySummary>> => request(`/entities${query}`, undefined, fetchImpl);

export const getEntity = (
	slug: string,
	opts: { backlinks?: boolean; html?: boolean } = {},
	fetchImpl: Fetch = fetch
): Promise<Entry> => {
	const params = new URLSearchParams();
	if (opts.backlinks) params.set('include', 'backlinks');
	if (opts.html) params.set('render', 'html');
	const query = params.toString();
	return request(`/entities/${encodeURIComponent(slug)}${query ? `?${query}` : ''}`, undefined, fetchImpl);
};

export const getLinks = (slug: string, fetchImpl: Fetch = fetch): Promise<OutgoingLink[]> =>
	request(`/entities/${encodeURIComponent(slug)}/links`, undefined, fetchImpl);

export const getBacklinks = (slug: string, fetchImpl: Fetch = fetch): Promise<Backlink[]> =>
	request(`/entities/${encodeURIComponent(slug)}/backlinks`, undefined, fetchImpl);

export const getStubs = (fetchImpl: Fetch = fetch): Promise<Stub[]> =>
	request('/stubs', undefined, fetchImpl);

export const getGraph = (fetchImpl: Fetch = fetch): Promise<Graph> =>
	request('/graph', undefined, fetchImpl);

// --- Search -----------------------------------------------------------------

/** `q` is required; `query` carries the rest of the query string verbatim
 * (`type=`, `tag=`, `<field>=`, `sort=`, `page=`, `per_page=` — docs/api.md §4). */
export const search = (
	query: string,
	fetchImpl: Fetch = fetch
): Promise<Page<SearchResult>> => request(`/search${query}`, undefined, fetchImpl);

// --- Assets -----------------------------------------------------------------

export const getAssets = (fetchImpl: Fetch = fetch): Promise<AssetInfo[]> =>
	request('/assets', undefined, fetchImpl);

/** Uploads a file under `assets/`; the server flattens its name to a bare
 * filename and refuses to overwrite an existing one (409). */
export const uploadAsset = (
	file: File | Blob,
	filename: string,
	fetchImpl: Fetch = fetch
): Promise<{ path: string }> => {
	const form = new FormData();
	form.append('file', file, filename);
	return request('/assets', { method: 'POST', body: form }, fetchImpl);
};

/** URL to fetch/display an asset by its project-relative path. */
export const assetUrl = (path: string): string =>
	`${BASE}/assets/${path.split('/').map(encodeURIComponent).join('/')}`;

export const createEntity = (
	body: { type: string; title: string; frontmatter?: Frontmatter; body?: string },
	fetchImpl: Fetch = fetch
): Promise<Entry> => request('/entities', json(body), fetchImpl);

export const updateEntity = (
	slug: string,
	body: { frontmatter?: Frontmatter; body?: string },
	fetchImpl: Fetch = fetch
): Promise<Entry> =>
	request(`/entities/${encodeURIComponent(slug)}`, { ...json(body), method: 'PATCH' }, fetchImpl);

// --- Files (Book Mode) ------------------------------------------------------

/** A file or directory entry from the files API. */
export interface FileEntry {
	path: string;
	name: string;
	is_dir: boolean;
}

/** Content of a file from the files API. */
export interface FileContent {
	path: string;
	content: string;
}

/** Lists files and directories in a path. Only .md files are returned. */
export const listFiles = (
	path?: string,
	fetchImpl: Fetch = fetch
): Promise<FileEntry[]> => {
	const params = new URLSearchParams();
	if (path) params.set('path', path);
	const query = params.toString();
	return request(`/files${query ? `?${query}` : ''}`, undefined, fetchImpl);
};

/** Reads the raw content of a file. */
export const readFile = (path: string, fetchImpl: Fetch = fetch): Promise<FileContent> =>
	request(`/files/${path.split('/').map(encodeURIComponent).join('/')}`, undefined, fetchImpl);

/** Writes raw content to a file. Only .md files are allowed. */
export const writeFile = (
	path: string,
	content: string,
	fetchImpl: Fetch = fetch
): Promise<void> =>
	request(
		`/files/${path.split('/').map(encodeURIComponent).join('/')}`,
		{ ...json({ content }), method: 'PUT' },
		fetchImpl
	);

// --- Books (standalone markdown folders) -------------------------------------

/** Opens a folder as a standalone book (no .storyteller directory required). */
export const openBook = (path: string, fetchImpl: Fetch = fetch): Promise<BookResponse> =>
	request('/books/open', json({ path }), fetchImpl);

/** Lists files and directories in the active book. Only .md files are returned. */
export const listBookFiles = (path?: string, fetchImpl: Fetch = fetch): Promise<FileEntry[]> => {
	const params = new URLSearchParams();
	if (path) params.set('path', path);
	const query = params.toString();
	return request(`/books/files${query ? `?${query}` : ''}`, undefined, fetchImpl);
};

/** Reads the raw content of a file from the active book. */
export const readBookFile = (path: string, fetchImpl: Fetch = fetch): Promise<FileContent> =>
	request(`/books/files/${path.split('/').map(encodeURIComponent).join('/')}`, undefined, fetchImpl);

/** Writes raw content to a file in the active book. Only .md files are allowed. */
export const writeBookFile = (
	path: string,
	content: string,
	fetchImpl: Fetch = fetch
): Promise<void> =>
	request(
		`/books/files/${path.split('/').map(encodeURIComponent).join('/')}`,
		{ ...json({ content }), method: 'PUT' },
		fetchImpl
	);
