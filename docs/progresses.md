# Analisi delle Funzionalità dell'Architettura

Questo documento traccia lo stato di implementazione e riconoscimento dei vari costrutti architetturali e tipi di dipendenza da parte dell'analizzatore *language-agnostic*.

## 1. Entità Architetturali Riconosciute ($\mathcal{C}$)
- [x] **Moduli / Namespace / Package** (`Module`)
  - *Note:* Funziona bene in Rust (`mod`) e parzialmente in C++ (`namespace`). In Java (packages) e Python (moduli/file) attualmente non produce moduli annidati ma inserisce tutto nel modulo `root`. **Da sistemare per un supporto language-agnostic completo.**
- [x] **Tipi Strutturati** (`StructuredType`):
  - [x] Classi (`Class`)
  - [x] Struct (`Struct`)
  - [x] Interfacce (`Interface`)
  - [x] Trait (`Trait`)
  - *Note:* Estrazione perfetta in Rust. In C++ estrae le classi. In Java estrae Classi e Interfacce. In Python ora le classi vengono estratte correttamente (Risolto bug `!kind.contains("argument")`). In C le struct vengono riconosciute correttamente includendo il parsing ricorsivo.
- [x] **Funzioni e Metodi** (`Function` con `Signature` indipendente)
  - *Note:* Funziona in Rust, Java, e Python. In C++ i metodi venivano estratti senza nome, ma ora l'estrattore cerca all'interno di `function_declarator` per estrarre l'identificatore corretto. In C le funzioni all'interno delle struct tramite puntatore sono correttamente categorizzate come campi.
- [x] **Variabili Globali e Statiche** (`free_variables`)
  - *Note:* Supportate e integrate nell'Indice Globale. I field level-module (come costanti o `static` in Rust) vengono catturati, tipizzati e resi accessibili agli altri componenti. Modificati i renderer (Cytoscape) per mostrarle con uno stile dedicato e distinto dai normali struct fields.

## 2. Relazioni e Dipendenze (Grafo $\mathcal{G}$)
- [x] **IsA** (Ereditarietà di classi/struct)
  - *Note:* Pienamente funzionante per Rust, Java, Python, e C++. Ho aggiornato l'astrazione in `extract_super_types` affinché consideri esplicitamente `superclass`, `interfaces` (Java), `superclasses` (Python) e itera sui child di tipo `base_class_clause` per il C++.
- [x] **UsesFieldType** (Il tipo di un campo dati)
  - *Note:* L'Indice Globale traccia e indicizza i field, permettendo la deduzione del tipo di ritorno anche in accessi annidati.
- [x] **UsesParamType** (Il tipo di un parametro di funzione/metodo)
  - *Note:* Funzionante in Rust e negli altri linguaggi.
- [x] **UsesReturnType** (Il tipo di ritorno di una funzione/metodo)
  - *Note:* Risolto il fallback bug che assegnava il nome della funzione stessa come tipo di ritorno quando questo non era esplicitato (es. Python/constructor) impostandolo correttamente a `Primitive(Void)`.
- [x] **NestedIn** (Tipi definiti all'interno di altri tipi o metodi)
  - *Note:* Il concetto funziona: come implementato per `ImplBlock`, le struct come `Breed` dentro `impl Cat` vengono appiattite e diventano `nested_types` di `Cat`.
- [x] **ModuleContainment** (Appartenenza a un modulo/namespace)
  - *Note:* Funziona per Rust e C++.

## 3. Casi Limite e Costrutti Complessi
- [x] **Type Inference Avanzata (Method Chaining)**
  - *Note:* Pienamente implementata l'inferenza del tipo tramite **Query Engine Algebrico**. Supporta catene complesse (es. `user.get_db().query()`) estraendo algebricamente il tipo di ritorno da ogni nodo intermedio fino al bersaglio finale.
- [x] Appiattimento dei blocchi di implementazione (`ImplBlock` flattening)
  - *Note:* **Pienamente funzionante in Rust.** I metodi vengono passati correttamente al tipo di riferimento.
- [x] Tipi annidati in blocchi di implementazione (es. `struct` dentro `impl` in Rust)
  - *Note:* Funziona perfettamente, appiattito nel target.
- [x] Tipi locali (Classi/Struct definite all'interno di funzioni/metodi)
  - *Note:* Funziona. L'estrattore perlustra correttamente il `body` e `declaration` annidandoli correttamente.
- [x] Ereditarietà multipla o implementazione di interfacce multiple
  - *Note:* In Rust funziona tramite multipli `ImplBlock`. In C++ e Java le interfacce e classi base sono correttamente estratte come `super_types` grazie al fix nell'estrattore di super-tipi.
- [x] Risoluzione dei riferimenti tra moduli diversi (Import / Use / Require)
  - *Note:* Appena implementata l'estrazione logica degli import e la risoluzione tramite "jump points" nel Contesto di Risoluzione ($\Gamma$). Ora quando un tipo non viene trovato localmente, l'analizzatore guarda nelle clausole di importazione (supportando alias ed exact matches) e crea un riferimento corretto invece di bollarlo come fallito.
- [x] Funzioni anonime / Callback / Higher Order Functions / Interazioni Comportamentali (come parametri o ritorni)
  - *Note:* Appena implementata l'estrazione "Best-Effort" dal `body` delle funzioni. Ora l'analizzatore scansiona l'albero per identificare chiamate a funzioni/metodi statici (`Calls`) e istanziazioni di oggetti (`Instantiates`). Le chiamate dinamiche su variabili locali vengono ignorate con grazia fallendo la risoluzione locale, mentre i target statici/qualificati diventano archi preziosi nel grafo.

---

### **Changelog Risoluzioni (Heuristics Fixes)**
- **Feature Analisi Comportamentale**: Aggiunti i campi `calls` e `instantiates` a `Function` nell'IR. Creato `extract_body_dependencies` per navigare l'AST nei body e intercettare `object_creation_expression`, `struct_expression`, `call_expression` ecc., costruendo i rispettivi archi comportamentali.
- **Fix Symbol Table Annidati**: L'indicizzazione di `build_symbol_table` è stata aggiornata per attraversare in profondità i tipi `nested_types`, permettendo ad esempio alle chiamate qualificate profonde (come `Outer.Inner.DeepInner.hello()`) di risolversi con successo.
- **Feature Import/Require**: Aggiunta la struct `Import` all'IR e a $\mathcal{M}$ (formalizzato come $\mathcal{I}_{imports}$). Implementato `try_parse_import` per gestire `use`, `import` e `#include`. Aggiornato il risolutore affinché usi gli import come jump point (percorsi di fallback) per risolvere i riferimenti esterni con successo, supportando gli `alias` testuali dell'AST.
- **Bug C++ Metodi Senza Nome**: Risolto esplorando il nodo child `declarator` se il nodo `function_definition` non dispone di un `name` o `identifier` di primo livello (cosa tipica dell'AST C++).
- **Bug Python Classi non Rilevate**: Rimosso il check restrittivo `!kind.contains("argument")` da `is_structured` in `try_parse_structured_type`. L'AST python definisce le superclassi dentro una `argument_list`, il che invalidava la scansione.
- **Bug Ereditarietà (Super Types) in Java/C++/Python**: L'euristica non riconosceva i nomi dei campi CST specifici dei linguaggi non-Rust. Aggiornata `extract_super_types` in `heuristics.rs` per interrogare esplicitamente `superclass`, `interfaces`, `superclasses` e `base_class_clause`.
- **Bug Falso Positivo Tipi di Ritorno**: In caso di mancato `return_type` esplicito (come nei costruttori Python o C++), l'analizzatore prendeva per sbaglio l'identificatore della funzione come tipo di ritorno. Aggiornata l'euristica per restituire `TypeRef::Primitive(PrimitiveType::Void)` in maniera sicura se fallisce il parse esplicito o tramite la sintassi `->`.
- **Fix conteggio Nested Types nel Summary**: Il resoconto `AnalysisSummary` riportava zero `StructuredType` per Python e C++ nonostante comparissero nel grafo, a causa del fatto che il parser del summary non navigava ricorsivamente nei sottomoduli né contava i `nested_types`. Aggiornato `build_analysis_summary` per scansionare ricorsivamente tutto l'albero.
- **Feature Variabili Globali e Statiche**: Implementata la cattura e la risoluzione delle variabili a livello di modulo (es. `static` in Rust). Aggiornato il `GlobalRegistry` per supportarle e migliorato l'estrattore AST per i blocchi macro/token_tree (es. `println!`) per non perdersi i path intermedi durante il token coalescing. Aggiornati i visualizzatori Cytoscape per differenziarle chiaramente dai campi di struct.

## Aggiustamenti post incontro (Pianificazione)

A seguito dell'incontro con il relatore, sono emersi alcuni punti fondamentali da implementare o rifinire prima di procedere. Di seguito l'analisi punto per punto:

### 1. Gestione Differenziata dei Moduli (Namespace)
- **Problema**: Lingue diverse usano logiche diverse per raggruppare i file (directory-based per Python/Rust vs package/module-based per Java/C++).
- **Strategia decisa**: Implementeremo diverse strategie di risoluzione dei moduli all'interno dell'analizzatore. L'abilitazione della strategia corretta avverrà in base al linguaggio analizzato, governata da un file di configurazione (es. `config.json` o tramite flag/opzioni). Questo approccio diventerà lo standard per gestire tutte le peculiarità comportamentali tra i linguaggi, mantenendo il core *language-agnostic* e delegando la specificità alla configurazione.

### 2. Import Transitivi (Python)
- **Problema**: In Python (e potenzialmente altri linguaggi), se il modulo `A` importa `B`, e `B` importa `C`, il modulo `A` può accedere a componenti di `C` attraverso `B` (es. `B.C.Componente`). Altri linguaggi non permettono questo comportamento di default.
- **Azione**: Dovremo estendere il sistema di `ExecutorContext` per gestire la visibilità e la transitività degli import, probabilmente permettendo di interrogare le esportazioni (e gli import ri-esportati) di un modulo durante la risoluzione, attivando questa funzione solo se la configurazione del linguaggio lo richiede.

### 3. Ereditarietà Ciclica nel MRO (Method Resolution Order)
- **Problema**: L'algoritmo attuale di MRO (usato in `Extract` per cercare i metodi nelle superclassi) potrebbe andare in loop infinito se il codice sorgente presenta ereditarietà ciclica (es. `class A(B)` e `class B(A)`).
- **Soluzioni proposte**:
  1. Forzare l'antisimmetria (rimuovere i cicli) già a livello di IR durante l'estrazione.
  2. Implementare un controllo a runtime durante la risoluzione (es. tenere traccia dei nodi visitati).
- **Scelta/Pianificazione**: L'opzione 2 (tenere traccia dei nodi visitati nella funzione `find_member` dell'executor tramite un parametro `visited: HashSet<QualifiedName>`) è l'approccio più sicuro e robusto, in quanto evita di dover implementare complesse logiche di normalizzazione o graph coloring in fase di parsing iniziale.

### 4. Il termine "Prefix Climbing"
- **Feedback**: Il relatore era dubbioso sull'uso del termine "prefix climbing" nel capitolo dell'Algebra delle Query.
- **Analisi**: Ha ragione. Il termine attuale crea confusione perché mischia l'operazione tecnica su stringhe (la rimozione di un suffisso dal path corrente) con il concetto semantico di risoluzione. In realtà, l'operazione che eseguiamo in `Query::Find` è nota come **Lexical Scope Climbing** (o **Lexical Ascending** / Risalita dello scope lessicale). Questo è il termine corretto nella teoria dei compilatori, dove l'interprete risale la gerarchia degli scope per trovare una dichiarazione. Provvederò ad aggiornare la documentazione e i commenti nel codice per usare la nomenclatura corretta.

### 5. Ottimizzazione del Global Registry (String operations)
- **Feedback**: Manipolare e clonare frequentemente array di stringhe (`Vec<String>`, ossia i `QualifiedName`) per verificare l'esistenza di un percorso nel registro globale può essere pesante dal punto di vista computazionale.
- **Possibile soluzione**: Attualmente il registro è una flat `HashMap<QualifiedName, RegistryEntry>`. Per evitare di manipolare array e stringhe ad ogni lookup, il registro potrebbe essere trasformato in un **Trie** (albero dei prefissi). In questo modo, le query potrebbero navigare i nodi dell'albero direttamente. Alternativamente, si potrebbe utilizzare uno *String Interning* in modo da confrontare interi al posto di stringhe. Segnerò questo punto come un potenziale task di ottimizzazione, utilissimo da riportare in tesi per discutere le scelte architetturali di performance.

### 6. Riconoscimento di `self` e `this`
- **Feedback**: Verificare come l'implementazione gestisce `self` (spesso parametro esplicito come in Python/Rust) vs `this` (spesso implicito come in Java/C++), e se avviene *ahead-of-time* o durante la risoluzione.
- **Verifica effettuata**: L'attuale implementazione in `src/resolver/builder.rs` gestisce **entrambe le convenzioni in modo automatico**, e lo fa *ahead-of-time* (nella Fase 2a di Query Building).
  - Quando si entra nello scope di una `StructuredType`, il Builder inietta *sia* `self` che `this` nello `SymbolStack`, associandoli a una query che punta alla classe corrente. Questo risolve il caso **implicito** (tipico di Java e C++).
  - Quando si entra in una funzione, i suoi parametri vengono analizzati e messi in cima allo `SymbolStack`. Se la funzione ha un parametro esplicito chiamato `self` (tipico di Python e Rust), questo farà *shadowing* del `self` inserito precedentemente dalla struct. Dato che il lookup (`iter().rev()`) cerca sempre dal frame più interno a quello più esterno, prenderà il parametro formale anziché il riferimento implicito della classe.
  - **Conclusione**: L'architettura a scope concentrici supporta naturalmente entrambi i comportamenti senza la necessità di write-code aggiuntivi per casi speciali!
