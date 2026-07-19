import { json } from '@sveltejs/kit';
import * as fs from 'fs';
import * as path from 'path';

export async function GET() {
    try {
        const projectRoot = path.resolve(process.cwd(), '..');
        const benchSuitesDir = path.join(projectRoot, 'tests', 'benchmarks');
        
        let suites = [];
        if (fs.existsSync(benchSuitesDir)) {
            const items = fs.readdirSync(benchSuitesDir, { withFileTypes: true });
            for (const item of items) {
                if (item.isDirectory() && item.name.startsWith('benchmark-')) {
                    // Return the relative path from projectRoot
                    suites.push(path.join('tests', 'benchmarks', item.name));
                }
            }
        }
        
        return json(suites, {
            headers: {
                'Cache-Control': 'no-cache, no-store, must-revalidate',
                'Pragma': 'no-cache',
                'Expires': '0'
            }
        });
    } catch (err) {
        console.error("Benchmark Suites API Error:", err);
        return json({ error: err.message }, { status: 500 });
    }
}
