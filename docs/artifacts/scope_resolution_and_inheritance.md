# Risoluzione dello Scope ed Ereditarietà (Punto 2)

Questo documento approfondisce le logiche avanzate di cui abbiamo discusso, attualmente accantonate ma fondamentali per mappare accuratamente il codice in linguaggi complessi come Java e Rust.

## 1. Risoluzione delle Variabili Locali e dello Scope
Quando analizziamo il corpo di una funzione o di un metodo (in un costrutto `Block`), l'estrattore individua chiamate a metodi (`method_invocation`) o accessi a campi (`field_access`). Per sapere a *quale* tipo strutturato appartiene quel metodo o campo, dobbiamo capire di che tipo è la variabile su cui viene invocato.

### Problema Attuale
Attualmente, se troviamo un'espressione come `myVar.doSomething()`, l'analizzatore potrebbe non riuscire a dedurre che `myVar` è di tipo `StructA`, e quindi fallisce nel creare un arco `Calls` verso `StructA.doSomething`.

### Soluzione: Tabella dei Simboli (Scope Tree)
Bisogna implementare una gerarchia di **Scope**:
1. **Scope Globale/Modulo**: dove vivono classi, struct, funzioni libere.
2. **Scope della Classe/Struct**: dove vivono i campi (proprietà) dell'oggetto.
3. **Scope della Funzione (Locale)**: dove vivono i parametri e le variabili dichiarate localmente.
Quando l'analizzatore incontra `myVar.doSomething()`, cercherà `myVar` partendo dallo scope più interno (le variabili locali), poi tra i campi della classe (se in un metodo), fino a trovare il suo tipo. Trovato il tipo (`StructA`), mapperà correttamente la dipendenza.

## 2. Ereditarietà e Trait Methods
L'ereditarietà in Java (`extends`, `implements`) e in Rust (`impl Trait for Struct`) permette a un oggetto di chiamare metodi che non sono stati definiti direttamente all'interno della classe o struct, ma ereditati.

### Problema Attuale
Se `StructA` implementa `TraitA` (che definisce il metodo `trait_method()`), una chiamata a `struct_a_instance.trait_method()` potrebbe fallire la risoluzione se l'analizzatore cerca il metodo solo all'interno del blocco principale di `StructA`.

### Soluzione: Risoluzione Polimorfica
Durante la risoluzione del grafo:
1. Se il metodo non si trova direttamente nel nodo `StructA`...
2. ...l'algoritmo ispeziona gli archi in uscita di tipo `Implements` o `Extends`.
3. Arriva a `TraitA` (o `SuperClass`), trova lì il `trait_method()` e risolve l'arco con successo puntando al metodo del tratto/superclasse.

## 3. Pattern `Deref` in Rust
In Rust esiste un tratto speciale chiamato `Deref`. Viene usato massicciamente per gli "smart pointer" (es. `Box<T>`, `Arc<T>`, o wrapper custom). Quando un tipo `MyWrapper` implementa `Deref<Target=StructC>`, tutte le chiamate a metodi non trovati su `MyWrapper` vengono instradate automaticamente da Rust verso `StructC`.

### Implementazione
Per simulare questo comportamento dell'AST di Rust, l'analizzatore dovrà:
1. Riconoscere l'arco di ereditarietà speciale verso il target del `Deref` (spesso espresso con un alias o trait assocciato `Target`).
2. Adottare la stessa logica polimorfica usata per l'ereditarietà normale, "scivolando" in modo trasparente sul target del Deref per risolvere il metodo chiamato.

Questi miglioramenti ridurranno a zero i falsi negativi derivati da chiamate "sepolte" in variabili complesse o classi ereditate.
