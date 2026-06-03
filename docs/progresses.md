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

## 2. Relazioni e Dipendenze (Grafo $\mathcal{G}$)
- [x] **IsA** (Ereditarietà di classi/struct)
  - *Note:* Pienamente funzionante per Rust, Java, Python, e C++. Ho aggiornato l'astrazione in `extract_super_types` affinché consideri esplicitamente `superclass`, `interfaces` (Java), `superclasses` (Python) e itera sui child di tipo `base_class_clause` per il C++.
- [x] **UsesFieldType** (Il tipo di un campo dati)
  - *Note:* Riscontrato in Rust. In C le struct annidate o i puntatori `struct Type` generano failed refs a causa degli spazi nel nome.
- [x] **UsesParamType** (Il tipo di un parametro di funzione/metodo)
  - *Note:* Funzionante in Rust e negli altri linguaggi.
- [x] **UsesReturnType** (Il tipo di ritorno di una funzione/metodo)
  - *Note:* Risolto il fallback bug che assegnava il nome della funzione stessa come tipo di ritorno quando questo non era esplicitato (es. Python/constructor) impostandolo correttamente a `Primitive(Void)`.
- [x] **NestedIn** (Tipi definiti all'interno di altri tipi o metodi)
  - *Note:* Il concetto funziona: come implementato per `ImplBlock`, le struct come `Breed` dentro `impl Cat` vengono appiattite e diventano `nested_types` di `Cat`.
- [x] **ModuleContainment** (Appartenenza a un modulo/namespace)
  - *Note:* Funziona per Rust e C++.

## 3. Casi Limite e Costrutti Complessi
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
