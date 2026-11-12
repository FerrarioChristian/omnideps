<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import cytoscape from 'cytoscape';
    import fcose from 'cytoscape-fcose';
    import style from '$lib/components/demo/cytoscape_style.js';
    import LegendModal from '$lib/components/demo/LegendModal.svelte';

    let { elements = [], rawOutput = null, statusMessage = '', controls }: {
        elements?: any[];
        rawOutput?: any;
        statusMessage?: string;
        controls?: any;
    } = $props();

    let cy: any = null;
    let cyContainer = $state<HTMLDivElement | null>(null);

    let isPanelCollapsed = $state(false);
    let isLegendOpen = $state(false);

    let selectedNode = $state<any>(null);
    let sidebarActive = $state(false);
    
    let inDegreeExpanded = $state(false);
    let outDegreeExpanded = $state(false);

    // Filters
    let structChecked = $state(true);
    let behavChecked = $state(true);
    
    let structFilters = $state<Record<string, boolean>>({
        'IsA': true,
        'Implements': true,
        'Imports': true,
        'UsesFieldType': true,
        'UsesParamType': true,
        'UsesReturnType': true,
        'UsesLocalType': true,
        'NestedIn': true,
        'ModuleContainment': true,
        'AnnotatedWith': true
    });
    
    let behavFilters = $state<Record<string, boolean>>({
        'Calls': true,
        'Instantiates': true,
        'AccessesField': true
    });

    let allNodesChecked = $state(false);
    let nodeFilters = $state<Record<string, boolean>>({
        'Module': true,
        'Class': true,
        'Struct': true,
        'Interface': true,
        'Trait': true,
        'Enum': true,
        'EnumVariant': true,
        'Function': true,
        'StaticVariable': true,
        'StructField': true,
        'ClassField': true,
        'Field': true,
        'Primitive': false,
        'External': false
    });

    let searchQuery = $state('');

    onMount(() => {
        if (typeof window !== 'undefined' && !(window as any).__cy_fcose_registered) {
            cytoscape.use(fcose);
            (window as any).__cy_fcose_registered = true;
        }
    });

    $effect(() => {
        if (cyContainer && elements && elements.length > 0) {
            initCytoscape();
        }
    });

    function initCytoscape() {
        if (cy) {
            cy.destroy();
        }

        cy = cytoscape({
            container: cyContainer,
            elements: elements,
            style: style as any
            // layout will be run after filters
        });

        cy.on('tap', 'node', function(e: any) {
            let node = e.target;
            let neighborhood = node.closedNeighborhood();
            let context = neighborhood.union(node.ancestors()).union(node.descendants());
            
            cy.elements().addClass('dimmed');
            context.removeClass('dimmed');

            let inEdges: any[] = [];
            node.incomers('edge').forEach((e: any) => {
                inEdges.push({
                    label: e.data('label'),
                    sourceId: e.source().data('id'),
                    sourceLabel: e.source().data('label') || e.source().data('id').split('::').pop()
                });
            });
            
            let outEdges: any[] = [];
            node.outgoers('edge').forEach((e: any) => {
                outEdges.push({
                    label: e.data('label'),
                    targetId: e.target().data('id'),
                    targetLabel: e.target().data('label') || e.target().data('id').split('::').pop()
                });
            });

            inDegreeExpanded = false;
            outDegreeExpanded = false;

            selectedNode = {
                name: node.data('label') || node.data('id').split('::').pop(),
                type: node.data('type') || 'Module',
                indegree: node.indegree(false),
                outdegree: node.outdegree(false),
                id: node.data('id'),
                parent: node.data('parent'),
                inEdges,
                outEdges
            };
            sidebarActive = true;
        });

        cy.on('tap', function(e: any) {
            if (e.target === cy) {
                cy.elements().removeClass('dimmed');
                sidebarActive = false;
                selectedNode = null;
            }
        });

        applyFilters();

        cy.layout({
            name: 'fcose',
            quality: "proof",
            randomize: true,
            animate: true,
            animationDuration: 1000,
            fit: true,
            padding: 30,
            nodeDimensionsIncludeLabels: true,
            uniformNodeDimensions: false,
            packComponents: true,
            step: "all",
            idealEdgeLength: () => 80,
            edgeElasticity: () => 0.45,
            nodeRepulsion: () => 3000,
            gravity: 0.25,
            gravityRange: 3.8,
            gravityCompound: 25,
            gravityRangeCompound: 1.5,
            numIter:4000,
            nestingFactor: 0.05,
        }).run();
    }

    function applyFilters() {
        if (!cy) return;
        
        cy.batch(() => {
            cy.edges().addClass('filtered-out');
            cy.nodes().addClass('filtered-out');
            
            let activeNodeFilters = [];
            for (const [key, val] of Object.entries(nodeFilters)) {
                if (val) activeNodeFilters.push(key);
            }
            if (activeNodeFilters.length > 0) {
                const nodeSelector = activeNodeFilters.map(val => `[type = "${val}"]`).join(', ');
                cy.nodes(nodeSelector).removeClass('filtered-out');
            }

            let activeEdgeFilters = [];
            for (const [key, val] of Object.entries(structFilters)) {
                if (val) activeEdgeFilters.push(key);
            }
            for (const [key, val] of Object.entries(behavFilters)) {
                if (val) activeEdgeFilters.push(key);
            }
            
            if (activeEdgeFilters.length > 0) {
                const edgeSelector = activeEdgeFilters.map(val => `[label = "${val}"]`).join(', ');
                cy.edges(edgeSelector).filter((ele: any) => {
                    return !ele.source().hasClass('filtered-out') && !ele.target().hasClass('filtered-out');
                }).removeClass('filtered-out');
            }
        });
    }

    function toggleStructural() {
        for (let k in structFilters) {
            structFilters[k] = structChecked;
        }
        applyFilters();
    }

    function toggleBehavioral() {
        for (let k in behavFilters) {
            behavFilters[k] = behavChecked;
        }
        applyFilters();
    }

    function toggleAllNodes() {
        for (let k in nodeFilters) {
            nodeFilters[k] = allNodesChecked;
        }
        applyFilters();
    }

    function onFilterChange() {
        structChecked = Object.values(structFilters).some(v => v);
        behavChecked = Object.values(behavFilters).some(v => v);
        allNodesChecked = Object.values(nodeFilters).some(v => v);
        applyFilters();
    }

    function handleSearch() {
        if (!cy) return;
        
        const query = searchQuery.toLowerCase().trim();
        if (query === "") {
            cy.elements().removeClass('dimmed');
            return;
        }

        const matched = cy.nodes().filter(function(ele: any) {
            return ele.data('label') && ele.data('label').toLowerCase().includes(query);
        });

        if (matched.length > 0) {
            const context = matched.union(matched.ancestors());
            cy.elements().addClass('dimmed');
            context.removeClass('dimmed');
        } else {
            cy.elements().addClass('dimmed');
        }
    }

    function exportJson() {
        if (!elements || elements.length === 0) return;
        const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(elements, null, 2));
        const downloadAnchorNode = document.createElement('a');
        downloadAnchorNode.setAttribute("href", dataStr);
        downloadAnchorNode.setAttribute("download", "cytoscape_graph.json");
        document.body.appendChild(downloadAnchorNode);
        downloadAnchorNode.click();
        downloadAnchorNode.remove();
    }

    function exportRawJson() {
        if (!rawOutput) return;
        const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(rawOutput, null, 2));
        const downloadAnchorNode = document.createElement('a');
        downloadAnchorNode.setAttribute("href", dataStr);
        downloadAnchorNode.setAttribute("download", "analyzer_raw_output.json");
        document.body.appendChild(downloadAnchorNode);
        downloadAnchorNode.click();
        downloadAnchorNode.remove();
    }

    onDestroy(() => {
        if (cy) {
            cy.destroy();
        }
    });
</script>

<style>
    #cy {
        width: 100%;
        height: 100%;
        position: absolute;
        top: 0;
        left: 0;
        display: block;
        background-color: #1a1a1a;
    }
    #info {
        position: absolute;
        top: 20px;
        left: 20px;
        background: rgba(30, 30, 30, 0.85);
        color: #ecf0f1;
        padding: 12px 15px;
        border-radius: 12px;
        border: 1px solid #444;
        z-index: 10;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(5px);
        min-width: 260px;
        max-width: 340px;
        max-height: calc(100% - 40px);
        overflow-y: auto;
    }
    #info h3 {
        margin: 0;
        font-size: 15px;
        color: #bdc3c7;
        text-transform: uppercase;
        letter-spacing: 1px;
    }
    #status {
        font-size: 12px;
        color: #2ecc71;
        margin-bottom: 10px;
        word-break: break-all;
    }
    select,
    input[type="text"] {
        padding: 8px 12px;
        width: 100%;
        background: #2c3e50;
        color: #fff;
        border: 1px solid #34495e;
        border-radius: 6px;
        font-size: 14px;
        outline: none;
        cursor: pointer;
        transition: border-color 0.2s;
        box-sizing: border-box;
    }
    select:hover,
    input[type="text"]:focus {
        border-color: #0969da;
    }
    .toggles-container {
        margin-top: 12px;
        padding-top: 10px;
        border-top: 1px solid #444;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .toggle-row {
        display: flex;
        align-items: center;
        gap: 10px;
        font-size: 14px;
        cursor: pointer;
    }
    .toggle-row input[type="checkbox"] {
        cursor: pointer;
        width: 16px;
        height: 16px;
        accent-color: #0969da;
    }
    details.advanced-filters {
        margin-top: 10px;
        border-top: 1px solid #444;
        padding-top: 8px;
    }
    details.advanced-filters summary {
        cursor: pointer;
        font-size: 13px;
        color: #bdc3c7;
        font-weight: bold;
        outline: none;
        list-style: none;
        display: flex;
        align-items: center;
    }
    details.advanced-filters summary::-webkit-details-marker {
        display: none;
    }
    details.advanced-filters summary::before {
        content: ">";
        font-size: 12px;
        margin-right: 5px;
        transition: transform 0.2s;
    }
    details.advanced-filters[open] summary::before {
        transform: rotate(90deg);
        display: inline-block;
    }
    .filters-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 8px;
        margin-top: 10px;
        padding-left: 10px;
    }
    .filters-grid .toggle-row {
        font-size: 12px;
        color: #aab2bd;
    }
    #sidebar {
        position: absolute;
        top: 20px;
        right: -350px;
        width: 280px;
        background: rgba(30, 30, 30, 0.85);
        color: #ecf0f1;
        padding: 15px 20px;
        border-radius: 12px;
        border: 1px solid #444;
        z-index: 10;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(5px);
        transition: right 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        max-height: calc(100% - 40px);
        overflow-y: auto;
    }
    #sidebar.active {
        right: 20px;
    }
    #sidebar h2 {
        margin: 0;
        font-size: 18px;
        color: #fff;
        word-break: break-word;
    }
    #sidebar h3 {
        font-size: 12px;
        color: #bdc3c7;
        text-transform: uppercase;
        margin-bottom: 5px;
        margin-top: 0;
    }
    .sidebar-stat {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid #444;
        padding: 8px 0;
        font-size: 13px;
    }
    .sidebar-stat strong {
        color: #0969da;
        font-size: 18px;
    }
    #sb-extra {
        margin-top: 15px;
        font-size: 13px;
        color: #95a5a6;
        background: rgba(0, 0, 0, 0.3);
        padding: 10px;
        border-radius: 6px;
        word-break: break-all;
    }
    .legendBtn {
        position: absolute;
        bottom: 20px;
        right: 20px;
        background: rgba(30, 30, 30, 0.85);
        color: #bdc3c7;
        width: 3rem;
        height: 3rem;
        border: 1px solid #444;
        text-decoration: none;
        border-radius: 999px;
        font-weight: bold;
        font-family: sans-serif;
        font-size: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 10;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(5px);
        transition: all 0.2s;
    }
    .legendBtn:hover {
        background: #0969da;
        color: white;
        border-color: #0969da;
        cursor: pointer;
    }
    #node-search-box {
        padding: 8px 12px;
        font-size: 13px;
        width: 100%;
        border-radius: 6px;
        border: 1px solid #444;
        background: rgba(20, 20, 20, 0.9);
        color: white;
        outline: none;
        transition: all 0.2s ease;
        box-sizing: border-box;
    }
    #node-search-box:focus {
        border-color: #0969da;
    }
    #node-search-box::placeholder {
        color: #888;
    }
    #info.collapsed {
        display: none;
    }
    .panel-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 15px;
    }
    .panel-header h3 {
        margin: 0;
    }
    .toggle-panel-btn {
        background: none;
        border: none;
        color: #bdc3c7;
        cursor: pointer;
        font-size: 20px;
        line-height: 1;
        padding: 0;
        transition: color 0.2s;
    }
    .toggle-panel-btn:hover {
        color: #fff;
    }
    .open-btn {
        position: absolute;
        top: 20px;
        left: 20px;
        background: rgba(30, 30, 30, 0.85);
        color: #bdc3c7;
        padding: 6px 12px;
        border-radius: 6px;
        border: 1px solid #444;
        z-index: 10;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(5px);
        font-weight: bold;
        font-size: 12px;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .open-btn:hover {
        color: #fff;
        border-color: #0969da;
    }
    .export-btn {
        flex: 1;
        padding: 6px 12px;
        background: rgba(30, 30, 30, 0.85);
        color: #bdc3c7;
        border: 1px solid #444;
        border-radius: 6px;
        cursor: pointer;
        font-weight: bold;
        transition: all 0.2s;
        font-size: 12px;
    }
    .export-btn:hover {
        color: #fff;
        border-color: #0969da;
    }
    
    .filter-section {
        margin-bottom: 10px;
        background: rgba(40, 40, 40, 0.5);
        border: 1px solid #444;
        border-radius: 6px;
        overflow: hidden;
    }
    .filter-section summary {
        padding: 10px;
        cursor: pointer;
        font-weight: bold;
        background: rgba(50, 50, 50, 0.5);
        user-select: none;
    }
    .filter-section summary:hover {
        background: rgba(60, 60, 60, 0.8);
    }
    .filter-section .filter-content {
        padding: 10px;
    }

    .edge-list {
        background: rgba(0, 0, 0, 0.2);
        padding: 10px;
        border-radius: 6px;
        margin-bottom: 15px;
        margin-top: 5px;
        font-size: 12px;
        max-height: 200px;
        overflow-y: auto;
    }
    .edge-item {
        margin-bottom: 8px;
        padding-bottom: 8px;
        border-bottom: 1px solid rgba(255,255,255,0.1);
        word-break: break-all;
    }
    .edge-item:last-child {
        margin-bottom: 0;
        padding-bottom: 0;
        border-bottom: none;
    }
    .edge-label {
        color: #e74c3c;
        font-weight: bold;
    }
    .edge-node {
        color: #3498db;
    }
</style>

<!-- Content -->
{#if isPanelCollapsed}
    <button class="open-btn" onclick={() => isPanelCollapsed = false}>
        ☰ Menu
    </button>
{/if}

<div id="info" class:collapsed={isPanelCollapsed}>
    <div class="panel-header">
        <h3>Control Panel</h3>
        <button class="toggle-panel-btn" onclick={() => isPanelCollapsed = true} title="Hide Menu">
            ✕
        </button>
    </div>
    <div style="margin-bottom: 15px;">
        <input type="text" id="node-search-box" bind:value={searchQuery} oninput={handleSearch} placeholder="Search node..." autocomplete="off" spellcheck="false" />
    </div>
    {#if statusMessage}
        <div id="status">{statusMessage}</div>
    {/if}
    
    {#if controls}
        {@render controls()}
    {/if}

    {#if elements && elements.length > 0}
        <div style="display: flex; gap: 10px; margin-bottom: 15px; margin-top: 10px;">
            <button class="export-btn" onclick={exportJson}>
                Export Cytoscape
            </button>
            {#if rawOutput}
                <button class="export-btn" onclick={exportRawJson}>
                    Export Raw JSON
                </button>
            {/if}
        </div>

        <div class="toggles-container" id="toggles-panel">
            <details class="filter-section">
                <summary>Edge Filters (Relationships)</summary>
                <div class="filter-content">
                    <label class="toggle-row" style="font-weight:bold; border-bottom: 1px solid #555; padding-bottom: 5px; margin-bottom: 5px;">
                        <input type="checkbox" bind:checked={structChecked} onchange={toggleStructural}>
                        <span>Structural (IsA, Uses, Imports...)</span>
                    </label>
                    <div class="filters-grid" style="margin-bottom: 15px;">
                        {#each Object.keys(structFilters) as key}
                            <label class="toggle-row">
                                <input type="checkbox" bind:checked={structFilters[key]} onchange={onFilterChange}> 
                                <span>{key}</span>
                            </label>
                        {/each}
                    </div>

                    <label class="toggle-row" style="font-weight:bold; border-bottom: 1px solid #555; padding-bottom: 5px; margin-bottom: 5px;">
                        <input type="checkbox" bind:checked={behavChecked} onchange={toggleBehavioral}>
                        <span>Behavioral (Calls, Instantiates...)</span>
                    </label>
                    <div class="filters-grid">
                        {#each Object.keys(behavFilters) as key}
                            <label class="toggle-row">
                                <input type="checkbox" bind:checked={behavFilters[key]} onchange={onFilterChange}> 
                                <span>{key}</span>
                            </label>
                        {/each}
                    </div>
                </div>
            </details>

            <details class="filter-section">
                <summary>Node Filters (Entities)</summary>
                <div class="filter-content">
                    <label class="toggle-row" style="font-weight:bold; border-bottom: 1px solid #555; padding-bottom: 5px; margin-bottom: 5px;">
                        <input type="checkbox" bind:checked={allNodesChecked} onchange={toggleAllNodes}>
                        <span>All Nodes</span>
                    </label>
                    <div class="filters-grid">
                        {#each Object.keys(nodeFilters) as key}
                            <label class="toggle-row">
                                <input type="checkbox" bind:checked={nodeFilters[key]} onchange={onFilterChange}> 
                                <span title={key === 'Module' || key === 'Enum' ? 'Hiding this node will also hide its content' : ''}>
                                    {key} {key === 'Module' || key === 'Enum' ? ' ⚠️' : ''}
                                </span>
                            </label>
                        {/each}
                    </div>
                    <div style="font-size: 0.8em; color: #aaa; margin-top: 10px; line-height: 1.3; max-width: 300px;">
                        ⚠️ Hiding a "container" node (e.g., Module, Enum) will automatically hide all nodes contained within it.
                    </div>
                </div>
            </details>
        </div>
    {/if}
</div>

<div id="sidebar" class:active={sidebarActive}>
    {#if selectedNode}
        <h3 id="sb-type">{selectedNode.type}</h3>
        <h2 id="sb-name">{selectedNode.name}</h2>
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
        <div class="sidebar-stat" style="margin-top: 15px; cursor: pointer; display: flex; align-items: center;" onclick={() => inDegreeExpanded = !inDegreeExpanded} role="button">
            <span style="display: flex; align-items: center; gap: 5px;">In-Degree (Receives from) 
                <span style="display: flex;">
                    {#if inDegreeExpanded}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    {:else}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                    {/if}
                </span>
            </span>
            <strong id="sb-indegree">{selectedNode.indegree}</strong>
        </div>
        {#if inDegreeExpanded && selectedNode.inEdges.length > 0}
            <div class="edge-list">
                {#each selectedNode.inEdges as edge}
                    <div class="edge-item">
                        <span class="edge-label">{edge.label}</span> from 
                        <span class="edge-node" title={edge.sourceId}>{edge.sourceLabel}</span>
                    </div>
                {/each}
            </div>
        {/if}

        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
        <div class="sidebar-stat" style="cursor: pointer; display: flex; align-items: center;" onclick={() => outDegreeExpanded = !outDegreeExpanded} role="button">
            <span style="display: flex; align-items: center; gap: 5px;">Out-Degree (Points to) 
                <span style="display: flex;">
                    {#if outDegreeExpanded}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"></polyline></svg>
                    {:else}
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"></polyline></svg>
                    {/if}
                </span>
            </span>
            <strong id="sb-outdegree">{selectedNode.outdegree}</strong>
        </div>
        {#if outDegreeExpanded && selectedNode.outEdges.length > 0}
            <div class="edge-list">
                {#each selectedNode.outEdges as edge}
                    <div class="edge-item">
                        <span class="edge-label">{edge.label}</span> to 
                        <span class="edge-node" title={edge.targetId}>{edge.targetLabel}</span>
                    </div>
                {/each}
            </div>
        {/if}
        <div id="sb-extra">
            <strong>Absolute Path:</strong><br/>{selectedNode.id}
            {#if selectedNode.parent}
                <br/><br/><strong>Parent Module:</strong><br/>{selectedNode.parent}
            {/if}
        </div>
    {/if}
</div>

<button class="legendBtn" title="Show Legend" onclick={() => isLegendOpen = true}>?</button>

{#if isLegendOpen}
    <LegendModal onClose={() => isLegendOpen = false} />
{/if}

<div id="cy" bind:this={cyContainer}></div>
