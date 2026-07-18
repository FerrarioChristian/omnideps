import { json } from '@sveltejs/kit';
import { exec } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { promisify } from 'util';

const execAsync = promisify(exec);

export async function POST({ request }) {
    const data = await request.json();
    const projectRoot = path.resolve(process.cwd(), '..');
    
    let targetPath = '';
    let tempDir = null;
    let isTemp = false;

    try {
        if (data.path) {
            // Absolute or relative path provided
            targetPath = path.isAbsolute(data.path) ? data.path : path.join(projectRoot, data.path);
            if (!fs.existsSync(targetPath)) {
                return json({ error: 'Path not found' }, { status: 400 });
            }
        } else if (data.code && data.extension) {
            // Raw code provided
            tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'analyzer-'));
            targetPath = path.join(tempDir, `temp_code.${data.extension}`);
            fs.writeFileSync(targetPath, data.code);
            isTemp = true;
        } else {
            return json({ error: 'Provide either path or code+extension' }, { status: 400 });
        }

        // Output directory is where we'll save the json
        const outDir = tempDir || fs.mkdtempSync(path.join(os.tmpdir(), 'analyzer-out-'));
        const outJson = path.join(outDir, 'output.json');
        
        // Execute the cargo command
        // We use cargo run from the project root
        const cmd = `cargo run --release --bin language-agnostic-analyzer -- "${targetPath}" --output "${outJson}"`;
        
        await execAsync(cmd, { cwd: projectRoot });

        // The CLI also generates cyto_output.json automatically alongside output.json
        const cytoOutJson = path.join(outDir, 'cyto_output.json');
        
        if (!fs.existsSync(cytoOutJson)) {
            throw new Error("Analyzer did not generate cyto_output.json");
        }

        const cytoData = fs.readFileSync(cytoOutJson, 'utf-8');
        const elements = JSON.parse(cytoData);

        // Cleanup
        if (isTemp) {
            fs.rmSync(tempDir, { recursive: true, force: true });
        } else if (outDir !== tempDir) {
            fs.rmSync(outDir, { recursive: true, force: true });
        }

        return json({ elements });
    } catch (err) {
        console.error("Analyzer Error:", err);
        
        // Try cleanup
        if (tempDir && fs.existsSync(tempDir)) {
            fs.rmSync(tempDir, { recursive: true, force: true });
        }
        
        return json({ error: err.message || 'Failed to run analyzer' }, { status: 500 });
    }
}
