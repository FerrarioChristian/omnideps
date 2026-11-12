import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/private';

export const POST: RequestHandler = async ({ request, fetch }) => {
	try {
		const payload = await request.json();

		if (!payload || !payload.code || !payload.extension) {
			return json(
				{ error: "Missing required parameters: 'code' and 'extension'" },
				{ status: 400 }
			);
		}

		// Security: limit input code size in server layer (max 64KB)
		if (payload.code.length > 64 * 1024) {
			return json({ error: 'Code payload exceeds maximum size limit (64KB)' }, { status: 413 });
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
				{ status: res.status }
			);
		}

		const data = await res.json();
		return json(data);
	} catch (err: any) {
		return json(
			{
				error:
					'Cannot reach Omnideps backend service. Make sure your server container is running: ' +
					(err?.message || err)
			},
			{ status: 502 }
		);
	}
};
