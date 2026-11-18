import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

const CORS_HEADERS = {
	'Access-Control-Allow-Origin': '*',
	'Access-Control-Allow-Methods': 'POST, OPTIONS',
	'Access-Control-Allow-Headers': 'Content-Type, Authorization'
};

const ALLOWED_EXTENSIONS = new Set(['java', 'rs', 'py', 'c', 'cpp']);

// In-memory rate limiting: 20 requests per minute per IP
const RATE_LIMIT_WINDOW_MS = 60 * 1000;
const MAX_REQUESTS_PER_WINDOW = 20;
const ipRequests = new Map<string, { count: number; resetTime: number }>();

function isRateLimited(ip: string): boolean {
	const now = Date.now();
	const entry = ipRequests.get(ip);

	if (!entry || now > entry.resetTime) {
		ipRequests.set(ip, { count: 1, resetTime: now + RATE_LIMIT_WINDOW_MS });
		return false;
	}

	if (entry.count >= MAX_REQUESTS_PER_WINDOW) {
		return true;
	}

	entry.count++;
	return false;
}

// Periodic cleanup of stale rate-limit entries every 5 minutes
setInterval(() => {
	const now = Date.now();
	for (const [ip, entry] of ipRequests.entries()) {
		if (now > entry.resetTime) {
			ipRequests.delete(ip);
		}
	}
}, 5 * 60 * 1000);

export const OPTIONS: RequestHandler = async () => {
	return new Response(null, {
		status: 204,
		headers: {
			...CORS_HEADERS,
			'Access-Control-Max-Age': '86400'
		}
	});
};

export const POST: RequestHandler = async ({ request, getClientAddress, fetch }) => {
	// 1. Rate Limiting Check
	let clientIp = 'unknown';
	try {
		clientIp =
			request.headers.get('x-forwarded-for')?.split(',')[0].trim() ||
			request.headers.get('x-real-ip') ||
			getClientAddress() ||
			'unknown';
	} catch {
		// Fallback for environments where getClientAddress is not available
	}

	if (clientIp !== 'unknown' && isRateLimited(clientIp)) {
		return json(
			{ error: 'Too many requests. Please wait a minute before analyzing more code.' },
			{
				status: 429,
				headers: {
					...CORS_HEADERS,
					'Retry-After': '60'
				}
			}
		);
	}

	try {
		const payload = await request.json();

		// 2. Input Validation
		if (!payload || !payload.code || !payload.extension) {
			return json(
				{ error: "Missing required parameters: 'code' and 'extension'" },
				{ status: 400, headers: CORS_HEADERS }
			);
		}

		// 3. Supported Languages Validation
		const ext = String(payload.extension).toLowerCase().trim();
		if (!ALLOWED_EXTENSIONS.has(ext)) {
			return json(
				{ error: `Unsupported language extension '${ext}'. Allowed: java, rs, py, c, cpp.` },
				{ status: 400, headers: CORS_HEADERS }
			);
		}

		// 4. Payload Size Limit (Max 64KB of code)
		if (payload.code.length > 64 * 1024) {
			return json(
				{ error: 'Code payload exceeds maximum size limit (64KB).' },
				{ status: 413, headers: CORS_HEADERS }
			);
		}

		const backendUrl = env.OMNIDEPS_BACKEND_URL || 'http://127.0.0.1:3000';

		// 5. Execution Timeout (Max 10 seconds per analysis)
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), 10000);

		try {
			const res = await fetch(`${backendUrl}/api/analyze`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				// Strictly only forward code and extension - NO paths or other fields
				body: JSON.stringify({
					code: payload.code,
					extension: ext
				}),
				signal: controller.signal
			});

			clearTimeout(timeoutId);

			if (!res.ok) {
				const errData = await res.json().catch(() => ({}));
				return json(
					{ error: errData.error || `Omnideps engine responded with HTTP ${res.status}` },
					{ status: res.status, headers: CORS_HEADERS }
				);
			}

			const data = await res.json();
			return json(data, { headers: CORS_HEADERS });
		} catch (fetchErr: any) {
			clearTimeout(timeoutId);
			if (fetchErr.name === 'AbortError') {
				return json(
					{ error: 'Analysis timed out (exceeded 10 seconds limit).' },
					{ status: 504, headers: CORS_HEADERS }
				);
			}
			throw fetchErr;
		}
	} catch (err: any) {
		return json(
			{
				error:
					'Cannot reach Omnideps backend service. Make sure your server container is running: ' +
					(err?.message || err)
			},
			{ status: 502, headers: CORS_HEADERS }
		);
	}
};
