<script>
    import { onMount } from 'svelte';
    import { marked } from 'marked';

    let tree = $state([]);
    let selectedFile = $state(null);
    let fileContent = $state('');
    let isHtml = $state(false);
    let isLoading = $state(true);
    let isContentLoading = $state(false);
    let openDirs = $state({});
    let isSidebarOpen = $state(true);
    let sidebarWidth = $state(300);
    let isResizing = $state(false);

    function toggleSidebar() {
        isSidebarOpen = !isSidebarOpen;
    }

    function startResize(e) {
        isResizing = true;
        if (typeof document !== 'undefined') {
            document.body.style.cursor = 'ew-resize';
            document.body.style.userSelect = 'none';
        }
    }

    function doResize(e) {
        if (!isResizing) return;
        let newWidth = e.clientX;
        if (newWidth < 200) newWidth = 200;
        if (newWidth > window.innerWidth * 0.5) newWidth = window.innerWidth * 0.5;
        sidebarWidth = newWidth;
    }

    function stopResize() {
        if (isResizing) {
            isResizing = false;
            if (typeof document !== 'undefined') {
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
            }
        }
    }

    onMount(async () => {
        try {
            const res = await fetch('/api/docs/tree?t=' + Date.now());
            const data = await res.json();
            tree = data.tree || [];
        } catch (err) {
            console.error(err);
        } finally {
            isLoading = false;
        }
    });

    async function selectFile(node) {
        if (node.type === 'directory') return;
        
        selectedFile = node;
        fileContent = '';
        isContentLoading = true;
        try {
            const res = await fetch('/api/docs/content?path=' + encodeURIComponent(node.path) + '&t=' + Date.now());
            const data = await res.json();
            if (data.error) throw new Error(data.error);
            fileContent = data.content;
            isHtml = data.isHtml;
        } catch (err) {
            console.error(err);
            fileContent = '# Error loading file\n' + err.message;
            isHtml = false;
        } finally {
            isContentLoading = false;
        }
    }

    function toggleDir(path) {
        openDirs[path] = !openDirs[path];
    }

    let renderedContent = $derived.by(() => {
        if (!fileContent) return '';
        if (isHtml) {
            return fileContent;
        } else {
            return marked(fileContent);
        }
    });
</script>

<svelte:window onmousemove={doResize} onmouseup={stopResize} />

<style>
    .docs-container {
        display: flex;
        height: 100vh;
        background-color: #121212;
        color: #ecf0f1;
    }
    
    .sidebar {
        min-width: 200px;
        max-width: 50vw;
        background: rgba(30, 30, 30, 0.5);
        border-right: 1px solid #333;
        overflow-y: auto;
        overflow-x: hidden;
        padding: 10px 5px 80px 5px;
        box-sizing: border-box;
    }
    
    .resizer {
        width: 4px;
        background: transparent;
        cursor: ew-resize;
        transition: background 0.2s;
        z-index: 10;
        flex-shrink: 0;
    }
    .resizer:hover, .resizer.active {
        background: #0969da;
    }
    
    .content-area {
        flex-grow: 1;
        overflow: hidden;
        display: flex;
        flex-direction: column;
        background: #1e1e1e;
    }
    
    .header {
        padding: 15px 30px;
        border-bottom: 1px solid #333;
        background: rgba(20, 20, 20, 0.8);
        font-weight: bold;
        font-size: 18px;
        color: #3498db;
        display: flex;
        align-items: center;
    }
    
    .toggle-btn {
        background: transparent;
        border: none;
        color: #ecf0f1;
        cursor: pointer;
        font-size: 16px;
        margin-right: 15px;
        padding: 5px;
        transition: color 0.2s;
    }
    .toggle-btn:hover {
        color: #3498db;
    }
    
    .viewer {
        flex-grow: 1;
        overflow-y: auto;
        padding: 30px 30px 80px 30px;
        box-sizing: border-box;
    }

    .html-frame {
        width: 100%;
        height: 100%;
        border: none;
        background: transparent;
    }

    /* Markdown Styles matching reports */
    :global(.md-content) {
        max-width: 900px;
        margin: 0 auto;
        line-height: 1.6;
        font-size: 16px;
    }
    :global(.md-content h1, .md-content h2, .md-content h3) {
        color: #3498db;
        margin-top: 25px;
        margin-bottom: 15px;
    }
    :global(.md-content h1) { font-size: 28px; border-bottom: 1px solid #444; padding-bottom: 8px; }
    :global(.md-content h2) { font-size: 22px; border-bottom: 1px solid #444; padding-bottom: 6px; }
    :global(.md-content h3) { font-size: 18px; }
    :global(.md-content pre) {
        background: rgba(0, 0, 0, 0.3);
        padding: 15px;
        border-radius: 8px;
        border: 1px solid #444;
        overflow-x: auto;
    }
    :global(.md-content code) {
        background: rgba(0, 0, 0, 0.3);
        padding: 2px 6px;
        border-radius: 4px;
        font-family: monospace;
        color: #e67e22;
    }
    :global(.md-content table) {
        width: 100%;
        border-collapse: collapse;
        margin-top: 15px;
        margin-bottom: 15px;
    }
    :global(.md-content th), :global(.md-content td) {
        border: 1px solid #444;
        padding: 8px 12px;
        text-align: left;
    }
    :global(.md-content th) { background-color: rgba(30, 30, 30, 0.8); }

    /* Tree Styles */
    ul { list-style: none; padding-left: 0; margin: 0; }
    .tree-list ul { padding-left: 20px; }
    
    .dir-name {
        padding: 4px 6px;
        font-weight: bold;
        color: #bdc3c7;
        margin-top: 2px;
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 13px;
        cursor: pointer;
        border-radius: 4px;
        transition: background 0.1s;
    }
    .dir-name:hover {
        background: rgba(255, 255, 255, 0.05);
    }
    .file-name {
        padding: 4px 6px 4px 20px;
        cursor: pointer;
        color: #ecf0f1;
        transition: background 0.1s, color 0.1s;
        border-radius: 4px;
        margin-bottom: 2px;
        display: flex;
        align-items: center;
        font-size: 13px;
    }
    .file-name:hover { background: rgba(255, 255, 255, 0.05); }
    .file-name.active {
        background: rgba(9, 105, 218, 0.2);
        color: #3498db;
        font-weight: bold;
    }
    .chevron {
        display: inline-block;
        width: 10px;
        text-align: center;
        transition: transform 0.2s;
        font-size: 14px;
        line-height: 1;
    }
    .chevron.open {
        transform: rotate(90deg);
    }
    
    .spinner {
        display: inline-block;
        width: 30px;
        height: 30px;
        border: 3px solid rgba(255,255,255,.3);
        border-radius: 50%;
        border-top-color: #fff;
        animation: spin 1s ease-in-out infinite;
        margin: 20px auto;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
    .loading-center { display: flex; justify-content: center; align-items: center; height: 100%; flex-direction: column; color: #888; }
</style>

{#snippet renderTree(nodes)}
    <ul>
        {#each nodes as node}
            <li>
                {#if node.type === 'directory'}
                    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
                    <div class="dir-name" onclick={() => toggleDir(node.path)} role="button">
                        <span class="chevron" class:open={openDirs[node.path]}>›</span> {node.name}
                    </div>
                    {#if openDirs[node.path]}
                        <div class="tree-list">
                            {@render renderTree(node.children)}
                        </div>
                    {/if}
                {:else}
                    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
                    <div 
                        class="file-name" 
                        class:active={selectedFile?.path === node.path} 
                        onclick={() => selectFile(node)}
                        role="button"
                    >
                        {node.name}
                    </div>
                {/if}
            </li>
        {/each}
    </ul>
{/snippet}

<div class="docs-container" class:resizing={isResizing}>
    {#if isSidebarOpen}
        <div class="sidebar" style="width: {sidebarWidth}px;">
            {#if isLoading}
                <div style="text-align: center; margin-top: 20px; color: #888;">Loading...</div>
            {:else if tree.length === 0}
                <div style="text-align: center; margin-top: 20px; color: #888;">No files found.</div>
            {:else}
                {@render renderTree(tree)}
            {/if}
        </div>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="resizer" class:active={isResizing} onmousedown={startResize}></div>
    {/if}
    
    <div class="content-area">
        {#if selectedFile}
            <div class="header">
                <button class="toggle-btn" onclick={toggleSidebar} title="Toggle Sidebar">
                    {#if isSidebarOpen}
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
                    {:else}
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                    {/if}
                </button>
                {selectedFile.path}
            </div>
            <div class="viewer">
                {#if isContentLoading}
                    <div class="loading-center">
                        <div class="spinner"></div>
                        Loading document...
                    </div>
                {:else if isHtml}
                    <iframe 
                        title="HTML Document Viewer" 
                        class="html-frame" 
                        srcdoc={renderedContent}
                    ></iframe>
                {:else}
                    <div class="md-content">
                        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                        {@html renderedContent}
                    </div>
                {/if}
            </div>
        {:else}
            <div class="header">
                <button class="toggle-btn" onclick={toggleSidebar} title="Toggle Sidebar">
                    {#if isSidebarOpen}
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"></polyline></svg>
                    {:else}
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                    {/if}
                </button>
                Documentation
            </div>
            <div class="loading-center" style="font-size: 1.2em;">
                Select a document from the sidebar to view it.
            </div>
        {/if}
    </div>
</div>
