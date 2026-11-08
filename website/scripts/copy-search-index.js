import fs from 'node:fs';
import path from 'node:path';

const rootDir = path.resolve(import.meta.dirname, '..');
const src = path.join(rootDir, 'static', 'pagefind');
const dest = path.join(rootDir, 'build', 'client', 'pagefind');

if (fs.existsSync(src) && fs.existsSync(path.join(rootDir, 'build', 'client'))) {
	fs.cpSync(src, dest, { recursive: true });
	console.log('[copy-search-index] Successfully synced static/pagefind -> build/client/pagefind');
}
