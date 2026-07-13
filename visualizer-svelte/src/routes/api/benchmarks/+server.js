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
    const benchMain = path.join(projectRoot, 'benchmarks');
    const benchRust = path.join(projectRoot, 'tests', 'benchmark-rust', 'src');
    const benchJava = path.join(projectRoot, 'tests', 'benchmark-java', 'src');
    
    let files = [];
    if (fs.existsSync(benchMain)) {
        files = files.concat(walkDir(benchMain));
    }
    if (fs.existsSync(benchRust)) {
        files = files.concat(walkDir(benchRust));
    }
    if (fs.existsSync(benchJava)) {
        files = files.concat(walkDir(benchJava));
    }
    
    // Make paths relative to project root for nicer display
    files = files.map(f => path.relative(projectRoot, f));
    
    return json(files);
}
