import { json } from '@sveltejs/kit';
import * as fs from 'fs';
import * as path from 'path';

function walkDir(dir, filesList = []) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const filePath = path.join(dir, file);
        if (fs.statSync(filePath).isDirectory()) {
            walkDir(filePath, filesList);
        } else {
            // Include common benchmark source files
            if (filePath.endsWith('.java') || filePath.endsWith('.rs') || filePath.endsWith('.py') || filePath.endsWith('.c') || filePath.endsWith('.cpp')) {
                filesList.push(filePath);
            }
        }
    }
    return filesList;
}

export async function GET() {
    const projectRoot = path.resolve(process.cwd(), '..');
    const benchGenerics = path.join(projectRoot, 'tests', 'generics');
    const benchSuites = path.join(projectRoot, 'tests', 'benchmarks');
    
    let files = [];
    if (fs.existsSync(benchGenerics)) {
        files = files.concat(walkDir(benchGenerics));
    }
    if (fs.existsSync(benchSuites)) {
        const items = fs.readdirSync(benchSuites, { withFileTypes: true });
        for (const item of items) {
            if (item.isDirectory() && item.name.startsWith('benchmark-')) {
                files.push(path.join(benchSuites, item.name));
            }
        }
    }
    
    // Make paths relative to project root for nicer display
    files = files.map(f => path.relative(projectRoot, f));
    
    return json(files);
}
