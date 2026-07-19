import { json } from '@sveltejs/kit';
import * as fs from 'fs';
import * as path from 'path';

function buildTree(dir, basePath = '') {
    const items = fs.readdirSync(dir, { withFileTypes: true });
    let tree = [];
    
    for (const item of items) {
        if (item.name.startsWith('.')) continue; // skip hidden files/dirs
        
        const fullPath = path.join(dir, item.name);
        const relativePath = path.join(basePath, item.name);
        
        if (item.isDirectory()) {
            const children = buildTree(fullPath, relativePath);
            if (children.length > 0) {
                tree.push({ type: 'directory', name: item.name, path: relativePath, children });
            }
        } else if (item.isFile() && (item.name.endsWith('.md') || item.name.endsWith('.html'))) {
            tree.push({ type: 'file', name: item.name, path: relativePath });
        }
    }
    
    // Sort directories first, then files alphabetically
    return tree.sort((a, b) => {
        if (a.type === b.type) return a.name.localeCompare(b.name);
        return a.type === 'directory' ? -1 : 1;
    });
}

export async function GET() {
    try {
        const projectRoot = path.resolve(process.cwd(), '..');
        const docsDir = path.join(projectRoot, 'docs');
        
        if (!fs.existsSync(docsDir)) {
            return json({ tree: [] });
        }
        
        const tree = buildTree(docsDir);
        
        return json({ tree }, {
            headers: {
                'Cache-Control': 'no-cache, no-store, must-revalidate',
                'Pragma': 'no-cache',
                'Expires': '0'
            }
        });
    } catch (err) {
        console.error("Docs API Error:", err);
        return json({ error: err.message }, { status: 500 });
    }
}
