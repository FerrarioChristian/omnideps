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

    /* Forme Nodi */
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

    /* Frecce (Archi) */
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
            <h1>Legenda Architetturale</h1>
            <button class="close-btn" onclick={onClose} title="Chiudi">✕</button>
        </div>
        <div class="legend-body">
            <h2>1. Componenti Software (Nodi)</h2>
            <div class="legend-grid">
                <div class="legend-card">
                    <div class="visual-element node-module">Module</div>
                    <div class="description">
                        <h3>Modulo / Namespace</h3>
                        <p>Un Nodo Composto (Bounding Box). I moduli agiscono come raccoglitori trasparenti (<em>scatole cinesi</em>). Qualsiasi classe o funzione dichiarata all'interno di un modulo verrà visualizzata fisicamente al suo interno.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-struct">Struct/Class</div>
                    <div class="description">
                        <h3>Tipo Strutturato (Classi, Struct)</h3>
                        <p>Rappresenta una struttura dati. Sono i nodi blu dai bordi smussati.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-enum">Enum</div>
                    <div class="description">
                        <h3>Enum Container</h3>
                        <p>Rappresenta una enumerazione. Contorno magenta/rosa tratteggiato, contiene le singole varianti al suo interno.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-trait">Interface</div>
                    <div class="description">
                        <h3>Interfaccia / Trait</h3>
                        <p>Rappresenta un contratto comportamentale puro (es. <code>interface</code> in Java, <code>trait</code> in Rust). Il verde con bordo in risalto li distingue dalle implementazioni concrete.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-function">Function</div>
                    <div class="description">
                        <h3>Funzione / Metodo</h3>
                        <p>Nodi circolari viola. Rappresentano metodi appiattiti all'interno delle classi, o funzioni libere all'interno dei moduli.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-field">Field</div>
                    <div class="description">
                        <h3>Campo di Struct / Classe</h3>
                        <p>Nodi color foglia di tè (teal). Rappresentano attributi di classe in Java/Python o campi di una struct in Rust/C.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-static">Static<br/>Variable</div>
                    <div class="description">
                        <h3>Variabile Statica / Globale</h3>
                        <p>Nodi gialli. Rappresentano variabili globali o costanti definite a livello di modulo.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-enum-variant">Enum<br/>Variant</div>
                    <div class="description">
                        <h3>Variante Enum</h3>
                        <p>Nodi magenta/rosa pieni. Rappresentano le singole opzioni (varianti) all'interno di una Enum.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-external">External</div>
                    <div class="description">
                        <h3>Dipendenza Esterna</h3>
                        <p>Grigi e semi-trasparenti. Rappresentano componenti che l'analizzatore ha rilevato tramite <code>import</code>, ma che non sono definiti all'interno dei file scansionati (es. classi della Standard Library o pacchetti esterni).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element node-primitive">Primitive</div>
                    <div class="description">
                        <h3>Tipo Primitivo</h3>
                        <p>Esagoni turchesi. Rappresentano i tipi fondamentali del linguaggio (es. <code>int</code>, <code>String</code>, <code>boolean</code>) che vengono referenziati direttamente dal codice.</p>
                    </div>
                </div>
            </div>

            <h2>2. Relazioni Architetturali (Archi)</h2>
            <div class="legend-grid">
                <div class="legend-card">
                    <div class="visual-element edge-container edge-isa">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>IsA (Ereditarietà)</h3>
                        <p>Una spessa linea rossa continua. Rappresenta la dipendenza più forte dell'OOP: ereditarietà di classi o implementazione rigorosa di un'interfaccia/trait.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-implements">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Implements</h3>
                        <p>Linea arancione/gialla tratteggiata spessa. Rappresenta l'implementazione formale di un'Interfaccia o di un Trait.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-calls">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Calls (Invocazione)</h3>
                        <p>Linea verde continua. Tracciata quando il body di una funzione o metodo ne invoca esplicitamente un altro.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-instantiates">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Instantiates (Creazione)</h3>
                        <p>Linea blu tratteggiata. Segnala l'allocazione esplicita in memoria di una Classe o Struct (es. chiamata al costruttore <code>new</code>).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-accesses">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>AccessesField (Accesso a Campo)</h3>
                        <p>Linea arancione tratteggiata. Tracciata quando una funzione o metodo tenta di leggere o scrivere un campo dato (es. <code>oggetto.campo</code>).</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-uses">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Uses (Dipendenza Strutturale)</h3>
                        <p>Linea grigia sottile e puntinata. Si divide in <code>UsesFieldType</code>, <code>UsesParamType</code>, e <code>UsesReturnType</code>. Indica un accoppiamento leggero: la classe A possiede un parametro, un ritorno o un campo dati di tipo B.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-imports">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>Imports (Inclusione)</h3>
                        <p>Linea viola solida. Generata dalle dichiarazioni di importazione/use esplicite a livello di modulo.</p>
                    </div>
                </div>
                <div class="legend-card">
                    <div class="visual-element edge-container edge-nested">
                        <div class="edge-line"></div>
                        <div class="edge-arrow"></div>
                    </div>
                    <div class="description">
                        <h3>NestedIn / ModuleContainment</h3>
                        <p>Linea grigia sottile e semi-trasparente. Indica il contenimento fisico/logico di un elemento dentro un altro (es. classe annidata, file nel modulo).</p>
                    </div>
                </div>
            </div>

            <div class="interaction-box">
                <h3>Visual Debugging Interattivo</h3>
                <p>L'interfaccia di Cytoscape supporta il focus interattivo. <strong>Facendo Click o Tap su un qualsiasi nodo</strong>, tutto il resto del grafo diventerà semi-trasparente e passerà in secondo piano.</p>
                <p>Rimarranno visibili solo:</p>
                <ul>
                    <li>Il nodo cliccato.</li>
                    <li>Tutti i nodi che <strong>ricevono</strong> una dipendenza da lui (le sue dipendenze dirette).</li>
                    <li>Tutti i nodi che <strong>puntano</strong> a lui (i componenti che dipendono da lui).</li>
                    <li>Il Modulo contenitore (Bounding Box) per non perdere il contesto visivo.</li>
                </ul>
                <p><em>Per ripristinare il grafo completo, basta cliccare in uno spazio vuoto dello schermo (background).</em></p>
            </div>
        </div>
    </div>
</div>
