<script>
    let { onClose } = $props();
</script>

<style>
    .legend-modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(4px);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 2000;
    }
    .legend-modal-content {
        background: rgba(30, 30, 30, 0.85);
        backdrop-filter: blur(10px);
        border: 1px solid #444;
        border-radius: 12px;
        width: 80vw;
        height: 80vh;
        max-width: 1000px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.8);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        color: #ecf0f1;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
    }
    .legend-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.6rem 1rem;
        border-bottom: 1px solid #444;
        background: rgba(20, 20, 20, 0.5);
    }
    .legend-header h1 {
        margin: 0;
        font-size: 1.5rem;
        color: #fff;
    }
    .close-btn {
        background: none;
        border: none;
        color: #bdc3c7;
        font-size: 24px;
        cursor: pointer;
        transition: color 0.2s;
        padding: 0;
        line-height: 1;
    }
    .close-btn:hover {
        color: #fff;
    }
    .legend-body {
        padding: 1rem;
        overflow-y: auto;
        flex-grow: 1;
        line-height: 1.6;
    }
    
    .legend-body h2 {
        font-size: 1.2em;
        color: #3498db;
        margin-top: 1.2rem;
        margin-bottom: 1rem;
    }
    .legend-body h2:first-child {
        margin-top: 0;
    }
    
    .legend-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
        gap: 1rem;
    }
    .legend-card {
        background: rgba(20, 20, 20, 0.5);
        border: 1px solid #444;
        border-radius: 12px;
        padding: 1rem;
        display: flex;
        align-items: flex-start;
        gap: 20px;
    }
    .visual-element {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        font-weight: bold;
        color: white;
        text-align: center;
    }
    .description {
        flex-grow: 1;
    }
    .description h3 {
        margin: 0 0 8px 0;
        font-size: 1em;
        color: #bdc3c7;
    }
    .description p {
        margin: 0;
        font-size: 0.8em;
        color: #aab2bd;
    }

    /* Node Shapes */
    .node-module {
        width: 140px;
        height: 80px;
        background-color: rgba(44, 62, 80, 0.3);
        border: 2px dashed #7f8c8d;
        color: #fff;
        border-radius: 4px;
        align-items: flex-start;
        padding-top: 10px;
    }
    .node-struct {
        width: 80px;
        height: 40px;
        background-color: rgba(41, 128, 185, 0.15);
        border: 2px solid #2980b9;
        border-radius: 8px;
    }
    .node-enum {
        width: 80px;
        height: 40px;
        background-color: rgba(232, 67, 147, 0.15); /* #e84393 with 15% opacity to match container logic if needed, but the user didn't explicitly ask for background change, I'll keep it transparent or just match the 15% like others */
        border: 2px solid #e84393;
        border-radius: 8px;
    }
    .node-trait {
        width: 80px;
        height: 40px;
        background-color: rgba(39, 174, 96, 0.15);
        border: 2px solid #27ae60;
        border-radius: 8px;
    }
    .node-function {
        width: 60px;
        height: 60px;
        background-color: #8e44ad;
        border-radius: 50%;
    }
    .node-field {
        width: 80px;
        height: 40px;
        background-color: #16a085;
        border-radius: 8px;
    }
    .node-static {
        width: 80px;
        height: 40px;
        background-color: #f1c40f;
        color: #333;
        border-radius: 8px;
    }
    .node-enum-variant {
        width: 80px;
        height: 40px;
        background-color: #e84393;
        border-radius: 8px;
    }
    .node-external {
        width: 80px;
        height: 40px;
        background-color: #34495e;
        opacity: 0.7;
        border: 1px solid #95a5a6;
        border-radius: 8px;
    }
    .node-primitive {
        width: 80px;
        height: 80px;
        background-color: #1abc9c;
        clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
    }
    .node-type-alias {
        width: 80px;
        height: 80px;
        background-color: #9b59b6;
        clip-path: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%);
    }

    /* Arrows (Edges) */
    .edge-container {
        width: 100px;
        height: 40px;
        position: relative;
    }
    .edge-line {
        position: absolute;
        top: 50%;
        left: 0;
        width: 85%;
        transform: translateY(-50%);
    }
    .edge-arrow {
        position: absolute;
        top: 50%;
        right: 0;
        transform: translateY(-50%);
        border-top: 6px solid transparent;
        border-bottom: 6px solid transparent;
    }
    
    .edge-isa .edge-line { height: 3px; background-color: #e74c3c; }
    .edge-isa .edge-arrow { border-left: 10px solid #e74c3c; }

    .edge-implements .edge-line { height: 3px; border-top: 3px dashed #f39c12; background: transparent; }
    .edge-implements .edge-arrow { border-left: 10px solid #f39c12; }

    .edge-calls .edge-line { height: 2px; background-color: #2ecc71; }
    .edge-calls .edge-arrow { border-left: 8px solid #2ecc71; }

    .edge-instantiates .edge-line { height: 2px; border-top: 2px dashed #3498db; background: transparent; }
    .edge-instantiates .edge-arrow { border-left: 8px solid #3498db; }

    .edge-uses .edge-line { height: 2px; border-top: 2px dotted #95a5a6; background: transparent; opacity: 0.7; }
    .edge-uses .edge-arrow { border-left: 8px solid #95a5a6; opacity: 0.7; }

    .edge-accesses .edge-line { height: 2px; border-top: 2px dashed #e67e22; background: transparent; }
    .edge-accesses .edge-arrow { border-left: 8px solid #e67e22; }

    .edge-imports .edge-line { height: 2px; background-color: #8e44ad; }
    .edge-imports .edge-arrow { border-left: 8px solid #8e44ad; }
    
    .edge-nested .edge-line { height: 1px; background-color: #7f8c8d; opacity: 0.5; }
    .edge-nested .edge-arrow { border-left: 6px solid #7f8c8d; opacity: 0.5; }

    .edge-caststo .edge-line { height: 2px; border-top: 2px dashed #00a8ff; background: transparent; }
    .edge-caststo .edge-arrow { border-left: 8px solid #00a8ff; }

    .edge-aliases .edge-line { height: 2px; border-top: 2px dotted #ff00ff; background: transparent; }
    .edge-aliases .edge-arrow { border-left: 8px solid #ff00ff; }
    
    .edge-annotated .edge-line { height: 2px; border-top: 2px dotted #ff00ff; background: transparent; }
    .edge-annotated .edge-arrow { border-left: 8px solid #ff00ff; }
    
    .interaction-box {
        background: rgba(20, 20, 20, 0.5);
        border: 1px solid #444;
        padding: 1rem;
        border-radius: 12px;
        margin-top: 1rem;
    }
    .interaction-box h3 { color: #3498db; margin-top: 0; margin-bottom: 0.5rem; }
    .interaction-box p, .interaction-box ul { font-size: 0.9em; color: #aab2bd; }
</style>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="legend-modal-backdrop" onclick={onClose} role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="legend-modal-content" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
        <div class="legend-header">
            <h1>Architectural Legend</h1>
            <button class="close-btn" onclick={onClose} title="Close">✕</button>
        </div>
        <div class="legend-body">
            <h2>1. Software Components (Nodes)</h2>
            <div class="legend-grid">
                <div class="legend-card">
                    <div class="visual-element node-module">Module</div>
                    <div class="description">
                        <h3>Module / Namespace</h3>
                        <p>A Compound Node (Bounding Box). Modules act as transparent containers. Any class or function declared inside a module will be visually placed within it.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-struct">Struct/Class</div>
                    <div class="description">
                        <h3>Structured Type (Classes, Structs)</h3>
                        <p>Represents a data structure. These are blue nodes with rounded edges.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-enum">Enum</div>
                    <div class="description">
                        <h3>Enum Container</h3>
                        <p>Represents an enumeration. Dashed magenta/pink outline, contains individual variants inside.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-trait">Interface</div>
                    <div class="description">
                        <h3>Interface / Trait</h3>
                        <p>Represents a pure behavioral contract (e.g., <code>interface</code> in Java, <code>trait</code> in Rust). The green highlighted border distinguishes them from concrete implementations.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-function">Function</div>
                    <div class="description">
                        <h3>Function / Method</h3>
                        <p>Purple circular nodes. They represent flattened methods within classes or free functions within modules.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-field">Field</div>
                    <div class="description">
                        <h3>Struct / Class Field</h3>
                        <p>Teal nodes. They represent class attributes in Java/Python or struct fields in Rust/C.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-static">Static<br/>Variable</div>
                    <div class="description">
                        <h3>Static / Global Variable</h3>
                        <p>Yellow nodes. They represent global variables or constants defined at the module level.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-enum-variant">Enum<br/>Variant</div>
                    <div class="description">
                        <h3>Enum Variant</h3>
                        <p>Solid magenta/pink nodes. They represent the individual options (variants) within an Enum.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-external">External</div>
                    <div class="description">
                        <h3>External Dependency</h3>
                        <p>Gray and semi-transparent. Represent components detected via <code>import</code>, but not defined within the scanned files (e.g., Standard Library classes or external packages).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-primitive">Primitive</div>
                    <div class="description">
                        <h3>Primitive Type</h3>
                        <p>Turquoise hexagons. Represent the fundamental language types (e.g., <code>int</code>, <code>String</code>, <code>boolean</code>) that are directly referenced by the code.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-type-alias">Type<br/>Alias</div>
                    <div class="description">
                        <h3>Type Alias</h3>
                        <p>Magenta diamonds. Represent type aliases (e.g., <code>type</code> in Python/TypeScript, <code>typedef</code> in C).</p>
                    </div>
                </div>
            </div>

            <h2>2. Architectural Relationships (Edges)</h2>
            <div class="legend-grid">
                <div class="legend-card">
                    <div class="visual-element edge-container edge-isa">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>IsA (Inheritance)</h3>
                        <p>A thick solid red line. Represents the strongest OOP dependency: class inheritance or strict interface/trait implementation.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-implements">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Implements</h3>
                        <p>Thick dashed orange/yellow line. Represents the formal implementation of an Interface or Trait.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-calls">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Calls (Invocation)</h3>
                        <p>Solid green line. Drawn when the body of a function or method explicitly invokes another.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-instantiates">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Instantiates (Creation)</h3>
                        <p>Dashed blue line. Indicates explicit memory allocation of a Class or Struct (e.g., calling the <code>new</code> constructor).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-accesses">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>AccessesField</h3>
                        <p>Dashed orange line. Drawn when a function or method attempts to read or write a given field (e.g., <code>object.field</code>).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-uses">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Uses (Structural Dependency)</h3>
                        <p>Thin dotted gray line. Divides into <code>UsesFieldType</code>, <code>UsesParamType</code>, and <code>UsesReturnType</code>. Indicates a lightweight coupling: class A has a parameter, return type, or data field of type B.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-imports">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Imports (Inclusion)</h3>
                        <p>Solid purple line. Generated by explicit import/use declarations at the module level.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-caststo">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>CastsTo (Type Conversion)</h3>
                        <p>Dashed blue (cerulean) line. Generated by explicit type casts or conversions.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-aliases">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Aliases (Type Alias)</h3>
                        <p>Dotted magenta line. Generated by a TypeAlias pointing to its original target type.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-annotated">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>AnnotatedWith (Annotations/Decorators)</h3>
                        <p>Dotted magenta line, used to indicate that an entity has been decorated or annotated with the target component (e.g., <code>@Autowired</code> or <code>#[derive]</code>).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-nested">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>NestedIn / ModuleContainment</h3>
                        <p>Thin, semi-transparent gray line. Indicates the physical/logical containment of one element inside another (e.g., nested class, file in a module).</p>
                    </div>
                </div>
            </div>

            <div class="interaction-box">
                <h3>Interactive Visual Debugging</h3>
                <p>The Cytoscape interface supports interactive focus. <strong>By clicking or tapping on any node</strong>, the rest of the graph will become semi-transparent and fade into the background.</p>
                <p>Only the following will remain visible:</p>
                <ul>
                    <li>The clicked node.</li>
                    <li>All nodes that <strong>receive</strong> a dependency from it (its direct dependencies).</li>
                    <li>All nodes that <strong>point</strong> to it (components depending on it).</li>
                    <li>The container Module (Bounding Box) so the visual context is not lost.</li>
                </ul>
                <p><em>To restore the full graph, simply click on an empty space on the screen (background).</em></p>
            </div>
        </div>
    </div>
</div>
