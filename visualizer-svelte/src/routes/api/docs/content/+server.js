import { json } from '@sveltejs/kit';
import * as fs from 'fs';
import * as path from 'path';

export async function GET({ url }) {
    try {
        const filePath = url.searchParams.get('path');
        if (!filePath) {
            return json({ error: 'Missing path' }, { status: 400 });
        }

        const projectRoot = path.resolve(process.cwd(), '..');
        const docsDir = path.join(projectRoot, 'docs');
        
        // Prevent directory traversal attacks
        const absolutePath = path.resolve(docsDir, filePath);
        if (!absolutePath.startsWith(docsDir)) {
             return json({ error: 'Invalid path' }, { status: 403 });
        }
        
        if (!fs.existsSync(absolutePath)) {
            return json({ error: 'File not found' }, { status: 404 });
        }
        
        const content = fs.readFileSync(absolutePath, 'utf8');
        const isHtml = absolutePath.endsWith('.html');
        
        return json({ content, isHtml }, {
            headers: {
                'Cache-Control': 'no-cache, no-store, must-revalidate',
                'Pragma': 'no-cache',
                'Expires': '0'
            }
        });
    } catch (err) {
        console.error("Docs Content API Error:", err);
        return json({ error: err.message }, { status: 500 });
    }
}
