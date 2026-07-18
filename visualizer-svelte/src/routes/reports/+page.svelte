<script>
    import { marked } from 'marked';

    let selectedBenchmark = $state('tests/benchmark-rust');
    let isRunning = $state(false);
    let error = $state(null);
    let reportMarkdown = $state('');

    let reportHtml = $derived(
        reportMarkdown ? marked(reportMarkdown) : ''
    );

    async function runBenchmark() {
        isRunning = true;
        error = null;
        reportMarkdown = '';
        
        try {
            const res = await fetch('/api/run-benchmark?t=' + Date.now(), {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Cache-Control': 'no-cache, no-store, must-revalidate',
                    'Pragma': 'no-cache',
                    'Expires': '0'
                },
                body: JSON.stringify({ benchmark: selectedBenchmark })
            });

            const data = await res.json();
            
            if (!res.ok) {
                throw new Error(data.error + (data.details ? ': ' + data.details : ''));
            }

            reportMarkdown = data.markdown;
        } catch (err) {
            error = err.message;
            console.error(err);
        } finally {
            isRunning = false;
        }
    }
</script>

<style>
    .reports-container {
        padding: 20px;
        max-width: 95vw;
        margin: 0 auto;
        color: #ecf0f1;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
        height: calc(100vh - 60px); /* Adjust for navbar */
        overflow-y: auto;
        box-sizing: border-box;
    }
    
    h1 {
        font-size: 1.5em;
        border-bottom: 1px solid #444;
        padding-bottom: 5px;
        margin-top: 0;
        margin-bottom: 15px;
        color: #fff;
    }

    .controls {
        background: rgba(30, 30, 30, 0.85);
        backdrop-filter: blur(10px);
        border: 1px solid #444;
        border-radius: 8px;
        padding: 10px 15px;
        margin-bottom: 15px;
        display: flex;
        gap: 15px;
        align-items: flex-end;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.5);
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
        flex-grow: 1;
    }

    .form-group label {
        font-weight: bold;
        color: #bdc3c7;
        font-size: 12px;
    }

    select {
        padding: 6px 10px;
        font-size: 13px;
        border-radius: 6px;
        border: 1px solid #444;
        background: rgba(10, 10, 10, 0.5);
        color: white;
        outline: none;
        appearance: none;
        cursor: pointer;
        transition: border-color 0.2s;
    }

    select:focus {
        border-color: #0969da;
    }

    .run-btn {
        padding: 6px 16px;
        background: #0969da;
        color: white;
        border: none;
        border-radius: 6px;
        cursor: pointer;
        font-weight: bold;
        font-size: 13px;
        transition: all 0.2s;
        min-width: 120px;
        display: flex;
        align-items: center;
        justify-content: center;
        height: 29px; /* match select height closely */
    }

    .run-btn:hover:not(:disabled) {
        opacity: 0.9;
        box-shadow: 0 4px 15px rgba(9, 105, 218, 0.4);
    }

    .run-btn:disabled {
        background: #444;
        cursor: not-allowed;
        color: #888;
    }

    .error-msg {
        background: rgba(231, 76, 60, 0.2);
        border: 1px solid #e74c3c;
        color: #ffb8b8;
        padding: 15px;
        border-radius: 8px;
        margin-bottom: 20px;
    }

    .report-content {
        background: rgba(20, 20, 20, 0.5);
        border: 1px solid #444;
        border-radius: 12px;
        padding: 20px;
        line-height: 1.5;
        overflow-x: auto;
        font-size: 16px;
    }

    /* Markdown Styles */
    :global(.report-content h1, .report-content h2, .report-content h3) {
        color: #3498db;
        font-size: 1.5rem;
        margin-top: 0;
        margin-bottom: 1rem;
    }
    :global(.report-content h2) {
        border-bottom: 1px solid #444;
        padding-bottom: 6px;
        font-size: 1.2rem;
    }
    :global(.report-content h3) {
        font-size: 1.5px;
    }
    :global(.report-content table) {
        width: 100%;
        border-collapse: collapse;
        margin-top: 15px;
        margin-bottom: 15px;
    }
    :global(.report-content th), :global(.report-content td) {
        border: 1px solid #444;
        padding: 0.5rem 0.5rem;
        text-align: left;
        line-height: 1.3;
        font-size: 0.8rem;
        white-space: nowrap;
    }
    :global(.report-content td img) {
        height: 18px;
        width: 18px;
        vertical-align: middle;
    }
    :global(.report-content th) {
        background-color: rgba(30, 30, 30, 0.8);
        color: #3498db;
        font-weight: bold;
        font-size: 0.8rem;
    }
    :global(.report-content tr:nth-child(even)) {
        background-color: rgba(255, 255, 255, 0.02);
    }
    :global(.report-content tr:hover) {
        background-color: rgba(9, 105, 218, 0.1);
    }
    :global(.report-content code) {
        background: rgba(0, 0, 0, 0.3);
        padding: 2px 6px;
        border-radius: 4px;
        font-family: monospace;
        color: #e67e22;
    }
    
    .spinner {
        display: inline-block;
        width: 20px;
        height: 20px;
        border: 3px solid rgba(255,255,255,.3);
        border-radius: 50%;
        border-top-color: #fff;
        animation: spin 1s ease-in-out infinite;
        margin-right: 10px;
    }
    
    @keyframes spin {
        to { transform: rotate(360deg); }
    }
</style>

<div class="reports-container">
    <h1>Benchmark Reports</h1>
    
    <div class="controls">
        <div class="form-group">
            <label for="benchmark-select">Seleziona Suite di Benchmark</label>
            <select id="benchmark-select" bind:value={selectedBenchmark}>
                <option value="tests/benchmark-rust">Benchmark Rust</option>
                <option value="tests/benchmark-java">Benchmark Java</option>
            </select>
        </div>
        
        <button class="run-btn" onclick={runBenchmark} disabled={isRunning}>
            {#if isRunning}
                <div class="spinner"></div> Running...
            {:else}
                ▶ Run Benchmark
            {/if}
        </button>
    </div>

    {#if error}
        <div class="error-msg">
            <strong>Errore:</strong> {error}
        </div>
    {/if}

    {#if reportHtml && !isRunning}
        <div class="report-content">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html reportHtml}
        </div>
    {/if}
</div>
