<script>
    import { onMount } from 'svelte';
    import GraphViewer from '$lib/GraphViewer.svelte';

    let files = $state([]);
    let selectedFile = $state('');
    let searchQuery = $state('');
    let elements = $state([]);
    let rawOutput = $state(null);
    let statusMessage = $state('Loading benchmarks...');
    let isAnalyzing = $state(false);
    
    let isModalOpen = $state(false);

    let filteredFiles = $derived(
        searchQuery ? files.filter(f => f.toLowerCase().includes(searchQuery.toLowerCase())) : files
    );

    onMount(async () => {
        try {
            const res = await fetch('/api/benchmarks?t=' + Date.now(), {
                headers: {
                    'Cache-Control': 'no-cache, no-store, must-revalidate',
                    'Pragma': 'no-cache',
                    'Expires': '0'
                }
            });
            files = await res.json();
            
            if (files.length === 0) {
                statusMessage = "No benchmark files found.";
            } else {
                statusMessage = "Select a benchmark to analyze.";
            }
        } catch (err) {
            statusMessage = "Error loading benchmark list.";
            console.error(err);
        }
        
        // Listen for Esc key to close modal
        window.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') isModalOpen = false;
        });
    });

    async function analyzeGraph(path) {
        if (!path) return;
        isAnalyzing = true;
        statusMessage = "Analyzing: " + path + " ...";
        try {
            const res = await fetch('/api/analyze?t=' + Date.now(), {
                method: 'POST',
                headers: { 
                    'Content-Type': 'application/json',
                    'Cache-Control': 'no-cache, no-store, must-revalidate',
                    'Pragma': 'no-cache',
                    'Expires': '0'
                },
                body: JSON.stringify({ path })
            });
            
            if (!res.ok) {
                const errData = await res.json();
                throw new Error(errData.error || "Failed to analyze");
            }
            
            const data = await res.json();
            elements = data.elements;
            rawOutput = data.rawOutput || null;
            statusMessage = path;
        } catch (err) {
            statusMessage = "Error analyzing: " + err.message;
            console.error(err);
        } finally {
            isAnalyzing = false;
        }
    }

    function selectFile(f) {
        selectedFile = f;
        isModalOpen = false;
        analyzeGraph(f);
    }
</script>

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(4px);
        display: flex;
        justify-content: center;
        align-items: flex-start;
        padding-top: 15vh;
        z-index: 1000;
    }
    .modal-content {
        background: rgba(30, 30, 30, 0.85);
        backdrop-filter: blur(10px);
        border: 1px solid #444;
        border-radius: 12px;
        width: 600px;
        max-width: 90vw;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.8);
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }
    .modal-search {
        padding: 15px;
        background: rgba(0, 0, 0, 0.2);
        border-bottom: 1px solid #444;
    }
    .modal-search input {
        width: 100%;
        padding: 12px 16px;
        font-size: 16px;
        border-radius: 8px;
        border: 1px solid #444;
        background: rgba(10, 10, 10, 0.5);
        color: white;
        outline: none;
        box-sizing: border-box;
        transition: border-color 0.2s;
    }
    .modal-search input:focus {
        border-color: #0969da;
    }
    .modal-file-list {
        max-height: 50vh;
        overflow-y: auto;
    }
    .modal-file-item {
        padding: 12px 20px;
        cursor: pointer;
        border-bottom: 1px solid #333;
        font-size: 14px;
        color: #ecf0f1;
        word-break: break-all;
        transition: background 0.1s;
    }
    .modal-file-item:hover {
        background: rgba(9, 105, 218, 0.2);
    }
    .modal-file-item:last-child {
        border-bottom: none;
    }
    
    .open-modal-btn {
        width: 100%;
        padding: 6px 12px;
        background: rgba(30, 30, 30, 0.85);
        color: #bdc3c7;
        border: 1px solid #444;
        border-radius: 6px;
        cursor: pointer;
        font-weight: bold;
        font-size: 12px;
        transition: all 0.2s;
        margin-bottom: 10px;
    }
    .open-modal-btn:hover {
        color: #fff;
        border-color: #0969da;
    }
</style>

{#snippet controls()}
    <button class="open-modal-btn" onclick={() => isModalOpen = true}>
        🔍 Cerca & Apri Benchmark...
    </button>
{/snippet}

<GraphViewer {elements} {rawOutput} {statusMessage} {controls} />

{#if isModalOpen}
    <div class="modal-backdrop" onclick={() => isModalOpen = false} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && (isModalOpen = false)}>
        <div class="modal-content" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={() => {}}>
            <div class="modal-search">
                <!-- svelte-ignore a11y_autofocus -->
                <input 
                    type="text" 
                    bind:value={searchQuery} 
                    placeholder="Cerca file (es. DECL-1, structs.rs)..." 
                    autofocus
                />
            </div>
            <div class="modal-file-list">
                {#each filteredFiles as f}
                    <div 
                        class="modal-file-item" 
                        onclick={() => selectFile(f)}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && selectFile(f)}
                    >
                        {f}
                    </div>
                {/each}
                {#if filteredFiles.length === 0}
                    <div style="padding: 15px 20px; color: #888; font-size: 14px; text-align: center;">Nessun risultato trovato</div>
                {/if}
            </div>
        </div>
    </div>
{/if}
