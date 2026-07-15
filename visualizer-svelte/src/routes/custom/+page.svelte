<script>
    import GraphViewer from '$lib/GraphViewer.svelte';

    let mode = $state('path'); // 'path', 'text', or 'json'
    let customPath = $state('');
    
    let rawCode = $state('');
    let selectedLang = $state('rs');
    
    let selectedJsonFile = $state(null);

    let elements = $state([]);
    let rawOutput = $state(null);
    let statusMessage = $state('Provide a path or raw code to analyze.');
    let isAnalyzing = $state(false);

    async function analyzeCode() {
        if (mode === 'json') {
            if (!selectedJsonFile) {
                statusMessage = "Error: No JSON file selected.";
                return;
            }
            try {
                const text = await selectedJsonFile.text();
                elements = JSON.parse(text);
                rawOutput = null; // No raw output when importing cytoscape json directly
                statusMessage = "Loaded " + selectedJsonFile.name + " successfully";
            } catch (err) {
                statusMessage = "Error parsing JSON: " + err.message;
            }
            return;
        }

        isAnalyzing = true;
        statusMessage = "Analyzing...";
        elements = []; // Clear current graph
        rawOutput = null;

        let payload = {};
        if (mode === 'path') {
            if (!customPath.trim()) {
                statusMessage = "Error: Path is empty.";
                isAnalyzing = false;
                return;
            }
            payload = { path: customPath.trim() };
        } else {
            if (!rawCode.trim()) {
                statusMessage = "Error: Code is empty.";
                isAnalyzing = false;
                return;
            }
            payload = { code: rawCode, extension: selectedLang };
        }

        try {
            const res = await fetch('/api/analyze?t=' + Date.now(), {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Cache-Control': 'no-cache, no-store, must-revalidate',
                    'Pragma': 'no-cache',
                    'Expires': '0'
                },
                body: JSON.stringify(payload)
            });
            
            if (!res.ok) {
                const errData = await res.json();
                throw new Error(errData.error || "Failed to analyze");
            }
            
            const data = await res.json();
            elements = data.elements;
            rawOutput = data.rawOutput || null;
            statusMessage = mode === 'path' ? customPath : "Raw Code Analyzed Successfully";
        } catch (err) {
            statusMessage = "Error analyzing: " + err.message;
            console.error(err);
        } finally {
            isAnalyzing = false;
        }
    }
</script>

<style>
    .input-group {
        display: flex;
        flex-direction: column;
        gap: 10px;
        margin-bottom: 15px;
    }
    textarea {
        width: 100%;
        height: 150px;
        background: #1e1e1e;
        color: #fff;
        border: 1px solid #444;
        border-radius: 6px;
        padding: 8px;
        font-family: monospace;
        resize: vertical;
        box-sizing: border-box;
    }
    textarea:focus {
        border-color: #0969da;
        outline: none;
    }
    .mode-switch {
        display: flex;
        gap: 10px;
        margin-bottom: 15px;
    }
    .mode-switch label {
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 5px;
    }
    .analyze-btn {
        padding: 6px 12px;
        background: rgba(30, 30, 30, 0.85);
        color: #bdc3c7;
        border: 1px solid #444;
        border-radius: 6px;
        cursor: pointer;
        font-weight: bold;
        font-size: 12px;
        width: 100%;
        transition: all 0.2s;
    }
    .analyze-btn:hover:not(:disabled) {
        color: #fff;
        border-color: #0969da;
    }
    .analyze-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>

{#snippet controls()}
    <div class="mode-switch">
        <label>
            <input type="radio" bind:group={mode} value="path">
            Path
        </label>
        <label>
            <input type="radio" bind:group={mode} value="text">
            Raw Code
        </label>
        <label>
            <input type="radio" bind:group={mode} value="json">
            Raw JSON
        </label>
    </div>

    {#if mode === 'path'}
        <div class="input-group">
            <input 
                type="text" 
                bind:value={customPath} 
                placeholder="/path/to/project/or/file.rs" 
            />
        </div>
    {:else if mode === 'text'}
        <div class="input-group">
            <select bind:value={selectedLang}>
                <option value="rs">Rust (.rs)</option>
                <option value="java">Java (.java)</option>
                <option value="c">C (.c)</option>
                <option value="cpp">C++ (.cpp)</option>
                <option value="py">Python (.py)</option>
            </select>
            <textarea 
                bind:value={rawCode} 
                placeholder="Paste code here..."
            ></textarea>
        </div>
    {:else}
        <div class="input-group">
            <input 
                type="file" 
                accept=".json"
                onchange={(e) => selectedJsonFile = e.target.files[0]}
                style="padding: 10px; background: rgba(10, 10, 10, 0.5); border: 1px solid #444; border-radius: 6px; color: white;"
            />
        </div>
    {/if}

    <button 
        onclick={analyzeCode} 
        disabled={isAnalyzing}
        class="analyze-btn"
    >
        {isAnalyzing ? 'Analyzing...' : 'Analyze'}
    </button>
{/snippet}

<GraphViewer {elements} {rawOutput} {statusMessage} {controls} />
