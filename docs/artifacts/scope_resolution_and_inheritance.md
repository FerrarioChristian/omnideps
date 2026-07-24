# Risoluzione dello Scope ed Ereditarietà: Implementazione

Questo documento approfondisce le logiche avanzate introdotte per mappare accuratamente il codice in linguaggi complessi come Java e Rust, superando i limiti delle prime versioni dell'analizzatore.

## 1. Risoluzione delle Variabili Locali e dello Scope (IMPLEMENTATO)
Quando analizziamo il corpo di una funzione o di un metodo (in un costrutto `Block`), l'estrattore individua chiamate a metodi (`method_invocation`) o accessi a campi (`field_access`). Per sapere a *quale* tipo strutturato appartiene quel metodo o campo, dobbiamo capire di che tipo è la variabile su cui viene invocato.

### Soluzione Implementata: `ScopeTree`
Il problema è stato risolto con l'introduzione della nuova architettura V4 **Scope Tree**. È stata implementata una gerarchia lessicale rigorosa allocata su una `Arena`:
1. **Scope Globale/Modulo**: dove vivono classi, struct, funzioni libere.
2. **Scope della Classe/Struct**: dove vivono i campi (proprietà) e i metodi dell'oggetto.
3. **Scope della Funzione (Locale)**: dove vivono i parametri e le variabili dichiarate localmente.

Quando l'analizzatore incontra un'espressione come `myVar.doSomething()`, il Query Engine risolve `myVar` sfruttando il *Lexical Climbing*: parte dallo scope più interno (il blocco di codice corrente), risale alla funzione, poi alla classe, e infine al modulo. Trovato il tipo originario (`StructA`), mappa correttamente la dipendenza architetturale `Calls` verso `StructA.doSomething`.

## 2. Ereditarietà e Trait Methods (IMPLEMENTATO)
L'ereditarietà in Java (`extends`, `implements`) e in Rust (`impl Trait for Struct`) permette a un oggetto di chiamare metodi che non sono stati definiti direttamente all'interno della classe o struct, ma ereditati.

### Soluzione Implementata: Risoluzione Polimorfica
La risoluzione dei metodi ereditati è ora pienamente supportata dal Query Engine:
1. Durante la scansione dello `ScopeTree`, se il metodo non si trova direttamente nel nodo (es. `StructA`)...
2. ...l'algoritmo ispeziona ricorsivamente la lista dei `super_types` (la lista di astrazioni parenti generate dagli archi `IsA`).
3. Risalendo l'albero di ereditarietà, giunge al `TraitA` (o alla SuperClasse), trova lì il `trait_method()` e risolve l'arco con successo, collegando il chiamante all'implementazione generica corretta.

## 3. Pattern `Deref` in Rust (LAVORO FUTURO)
In Rust esiste un tratto speciale chiamato `Deref`. Viene usato massicciamente per gli "smart pointer" (es. `Box<T>`, `Arc<T>`, o wrapper custom). Quando un tipo `MyWrapper` implementa `Deref<Target=StructC>`, tutte le chiamate a metodi non trovati su `MyWrapper` vengono instradate automaticamente da Rust verso `StructC`.

### Implementazione Futura
Per simulare questo comportamento dell'AST di Rust, l'analizzatore dovrà in futuro:
1. Riconoscere l'arco di ereditarietà speciale verso il target del `Deref` (spesso espresso con un alias o trait assocciato `Target`).
2. Adottare la stessa logica polimorfica usata per l'ereditarietà normale e attualmente operativa, "scivolando" in modo trasparente sul target del Deref per risolvere il metodo chiamato.

Questi miglioramenti riducono a zero i falsi negativi derivati da chiamate "sepolte" in variabili complesse o classi ereditate.
