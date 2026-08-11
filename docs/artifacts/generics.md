# Implementazione Formale dei Generics e Template Types

Questo documento descrive lo stato attuale del supporto ai tipi generici (e ai template in C++) all'interno del *language-agnostic-analyzer*, illustrando le soluzioni temporanee (euristiche) attualmente in uso e tracciando una roadmap dettagliata per una futura implementazione formale e architetturalmente corretta.

## 1. Lo Stato Attuale (Soluzione Euristica)

Nella versione iniziale dell'estrattore dei tipi (`extract_type_ref` in `src/heuristics/type_extraction.rs`), un tipo complesso come `std::vector<Car>` veniva analizzato in modo superficiale. L'analizzatore estraeva il testo grezzo e lo "troncava" non appena incontrava una parentesi angolare `<` (tramite uno split testuale). 
Il risultato era che la variabile veniva identificata esclusivamente con il tipo `std::vector`, ignorando del tutto la classe `Car`. Di conseguenza, nel Dependency Graph finale mancavano gli archi di dipendenza tra le collezioni e i tipi in esse contenuti.

Per risolvere rapidamente la mancanza di questi archi (ad esempio per far passare il benchmark C++ senza perdere le dipendenze in collezioni come `Fleet.cars`), è stata adottata una **soluzione euristica basata sui tipi unione**:

1. L'estrattore intercetta i nodi sintattici relativi ai template (come i nodi `template_type` generati dal parser Tree-sitter per C++).
2. Se rileva la sintassi dei generici (`< ... >`), separa la base del tipo (es. `std::vector`) dagli argomenti generici contenuti all'interno (es. `Car`), dividendo eventuali parametri multipli per virgola.
3. Inserisce tutti i tipi individuati all'interno di un costrutto **`TypeRef::Union`** (es. `Union([std::vector, Car])`).

### Perché usare `TypeRef::Union`?
Il costrutto `Union` (nato per gestire sintassi come `A | B` in Python o TypeScript) impone al motore di Name Resolution di cercare e tracciare dipendenze verso **tutti** i tipi elencati nell'unione. Modellando un generic come una Unione, "inganniamo" l'analizzatore forzandolo a creare un arco verso la base (`std::vector`) e uno verso il parametro (`Car`). Questo ripristina la corretta visibilità degli archi di composizione nel grafo.

## 2. I Limiti della Soluzione Attuale

Sebbene la soluzione garantisca l'estrazione degli archi di dipendenza fondamentali, essa presenta limitazioni sia semantiche che strutturali:
* **Inesattezza Semantica:** Un `std::vector<Car>` non significa "un vettore *oppure* una macchina" (come indicherebbe una vera unione logica), bensì un vettore *di* macchine (un'applicazione di un tipo parametrico).
* **Fragilità del Parsing Testuale:** Lo split testuale sulle parentesi `< >` e virgole è fragile e non gestisce adeguatamente generici annidati e complessi (es. `Map<String, List<Car>>` o firme con spazi/ritorni a capo complessi).

## 3. Roadmap per l'Implementazione Formale

Per supportare a pieno titolo e in modo language-agnostic i tipi generici, l'architettura necessita di un intervento strutturale suddiviso in 4 fasi.

### Fase 1: Modifica del Modello Dati (`src/model.rs`)
È necessario estendere l'enum `TypeRef` aggiungendo una variante dedicata per i tipi generici, capace di distinguere la base dai suoi argomenti:

```rust
pub enum TypeRef {
    Primitive(String),
    Unresolved(QualifiedName),
    Union(Vec<TypeRef>),
    // Nuova variante per i Generics
    Generic {
        base: Box<TypeRef>,
        arguments: Vec<TypeRef>,
    },
    Failed(QualifiedName),
}
```

### Fase 2: Estrazione Strutturata AST-Based (`src/heuristics/type_extraction.rs`)
Bisogna abbandonare le manipolazioni testuali basate su `.find('<')`. Invece:
* Identificare i nodi Tree-sitter specifici per i generics (es. `generic_type`, `template_type`, `type_arguments`).
* Esplorare ricorsivamente l'albero sintattico per popolare la nuova struttura `TypeRef::Generic`. Tipicamente, Tree-sitter fornisce un sottonodo per la "base" e una lista di nodi figli all'interno del campo `arguments` o `type_arguments`. Questo garantirà robustezza assoluta anche in caso di annidamenti complessi.

### Fase 3: Cattura delle Definizioni Parametriche (Type Parameters)
Attualmente l'analizzatore esplora le classi e le funzioni per estrarne nome, campi e metodi, ma ignora del tutto i **Type Parameters** al momento della dichiarazione (es. il `<T, K>` in `template <typename T, typename K> class Map`). 
* Occorre aggiungere un campo `type_parameters: Vec<String>` (o un costrutto più complesso se includono limiti/constraints) alle struct di definizione come `StructuredType` e `Function` in `model.rs`.
* Questo permetterà di sapere che una data entità si aspetta dei tipi generici quando viene istanziata, informazione fondamentale per l'analisi del flusso (data-flow).

### Fase 4: Name Resolution e Creazione Archi (`src/resolver/executor.rs`)
L'engine di risoluzione dovrà essere aggiornato per gestire la variante `TypeRef::Generic`:
1. **Risoluzione Multipla:** Quando incontra `TypeRef::Generic`, il resolver dovrà recuperare nel database dei moduli il tipo `base` e, indipendentemente, risolvere ciascun tipo presente negli `arguments`.
2. **Generazione di Archi Precisi:** Anziché un arco generico, l'analyzer potrebbe produrre archi specifici (es. `InstantiatesGeneric` o `UsesGenericArgument`), differenziando la dipendenza dal contenitore da quella del contenuto.
3. **Mappatura Avanzata (Data-Flow Parziale):** Nelle evoluzioni future, avendo sia i "Type Parameters" catturati alla Fase 3 (es. `T`), sia i `TypeRef::Generic` popolati (es. `Car`), sarà possibile costruire un dizionario di sostituzione durante la risoluzione delle variabili locali. In questo modo, l'analyzer saprà dedurre che un'invocazione di `my_list.get()` restituisce un oggetto di tipo concreto `Car` e non un anonimo `T`, permettendo il rilevamento di dipendenze comportamentali (call graph) altrimenti impossibili da dedurre.
