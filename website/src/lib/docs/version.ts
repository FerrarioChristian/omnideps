const CACHE_DURATION = 1000 * 60 * 60; // 1 hour
export const FALLBACK_VERSION = 'v1.0.0';
export const FALLBACK_RELEASE_URL = 'https://github.com/FerrarioChristian/omnideps/releases/latest';

export interface ReleaseInfo {
	tag: string;
	url: string;
}

export async function fetchLatestRelease(): Promise<ReleaseInfo> {
	if (typeof window === 'undefined') {
		return { tag: FALLBACK_VERSION, url: FALLBACK_RELEASE_URL };
	}

	try {
		const stored = sessionStorage.getItem('omnideps_latest_release');
		if (stored) {
			const parsed = JSON.parse(stored);
			if (Date.now() - parsed.timestamp < CACHE_DURATION && parsed.tag) {
				return { tag: parsed.tag, url: parsed.url };
			}
		}
	} catch {
		// SessionStorage parse/access failure ignored
	}

	try {
		const res = await fetch('https://api.github.com/repos/FerrarioChristian/omnideps/releases/latest');
		if (res.ok) {
			const data = await res.json();
			const info: ReleaseInfo = {
				tag: data.tag_name || FALLBACK_VERSION,
				url: data.html_url || FALLBACK_RELEASE_URL
			};
			try {
				sessionStorage.setItem(
					'omnideps_latest_release',
					JSON.stringify({ ...info, timestamp: Date.now() })
				);
			} catch {
				// Quota or private mode
			}
			return info;
		}
	} catch {
		// Network failure, use fallback
	}

	return { tag: FALLBACK_VERSION, url: FALLBACK_RELEASE_URL };
}
