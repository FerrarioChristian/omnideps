import { json } from '@sveltejs/kit';
import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs/promises';

export async function POST({ request }) {
    try {
        const { benchmark } = await request.json();
        
        if (!benchmark) {
            return json({ error: 'Missing benchmark path' }, { status: 400 });
        }

        // Validate benchmark name to avoid directory traversal
        if (benchmark !== 'tests/benchmark-rust' && benchmark !== 'tests/benchmark-java') {
            return json({ error: 'Invalid benchmark path' }, { status: 400 });
        }

        const projectRoot = path.resolve(process.cwd(), '..');
        const targetPath = path.resolve(projectRoot, benchmark);
        const reportPath = path.join(targetPath, 'report.md');

        return new Promise((resolve) => {
            const cargo = spawn('cargo', ['run', '--release', '--bin', 'benchmark_runner', '--', targetPath], {
                cwd: projectRoot,
            });

            let stderrData = '';
            let stdoutData = '';

            cargo.stdout.on('data', (data) => {
                stdoutData += data.toString();
            });

            cargo.stderr.on('data', (data) => {
                stderrData += data.toString();
            });

            cargo.on('close', async (code) => {
                if (code !== 0) {
                    console.error("Cargo run failed:", stderrData);
                    resolve(json({ error: 'Failed to run benchmark', details: stderrData }, { status: 500 }));
                    return;
                }

                try {
                    // Wait a bit to ensure file is written completely just in case
                    const markdownContent = await fs.readFile(reportPath, 'utf8');
                    resolve(json({ markdown: markdownContent }));
                } catch (err) {
                    console.error("Failed to read report.md:", err);
                    resolve(json({ error: 'Failed to read report.md', details: err.message }, { status: 500 }));
                }
            });
        });

    } catch (err) {
        console.error(err);
        return json({ error: err.message }, { status: 500 });
    }
}
