import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

const CORS_HEADERS = {
	'Access-Control-Allow-Origin': '*',
	'Access-Control-Allow-Methods': 'POST, OPTIONS',
	'Access-Control-Allow-Headers': 'Content-Type, Authorization'
};

export const OPTIONS: RequestHandler = async () => {
	return new Response(null, {
		status: 204,
		headers: {
			...CORS_HEADERS,
			'Access-Control-Max-Age': '86400'
		}
	});
};

export const POST: RequestHandler = async ({ request, fetch }) => {
	try {
		const payload = await request.json();

		if (!payload || !payload.code || !payload.extension) {
			return json(
				{ error: "Missing required parameters: 'code' and 'extension'" },
				{ status: 400, headers: CORS_HEADERS }
			);
		}

		// Security: limit input code size in server layer (max 64KB)
		if (payload.code.length > 64 * 1024) {
			return json(
				{ error: 'Code payload exceeds maximum size limit (64KB)' },
				{ status: 413, headers: CORS_HEADERS }
			);
		}

		const backendUrl = env.OMNIDEPS_BACKEND_URL || 'http://127.0.0.1:3000';

		const res = await fetch(`${backendUrl}/api/analyze`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				code: payload.code,
				extension: payload.extension
			})
		});

		if (!res.ok) {
			const errData = await res.json().catch(() => ({}));
			return json(
				{ error: errData.error || `Omnideps engine responded with HTTP ${res.status}` },
				{ status: res.status, headers: CORS_HEADERS }
			);
		}

		const data = await res.json();
		return json(data, { headers: CORS_HEADERS });
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
