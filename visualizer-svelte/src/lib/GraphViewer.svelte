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
        'ModuleContainment': true
    });
    
    let behavFilters = $state({
        'Calls': true,
        'Instantiates': true,
        'AccessesField': true
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

            selectedNode = {
                name: node.data('label') || node.data('id').split('::').pop(),
                type: node.data('type') || 'Module',
                indegree: node.indegree(false),
                outdegree: node.outdegree(false),
                id: node.data('id'),
                parent: node.data('parent')
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
            idealEdgeLength: () => 50,
            edgeElasticity: () => 0.45,
            nodeRepulsion: () => 4500,
            gravity: 0.25,
            gravityRange: 3.8,
            gravityCompound: 1.0,
            gravityRangeCompound: 1.5,
        }).run();
    }

    function applyFilters() {
        if (!cy) return;
        
        cy.batch(() => {
            cy.edges().addClass('filtered-out');
            
            let activeFilters = [];
            for (const [key, val] of Object.entries(structFilters)) {
                if (val) activeFilters.push(key);
            }
            for (const [key, val] of Object.entries(behavFilters)) {
                if (val) activeFilters.push(key);
            }
            
            if (activeFilters.length > 0) {
                const selector = activeFilters.map(val => `[label = "${val}"]`).join(', ');
                cy.edges(selector).removeClass('filtered-out');
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

    function onFilterChange() {
        structChecked = Object.values(structFilters).some(v => v);
        behavChecked = Object.values(behavFilters).some(v => v);
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
            <label class="toggle-row">
                <input type="checkbox" bind:checked={structChecked} onchange={toggleStructural}>
                <span>Struttura (IsA, Uses)</span>
            </label>
            <label class="toggle-row">
                <input type="checkbox" bind:checked={behavChecked} onchange={toggleBehavioral}>
                <span>Comportamento (Calls, Instantiates)</span>
            </label>

            <details class="advanced-filters">
                <summary>Filtri Avanzati</summary>
                <div class="filters-grid">
                    {#each Object.keys(structFilters) as key}
                        <label class="toggle-row">
                            <input type="checkbox" bind:checked={structFilters[key]} onchange={onFilterChange}> 
                            <span>{key}</span>
                        </label>
                    {/each}
                    {#each Object.keys(behavFilters) as key}
                        <label class="toggle-row">
                            <input type="checkbox" bind:checked={behavFilters[key]} onchange={onFilterChange}> 
                            <span>{key}</span>
                        </label>
                    {/each}
                </div>
            </details>
        </div>
    {/if}
</div>

<div id="sidebar" class:active={sidebarActive}>
    {#if selectedNode}
        <h3 id="sb-type">{selectedNode.type}</h3>
        <h2 id="sb-name">{selectedNode.name}</h2>
        <div class="sidebar-stat" style="margin-top: 15px;"><span>In-Degree (Riceve da)</span><strong id="sb-indegree">{selectedNode.indegree}</strong></div>
        <div class="sidebar-stat"><span>Out-Degree (Punta a)</span><strong id="sb-outdegree">{selectedNode.outdegree}</strong></div>
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
