# Risoluzione dello Scope ed Ereditarietà: Logiche Architetturali

Questo documento descrive le logiche implementate per mappare accuratamente il codice sorgente in linguaggi dotati di scope complessi e sistemi a ereditarietà multipla o a tratti (es. Java e Rust). L'obiettivo è superare le limitazioni di precisione riscontrate nei sistemi di analisi legacy.

## 1. Risoluzione Lessicale delle Variabili Locali (`ScopeTree`)

Durante l'analisi del corpo di una funzione o di un metodo (costrutto `Block`), l'estrattore individua comunemente chiamate a metodi (`method_invocation`) o accessi a campi (`field_access`). Identificare a quale tipo strutturato appartenga la destinazione logica richiede l'inferenza accurata del tipo della variabile originaria.

### Architettura Implementata
Il tracciamento lessicale è garantito dall'architettura del **Query Engine V4**, imperniato sulla struttura dati `ScopeTree`. Questo albero, allocato deterministicamente su una *Memory Arena*, modella una rigida gerarchia lessicale:
1. **Scope di Modulo (Globale)**: Contiene la dichiarazione di classi, struct e funzioni libere (o *free variables*).
2. **Scope di Classe/Struct**: Contiene i campi (proprietà) e i metodi dell'oggetto.
3. **Scope di Funzione (Locale)**: Contiene i parametri in ingresso e le variabili dichiarate all'interno del blocco.

**Flusso di Risoluzione (*Lexical Climbing*):**
All'incontro di un'espressione come `myVar.doSomething()`, l'executor risolve il prefisso `myVar` sfruttando il meccanismo di risalita lessicale. L'algoritmo interroga lo scope più interno (il blocco di codice corrente); se non trova la dichiarazione, risale allo scope genitore (la funzione), poi alla classe, e infine al modulo. Una volta individuato il tipo originario della variabile (es. `StructA`), il motore mappa correttamente una dipendenza architetturale `Calls` mirata all'entità `StructA.doSomething`.

---

## 2. Risoluzione Polimorfica: Ereditarietà e Trait Methods

Sistemi a ereditarietà nativa (Java tramite `extends/implements`) o a estensione comportamentale (Rust tramite `impl Trait for Struct`) permettono alle istanze di invocare metodi non definiti intrinsecamente nel corpo della classe genitrice. 

### Architettura Implementata
La risoluzione dei metodi ereditati è interamente delegata alle funzionalità di esplorazione polimorfica del Query Engine:
1. Durante la valutazione di una query algebrica sullo `ScopeTree`, l'algoritmo ispeziona prima i membri diretti del nodo target (es. `StructA`).
2. In assenza di match, il risolutore interroga ricorsivamente la lista dei `super_types` (le astrazioni genitrici generate e collegate tramite archi `IsA`).
3. Attraversando l'albero di ereditarietà in profondità, il sistema raggiunge l'entità originaria (es. `TraitA` o `SuperClass`), risolve il costrutto e collega il chiamante all'implementazione astratta corretta, preservando il tracciamento del *Dynamic Dispatch*.

---

## 3. Pattern Speciali: `Deref` in Rust (Lavoro Futuro)

Nel linguaggio Rust, il tratto `Deref` introduce un livello ulteriore di complessità. È utilizzato massicciamente per l'implementazione degli *Smart Pointers* (es. `Box<T>`, `Arc<T>` o wrapper custom). Quando un tipo `MyWrapper` implementa `Deref<Target=StructC>`, il compilatore inietta chiamate implicite per instradare le invocazioni dei metodi non trovati su `MyWrapper` direttamente verso `StructC`.

### Proposta di Implementazione
Per simulare questo comportamento in analisi statica, le future evoluzioni dell'analizzatore dovranno:
1. Intercettare semanticamente l'implementazione del trait `Deref` nell'AST, instanziando un arco di ereditarietà speciale puntato verso il tipo definito nell'alias `Target`.
2. Sfruttare la logica polimorfica attualmente in uso (descritta nel paragrafo 2) per "scivolare" trasparentemente sul bersaglio del Deref durante la risoluzione del metodo.

Questa integrazione minimizzerà i falsi negativi architetturali derivanti da chiamate mascherate da wrapper complessi, portando l'analizzatore a eguagliare il control-flow del compilatore stesso.
