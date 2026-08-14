<script>
    import { onMount, onDestroy } from 'svelte';
    import cytoscape from 'cytoscape';
    import fcose from 'cytoscape-fcose';
    import style from '$lib/cytoscape_style.js';
    import LegendModal from '$lib/LegendModal.svelte';

    let { elements = [], rawOutput = null, statusMessage = '', controls } = $props();

    let cy = null;
    let cyContainer = null;

    let isPanelCollapsed = $state(false);
    let isLegendOpen = $state(false);

    let selectedNode = $state(null);
    let sidebarActive = $state(false);
    
    let inDegreeExpanded = $state(false);
    let outDegreeExpanded = $state(false);

    // Filters
    let structChecked = $state(true);
    let behavChecked = $state(true);
    
    let structFilters = $state({
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
    
    let behavFilters = $state({
        'Calls': true,
        'Instantiates': true,
        'AccessesField': true
    });

    let allNodesChecked = $state(false);
    let nodeFilters = $state({
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
        'External': true
    });

    let searchQuery = $state('');

    onMount(() => {
        if (typeof window !== 'undefined' && !window.__cy_fcose_registered) {
            cytoscape.use(fcose);
            window.__cy_fcose_registered = true;
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
            style: style
            // layout will be run after filters
        });

        cy.on('tap', 'node', function(e) {
            let node = e.target;
            let neighborhood = node.closedNeighborhood();
            let context = neighborhood.union(node.ancestors()).union(node.descendants());
            
            cy.elements().addClass('dimmed');
            context.removeClass('dimmed');

            let inEdges = [];
            node.incomers('edge').forEach(e => {
                inEdges.push({
                    label: e.data('label'),
                    sourceId: e.source().data('id'),
                    sourceLabel: e.source().data('label') || e.source().data('id').split('::').pop()
                });
            });
            
            let outEdges = [];
            node.outgoers('edge').forEach(e => {
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

        cy.on('tap', function(e) {
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
                cy.edges(edgeSelector).filter(ele => {
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

        const matched = cy.nodes().filter(function(ele) {
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
        <button class="toggle-panel-btn" onclick={() => isPanelCollapsed = true} title="Nascondi Menu">
            ✕
        </button>
    </div>
    <div style="margin-bottom: 15px;">
        <input type="text" id="node-search-box" bind:value={searchQuery} oninput={handleSearch} placeholder="Cerca nodo..." autocomplete="off" spellcheck="false" />
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
                <summary>Filtri Archi (Relazioni)</summary>
                <div class="filter-content">
                    <label class="toggle-row" style="font-weight:bold; border-bottom: 1px solid #555; padding-bottom: 5px; margin-bottom: 5px;">
                        <input type="checkbox" bind:checked={structChecked} onchange={toggleStructural}>
                        <span>Strutturali (IsA, Uses, Imports...)</span>
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
                        <span>Comportamentali (Calls, Instantiates...)</span>
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
                <summary>Filtri Nodi (Entità)</summary>
                <div class="filter-content">
                    <label class="toggle-row" style="font-weight:bold; border-bottom: 1px solid #555; padding-bottom: 5px; margin-bottom: 5px;">
                        <input type="checkbox" bind:checked={allNodesChecked} onchange={toggleAllNodes}>
                        <span>Tutti i Nodi</span>
                    </label>
                    <div class="filters-grid">
                        {#each Object.keys(nodeFilters) as key}
                            <label class="toggle-row">
                                <input type="checkbox" bind:checked={nodeFilters[key]} onchange={onFilterChange}> 
                                <span title={key === 'Module' || key === 'Enum' ? 'Nascondendo questo nodo si nasconderà anche il suo contenuto' : ''}>
                                    {key} {key === 'Module' || key === 'Enum' ? ' ⚠️' : ''}
                                </span>
                            </label>
                        {/each}
                    </div>
                    <div style="font-size: 0.8em; color: #aaa; margin-top: 10px; line-height: 1.3; max-width: 300px;">
                        ⚠️ Nascondere un nodo "contenitore" (es. Module, Enum) nasconderà automaticamente anche tutti i nodi contenuti al suo interno.
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
            <span style="display: flex; align-items: center; gap: 5px;">In-Degree (Riceve da) 
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
                        <span class="edge-label">{edge.label}</span> da 
                        <span class="edge-node" title={edge.sourceId}>{edge.sourceLabel}</span>
                    </div>
                {/each}
            </div>
        {/if}

        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_interactive_supports_focus -->
        <div class="sidebar-stat" style="cursor: pointer; display: flex; align-items: center;" onclick={() => outDegreeExpanded = !outDegreeExpanded} role="button">
            <span style="display: flex; align-items: center; gap: 5px;">Out-Degree (Punta a) 
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
                        <span class="edge-label">{edge.label}</span> verso 
                        <span class="edge-node" title={edge.targetId}>{edge.targetLabel}</span>
                    </div>
                {/each}
            </div>
        {/if}
        <div id="sb-extra">
            <strong>Percorso Assoluto:</strong><br/>{selectedNode.id}
            {#if selectedNode.parent}
                <br/><br/><strong>Modulo Padre:</strong><br/>{selectedNode.parent}
            {/if}
        </div>
    {/if}
</div>

<button class="legendBtn" title="Mostra Legenda" onclick={() => isLegendOpen = true}>?</button>

{#if isLegendOpen}
    <LegendModal onClose={() => isLegendOpen = false} />
{/if}

<div id="cy" bind:this={cyContainer}></div>
