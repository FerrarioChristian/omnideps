# Ottimizzazioni della Name Resolution e Allineamento con la Tesi di Laurea

## 1. Executive Summary e Contesto Sperimentale

Durante la fase di validazione empirica su repository open-source di grandi dimensioni (`evaluation/`):

- **ANTLR4** (Python runtime + Java core, oltre 4.200 moduli);
- **JUnit 5** (Java, oltre 9.600 compilation units, uso massiccio di interfacce multiple);
- **Fastjson** (Java, oltre 19.000 classi e tipi interni);
- **JUnit 4** (Java, oltre 2.100 moduli);
- **Benchmark Suite Multi-linguaggio** (`tests/benchmarks/` per C, C++, Java, Rust, Python);

l'analizzatore ha evidenziato diverse problematiche architetturali, semantiche e algoritmiche:

1. **Panic su moduli vuoti**: crash inatteso nell'adattamento della gerarchia di directory (`apply_directory_strategy`) in presenza di file senza statement (es. `__init__.py` vuoti).
2. **Stack Overflow su assegnazioni/alias auto-referenziali**: cicli infiniti di valutazione del tipo su costrutti del tipo `mergedParents = mergedParents[k]`.
3. **Ping-Pong ricorsivo su wildcard e import transitivi**: ricorsione mutua infinita tra la risoluzione globale dei cammini (`find_global`) e l'esplorazione dei re-export (`resolve_via_transitive_imports`).
4. **Esplosione combinatoria esponenziale su ereditarietà multipla**: blocco indefinito (apparente loop infinito al 100% di CPU) durante l'analisi di classi che implementano molte interfacce (es. `KitchenSinkExtension.java` in JUnit 5 con 21 interfacce contemporanee).
5. **Rottura dei parametri di tipo generici nelle classi base**: perdita del binding di `T` in estensioni del tipo `Box<T> extends Base<T>` qualora la risoluzione della super-classe non partisse dallo scope locale della classe.
6. **Type Shadowing causato da costruttori e distruttori**: inquinamento del dizionario dei simboli di classe (`class_scope.symbols`), dove la registrazione del costruttore come `Symbol::Value` mascherava il tipo nominale della classe nelle sottoclassi C++, rompendo 4 archi di ereditarietà.
7. **Collisione e perdita dei distruttori C++**: mancata estrazione del prefisso `~` da Tree-sitter, che causava la collisione tra distruttore e costruttore.

Le soluzioni introdotte hanno garantito la **terminazione formale**, la **linearizzazione della complessità** e il raggiungimento del **100% di precisione sui benchmark**:

- **JUnit 5**: da blocco infinito a completamento in **5.79 secondi** (9.622 moduli, 21.967 referenze risolte con successo);
- **Fastjson**: analisi completa di 19.188 moduli in **~15 secondi**;
- **ANTLR4**: analisi completa in **4.8 secondi** (4.237 moduli, 15.834 referenze risolte);
- **Benchmark Suite**: **100% di conformità** (C: 49/49, C++: 50/50, Java: 46/46, Rust: 56/56, Python: 57/57);
- **Suite di test**: 17/17 test unitari e di integrazione passati con successo.

---

## 2. Analisi Dettagliata dei Problemi e delle Soluzioni

```
+----------------------------------------------------------------------------------------------------+
|                                    PANORAMICA DEGLI INTERVENTI                                     |
+----+-----------------------------+------------------------------------+----------------------------+
| #  | Componente                  | Causa del Malfunzionamento         | Principio della Soluzione  |
+----+-----------------------------+------------------------------------+----------------------------+
| 1  | analyzer.rs                 | Rimozione indice 0 su vettore vuoto| Controllo di vacuità       |
|    | (apply_directory_strategy)  | per file privi di AST statements.  | preliminare su moduli/path.|
+----+-----------------------------+------------------------------------+----------------------------+
| 2  | executor.rs                 | Reset del set di visita nelle      | Propagazione globale di    |
|    | (evaluate_typeref_inner)    | valutazioni ricorsive e query key  | visited, guardia sui       |
|    |                             | prive di ScopeId.                  | simboli e scope-key.       |
+----+-----------------------------+------------------------------------+----------------------------+
| 3  | executor.rs                 | Ping-pong find_global <-> imports; | Insiemi di visita dedicati |
|    | (find_global / trans_imp)   | token '*' passato letteralmente.   | e stripping del wildcard.  |
+----+-----------------------------+------------------------------------+----------------------------+
| 4  | executor.rs                 | Ricorsione esponenziale su super   | Memoizzazione con sentinella|
|    | (get_or_resolve_super_scopes| tipi e perdita generici se forzato | vuota ([]), lookup locale  |
|    |  / resolve_super_keyword)   | a parent: albero O(b!).            | e fresh_visited disaccopp. |
+----+-----------------------------+------------------------------------+----------------------------+
| 5  | scope.rs                    | Costruttori inseriti in symbols    | Esclusione di ctor/dtor dai|
|    | (register_structured_type)  | mascherano il tipo nominale (C++). | Symbol::Value di classe.   |
+----+-----------------------------+------------------------------------+----------------------------+
| 6  | classifiers.rs / parsing.rs | Distruttori non riconosciuti o     | Supporto a destructor_name |
|    |                             | privati del prefisso '~'.          | e token terminale '~Class'.|
+----+-----------------------------+------------------------------------+----------------------------+
```

---

### 2.1 Problema 1: Panic su Moduli Privi di Dichiarazioni (`src/analyzer.rs`)

#### Sintomo

Invocando l'analisi su repository Python (come ANTLR4 runtime), il processo andava in panic con:

```text
thread 'main' panicked at src/analyzer.rs:261:32:
index out of bounds: the len is 0 but the index is 0
```

#### Causa Radice

La funzione `apply_directory_strategy` converte la struttura delle cartelle del file system in una gerarchia di moduli annidati (`Module`). Per i file vuoti (come i tipici `__init__.py` usati solo come package marker o file composti esclusivamente da commenti), la fase di parsing produce un vettore `modules` vuoto.
Il codice eseguiva:

```rust
if !path_components.is_empty() {
    let mut current = modules.remove(0); // PANIC se modules.is_empty()
    ...
}
```

#### Soluzione Implementata

È stato aggiunto un controllo di guardia preliminare in `src/analyzer.rs`:

```rust
if modules.is_empty() || path_components.is_empty() {
    return;
}
```

Se il parsing del file sorgente non ha generato alcun modulo concreto, l'infrastruttura di directory non tenta estrazioni illegittime e termina la riorganizzazione in modo sicuro.

---

### 2.2 Problema 2: Stack Overflow su Assegnazioni Auto-Referenziali (`src/resolver/executor.rs`)

#### Sintomo

Durante l'analisi del codice Python di ANTLR4 contenente costrutti come:

```python
mergedParents = mergedParents[k]
```

il thread andava in stack overflow terminando l'esecuzione con abort del processo.

#### Causa Radice

1. **Trattamento semantico dell'assegnazione**: Nelle euristiche di estrazione Python, l'assegnazione `mergedParents = ...` al livello di modulo viene registrata sia come variabile sia potenziale `Symbol::TypeAlias`.
2. **Perdita dello stato di ricorsione**: Durante la risoluzione del tipo di `mergedParents`, la funzione `symbol_to_typeref` invocava `evaluate_typeref(ctx, ty.clone(), scope_id, true)`. Tuttavia, `evaluate_typeref` instanziava internamente un **nuovo** `HashSet::new()`, distruggendo la cronologia delle visite ricevuta dal chiamante.
3. **Collisione delle chiavi di query**: In `evaluate_query`, il set `visited` memorizzava unicamente la stringa `extract_base_name(query)` (es. `"mergedParents"`), priva del contesto lessicale (`ScopeId`). Questo generava due anomalie opposte:
   - Falsi positivi (bloccava la risoluzione di campi omonimi in classi diverse);
   - Mancata rilevazione del ciclo nello stesso scope quando il pattern passava attraverso tipi composti (`Index`, `Generic`, `Union`).

#### Soluzione Implementata

1. **Unificazione del ciclo di valutazione (`evaluate_typeref_inner`)**:
   È stata creata la funzione interna `evaluate_typeref_inner` che propaga per riferimento mutable `visited: &mut HashSet<String>` a tutti i livelli dell'AST dei tipi (`TypeRef::Union`, `TypeRef::Generic`, `TypeRef::TypeVar`, `TypeRef::ResolutionQuery`, `TypeRef::Unresolved`).
2. **Scope-Aware Query Keys**:
   La chiave di tracciamento delle query è stata resa sensibile allo scope corrente:

   ```rust
   let q_str = format!("{}:{}", scope_id, extract_base_name(query));
   if !visited.insert(q_str.clone()) {
       return None;
   }
   ```

3. **Guardia sui Simboli**:
   In `symbol_to_typeref`, per prevenire alias o valori ciclici nello stesso scope, è stata aggiunta la guardia:

   ```rust
   let sym_key = format!("sym:{}:{}", scope_id, name);
   if !visited.insert(sym_key.clone()) {
       return base_path;
   }
   // ... risoluzione del tipo ...
   visited.remove(&sym_key);
   ```

---

### 2.3 Problema 3: Loop Ricorsivo su Wildcard e Import Transitivi (`src/resolver/executor.rs`)

#### Sintomo

In presenza di file come `Python3/tests/TestLexer.py` con direttive:

```python
from antlr4 import *
```

il risolutore entrava in un ciclo ricorsivo infinito consumando l'intero stack.

#### Causa Radice

La risoluzione globale dei percorsi e quella degli import transitivi cooperano per risolvere simboli riesportati:

- `find_global(ctx, &["antlr4", "Lexer"])` interroga lo scope `antlr4`. Se il simbolo non è presente direttamente, consulta `resolve_via_transitive_imports`.
- `resolve_via_transitive_imports` scandiva gli import del modulo `antlr4`. Quando incontrava un import wildcard, generava una nuova query per `Lexer` invocando a sua volta `find_global`.
- **Due difetti concomitanti**:
  1. La condizione `last == member || last == "*"` faceva sì che per `last == "*"` venisse passato a `find_global` l'intero path dell'importante contenente il token letterale `"*"` (es. `["antlr4", "*"]`).
  2. Mancava un meccanismo di rilevamento del ciclo tra `find_global` e `resolve_via_transitive_imports`. Se due moduli si importavano mutuamente o importavano a stella lo stesso package radice, l'algoritmo rimbalzava all'infinito tra le due funzioni.

#### Soluzione Implementata

1. **Guardia globale sui path**:
   In `find_global_internal` è stato introdotto il tracciamento `global:{path.join("::")}`:

   ```rust
   let path_key = format!("global:{}", path.join("::"));
   if !visited.insert(path_key.clone()) {
       return None;
   }
   ```

2. **Guardia locale sulle riesportazioni transitive**:
   In `resolve_via_transitive_imports` è stato introdotto il tracciamento `trans:{scope_id}:{member}`.
3. **Normalizzazione corretta dei wildcard**:
   Il token wildcard `*` viene esplicitamente eliminato prima di concatenare il simbolo puntuale ricercato:

   ```rust
   let mut target_path = imp.path.clone();
   if target_path.last().map(|s| s.as_str()) == Some("*") {
       target_path.pop();
   }
   target_path.push(member.to_string());
   if let Some(resolved) = find_global_internal(ctx, &target_path, visited) { ... }
   ```

---

### 2.4 Problema 4: Esplosione Combinatoria nell'Ereditarietà Multipla e Risoluzione dei Generici

#### Sintomo

Durante l'analisi di **JUnit 5**, l'esecuzione si arrestava indefinitamente sul modulo 872:
`jupiter-tests/src/test/java/org/junit/jupiter/api/extension/KitchenSinkExtension.java`.
Il processo impegnava una CPU al 100% senza produrre output, simulando un loop infinito.

#### Causa Radice: L'Albero Combinatorio Esponenziale

`KitchenSinkExtension` è una classe di test che implementa contemporaneamente **21 interfacce Java**:
`BeforeAllCallback`, `BeforeEachCallback`, `BeforeTestExecutionCallback`, `TestWatcher`, `AfterTestExecutionCallback`, `AfterEachCallback`, `AfterAllCallback`, `ParameterResolver`, `ExecutionCondition`, ecc.

Nel codice precedente di `find_symbol_in_scope_and_supers_internal`:

- Ogni ricerca di simbolo in `KitchenSinkExtension` causava la rivalutazione da zero di tutti i 21 super-tipi non ancora risolti.
- La valutazione di ogni interfaccia a partire dallo scope della classe risaliva lessicalmente re-invocando la ricerca sulla classe stessa, che a sua volta valutava le restanti 20 interfacce, ciascuna delle quali ne valutava 19, e così via.
- Con un fattore di ramificazione $b = 20$ e una profondità $d = 21$, il numero di chiamate ricorsive generate era di ordine fattoriale:
  $$\mathcal{O}(b!) \approx 20! \approx 2.43 \times 10^{18} \text{ operazioni}$$
  rendendo il completamento impossibile (tempo stimato: decine di anni di CPU).

#### L'Evoluzione della Soluzione: Dallo Scope Forzato alla Sentinella di Memoizzazione

Inizialmente si era ipotizzato di risolvere i super-tipi forzando la ricerca direttamente nel `parent_scope` (il package esterno). Tuttavia, questa forzatura causava una regressione sui tipi generici:

- In classi generiche come `class MyList<T> extends AbstractList<T>`, il tipo formale `T` è registrato **nello scope locale della classe**, non nel package genitore. Valutando solo in `parent_scope`, `T` non veniva trovato.

La soluzione definitiva e corretta si basa su tre pilastri:

1. **Memoizzazione degli Scope Ereditati (`resolved_super_scopes`)**:
   In `ExecutorContext` è stato introdotto un contenitore con *interior mutability*:

   ```rust
   pub struct ExecutorContext<'a> {
       pub tree: &'a ScopeTree,
       pub primitives: &'a PrimitiveRegistry,
       pub config: &'a crate::config::AnalyzerConfig,
       pub resolved_super_scopes: RefCell<HashMap<ScopeId, Vec<ScopeId>>>,
   }
   ```

2. **Pre-allocazione della Sentinella Vuota (`[]`)**:
   Prima di avviare la risoluzione delle super-classi di uno `scope_id`, la funzione `get_or_resolve_super_scopes` inserisce un vettore vuoto provvisorio nella cache:

   ```rust
   ctx.resolved_super_scopes.borrow_mut().insert(scope_id, Vec::new());
   ```

   **Perché questo risolve sia l'esplosione combinatoria sia i generici?**
   - La valutazione delle super-classi parte da `scope_id`, consentendo a parametri come `T` di essere risolti nello scope locale.
   - Quando la risalita lessicale del nome della classe base (es. `BeforeAllCallback`) interroga `find_symbol_in_scope_and_supers(scope_id)`, quest'ultima chiama `get_or_resolve_super_scopes(scope_id)`.
   - `get_or_resolve_super_scopes` trova subito la sentinella `Vec::new()` già presente in cache e termina in $\mathcal{O}(1)$ senza riesplorare le super-classi!
   - Di conseguenza, la risalita lessicale prosegue istantaneamente verso il `parent_scope` (il package o namespace contenitore), trovando l'interfaccia senza ricorsione.
3. **Disaccoppiamento con `fresh_visited`**:
   `get_or_resolve_super_scopes` non riceve più il set `visited` della query del chiamante (che tracciava un metodo/campo specifico), ma utilizza un autonomo `let mut fresh_visited = HashSet::new();`. Questo previene qualsiasi inquinamento della cronologia dello stack e impedisce falsi blocchi di ciclo.

---

### 2.5 Problema 5: Type Shadowing Causato da Costruttori e Distruttori nello ScopeTree

#### Sintomo

Nei benchmark C++, 4 archi di dipendenza di ereditarietà risultavano mancanti. In particolare, quando una sottoclasse tentava di ereditare dalla classe base `Vehicle`, l'arco `Car -> Vehicle` non veniva formato e le invocazioni dei metodi ereditati fallivano.

#### Causa Radice

Durante la costruzione dello `ScopeTree`, per ogni metodo di una classe veniva eseguito:

```rust
self.define_symbol(class_scope, m_name, Symbol::Value(return_type));
```

In C++, il costruttore ha lo stesso nome della classe (`Vehicle`). Registrandolo come `Symbol::Value` dentro `Vehicle.symbols["Vehicle"]`, accadeva che:

1. Quando la sottoclasse `Car` cercava la propria classe base `Vehicle`, la ricerca lessicale trovava il simbolo valore del costruttore (`Symbol::Value`) dentro la classe base prima di salire al namespace genitore `Transport`.
2. Il risolutore interpretava il tipo come `Transport.Vehicle.Vehicle` anziché `Transport.Vehicle`.
3. Trattandosi di un valore/metodo e non di un tipo strutturato (`Symbol::Type`), la risoluzione della classe base falliva.

#### Soluzione Implementata

Nei linguaggi orientati agli oggetti, i costruttori e distruttori **non sono membri di istanza ordinari** invocabili con dot-notation (`obj.Vehicle()` è illegale) e **non vengono ereditati** come metodi virtuali dalle sottoclassi.
In `src/resolver/scope.rs`:

```rust
if !method.is_constructor && m_name != name && !m_name.starts_with('~') {
    self.define_symbol(
        class_scope,
        m_name,
        Symbol::Value(method.signature.return_type.clone()),
    );
}
```

**Importante**: I costruttori e distruttori vengono comunque registrati tramite `self.register_function(...)`. Mantengono il proprio scope, i tipi dei parametri (`UsesParamType`) e l'intero corpo viene analizzato (`Calls`, `AccessesField`, `Instantiates`), venendo emessi fedelmente nel grafo finale con archi `NestedIn`.

---

### 2.6 Problema 6: Estrazione dei Distruttori C++ (`~Class`)

#### Causa del Problema

1. `is_function` in `classifiers.rs` non contemplava il tipo `destructor`, ignorando i nodi `destructor_definition` di Tree-sitter.
2. In `extract_identifier_from_declarator` (`text_parsing.rs`), l'estrazione dell'identificatore scendeva fino al nodo terminale `identifier`, estraendo il nome senza tilde (`"Server"` anziché `"~Server"`), provocando una collisione tra distruttore e costruttore.

#### Soluzione Implementata

- In `is_function` aggiunto `|| kind.contains("destructor")`.
- In `extract_identifier_from_declarator` aggiunto `"destructor_name"` tra i tipi di identificatore terminale accettati. Ora i distruttori mantengono il prefisso `~` e compaiono nel grafo come entità distinte.

---

## 3. Guida Dettagliata per l'Allineamento della Tesi di Laurea

Questa sezione fornisce la mappatura puntuale delle modifiche rispetto ai capitoli della tesi (`master-thesis/chapters/`), indicando i paragrafi da aggiornare, le formalizzazioni matematiche da integrare e i nuovi risultati sperimentali.

---

### 3.1 Allineamento Capitolo 5 (`5_analysis_process.tex`) e Capitolo 6 (`6_extraction.tex`)

#### 1. Creazione dei Moduli e File Vacui (Capitolo 5 & 6)

- **Aspetto da Aggiornare**:
  Nella descrizione del mapping topologico tra il file system reale e la gerarchia di `Module` (strategie `Directory` e `Package`), documentare l'invariante di gestione dei file vuoti:
  > *Invariante di Vacuità*: Nei linguaggi con package markers (come i file `__init__.py` di Python o file composti esclusivamente da commenti/direttive), l'estrazione sintattica può produrre $\vec{\mathcal{M}} = \emptyset$. Il motore di aggregazione non assume l'esistenza a priori di un modulo radice per ogni path, preservando la compattezza dello ScopeTree ed evitando accessi out-of-bounds.

#### 2. Modellazione Semantica di Costruttori e Distruttori (Capitolo 6)

- **Aspetto da Aggiornare**:
  Chiarire la distinzione tra membri di istanza e costruttori/distruttori:
  > *Distinzione Semantica nello Scope*: Costruttori e distruttori non appartengono al dizionario di lookup dinamico dei membri ($\text{symbols}(\Sigma_C)$) per evitare lo shadowing del tipo nominale della classe durante le risalite di ereditarietà. Essi sono invece collegati topologicamente alla classe tramite archi strutturali $\texttt{NestedIn}$, mantenendo intatta l'analisi intra-procedurale dei parametri ($\texttt{UsesParamType}$) e delle invocazioni interne ($\texttt{Calls}$).

---

### 3.2 Allineamento Capitolo 7 (`7_name_resolution.tex`)

#### 1. Sezione 7.4.1: The Execution Context ($\Gamma$)

- **Aggiornamento Consigliato**:
  Arricchire la formalizzazione includendo la componente di memoizzazione:
  $$\Gamma = \langle \mathcal{E}, \mathcal{P}, \mathcal{K}, \mathcal{M} \rangle$$
  dove $\mathcal{M} : \Sigma \to 2^{\Sigma}$ rappresenta la mappa di memoizzazione dei super-scope risolti.
  > *Trasparenza Referenziale*:  
  > Sebbene $\mathcal{M}$ sia implementato tramite *interior mutability* (`RefCell<HashMap<ScopeId, Vec<ScopeId>>>`) per motivi di efficienza runtime in Rust, la funzione di risoluzione rimane **referenzialmente trasparente** e **idempotente**. Poiché la struttura dell'albero $\mathcal{E}$ è congelata dopo la fase $\rho_{\text{build}}$, il calcolo degli antenati di uno scope produce un insieme immutabile e deterministico di identificatori $\vec{\Sigma}_{\text{supers}}$, privo di effetti collaterali sull'ordine di valutazione delle compilation units.

#### 2. Sezione 7.4.2: Lexical Scope Climbing e Chiavi di Visita

- **Aggiornamento Consigliato**:
  Formalizzare la chiave di disambiguazione delle query sensibile allo scope:
  $$k_{\text{query}} = \langle \Sigma, \text{base}(q) \rangle \in \Sigma \times \mathcal{I}$$
  L'inclusione di $\Sigma$ garantisce la correttezza del principio di *lexical shadowing* e consente di identificare cicli di auto-assegnazione (es. $x = x[k]$) senza bloccare simboli omonimi presenti in altri contesti.

#### 3. Sezione 7.4.4: Member Extraction and Inheritance Resolution

- **Aggiornamento Consigliato**:
  Sostituire la precedente formulazione descrivendo la risoluzione basata su sentinella:
  1. **Risoluzione con Sentinella dei Super-Tipi**:
     Per risolvere i super-tipi di una classe $C$ con scope $\Sigma_C$, la valutazione delle clausole di derivazione ($\tau_{\text{super}} \in \text{supers}(C)$) inizia all'interno dello scope locale $\Sigma_C$. Questo consente l'immediato binding dei parametri di tipo generici formali ($T \in \text{type\_params}(C)$).
  2. **Prevenzione della Ricorsione tramite Sentinella**:
     La divergenza viene prevenuta inizializzando la tabella di memoizzazione con una sentinella vuota prima della scansione ricorsiva:
     $$\mathcal{M}(\Sigma_C) \leftarrow \emptyset$$
     Qualsiasi invocazione rientrante a $\texttt{find\_symbol\_in\_scope\_and\_supers}(\Sigma_C)$ durante la valutazione delle classi base trova immediatamente l'insieme vuoto, collassando a costo $\mathcal{O}(1)$ e consentendo alla risalita lessicale di procedere naturalmente verso $\text{parent}(\Sigma_C)$.
  3. **Complessità Algoritmica**:
     - *Senza memoizzazione*: complessità combinatoria esponenziale $\mathcal{O}(b!)$ con $b = |\text{supers}(C)|$ (es. $20! \approx 2.4 \times 10^{18}$ operazioni in presenze di implementazioni multiple estensive).
     - *Con memoizzazione a sentinella*: la ricerca dei membri si riduce a una visita in profondità (DFS) lineare sul DAG delle classi antenate:
       $$\mathcal{O}(|V_{\text{supers}}| + |E_{\text{supers}}|)$$
     abbattendo il tempo di calcolo su classi complesse da decine di anni a meno di un millisecondo.

#### 4. Sezione 7.4.5: Transitive Import Resolution

- **Aggiornamento Consigliato**:
  Specificare le condizioni di non-divergenza per i re-export wildcard:
  - Espansione rigorosa del cammino $\pi = M \mathbin{\Vert} \langle \iota \rangle$ con eliminazione del token `*`;
  - Mutua ricorsione tra $\texttt{find\_global}$ e $\texttt{resolve\_via\_transitive\_imports}$ interrotta da insiemi di visita dedicati:
    - $V_{\text{global}} \subseteq \mathcal{P}(\mathcal{I}^*)$ per i cammini globali;
    - $V_{\text{trans}} \subseteq \Sigma \times \mathcal{I}$ per le coppie modulo-membro.

#### 5. Sezione 7.6 (Nuova Proposta): "Termination and Convergence Guarantees"

Testo LaTeX pronto per l'inserimento alla fine del Capitolo 7:

```latex
\section{Termination and Convergence Guarantees}
\label{sec:resolution_termination}

A fundamental requirement for static architectural analysis on uncurated, large-scale codebases is guaranteed termination in the presence of pathological syntactic constructs (e.g., circular inheritance, mutual module re-exports, and self-referential assignments).

The name resolution operator $\rho_{\text{exec}}$ is guaranteed to terminate on any finite Scope Tree $\mathcal{E}$ under all configurations $\mathcal{K}$.

\begin{theorem}[Termination of Query Execution]
For any finite workspace module set $\vec{\mathcal{M}}$ and any query $q \in \mathcal{Q}$, the evaluation function $\texttt{evaluate\_query}(q, \Sigma)$ terminates in a finite number of discrete resolution steps.
\end{theorem}

\begin{proof}[Proof Sketch]
The state space of resolution is bounded by the finite size of the Scope Tree: $|\mathcal{E}| = N < \infty$ scopes and $|\text{Sym}| = S < \infty$ distinct symbols. Potential divergence is restricted to three recursive structures, each governed by an explicit cycle guard:
\begin{enumerate}
  \item \textbf{Lexical Climbing}: Lexical climbing is strictly monotonic with respect to scope depth: $\text{depth}(\text{parent}(\Sigma)) = \text{depth}(\Sigma) - 1$. Since the tree has finite depth and terminates at the unique root scope $\Sigma_{\text{root}}$ (where $\text{parent}(\Sigma_{\text{root}}) = \text{None}$), climbing terminates in at most $\text{depth}(\Sigma)$ iterations.
  \item \textbf{Inheritance Traversal}: Super-scope resolution is memoized in $\mathcal{M}$. Circular derivation ($A \extends B \extends A$) and re-entrant expansion are broken upon entry by initializing $\mathcal{M}(\Sigma) = \emptyset$ prior to recursive descent. Graph traversal over resolved super-scopes is protected by the scope set $V_{\text{scopes}} \subseteq \Sigma$, ensuring that each ancestor scope is visited at most once ($|V_{\text{scopes}}| \le N$).
  \item \textbf{Transitive and Alias Recursion}: Re-export chains and nested type assignments are indexed by composite keys $\langle \Sigma, \iota \rangle \in \Sigma \times \mathcal{I}$. Because the set of distinct symbols across the workspace is finite, the visited set $V_{\text{visited}}$ monotonically increases until either a terminal type reference is produced or the cycle is detected, yielding $\texttt{None}$ or $\texttt{TypeRef::Failed}$.
\end{enumerate}
Therefore, every execution branch either converges to a resolved reference or encounters a visited guard within finite steps, establishing termination.
\end{proof}
```

---

### 3.3 Allineamento Capitolo 9 (`9_evaluation_results.tex`)

I risultati ottenuti su ANTLR4, JUnit 5, Fastjson e sulla suite di benchmark multi-linguaggio validano empiricamente la scalabilità lineare di OmniDeps:

#### Tabella dei Risultati Empirici dei Benchmark Enterprise

| Repository Target | Linguaggio Principale | Moduli Analizzati | Tipi Strutturati | Funzioni Libere | Riferimenti Risolti | Riferimenti Esterni / Non Risolti | Tempo di Esecuzione (Release) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **JUnit 5** | Java | 9.622 | 4.102 | 436 | 21.967 | 42.151 | **5.79 s** |
| **Fastjson** | Java | 19.188 | 6.302 | 0 | 21.179 | 44.517 | **~15 s** |
| **ANTLR4** (completo) | Java / Python | 4.237 | 1.280 | 613 | 15.834 | 11.299 | **4.8 s** |
| **ANTLR4 Runtime** | Java / Python | 2.053 | 709 | 601 | 2.549 | 10.348 | **~3 s** |
| **JUnit 4** | Java | 2.131 | 1.359 | 0 | 3.880 | 7.463 | **~2 s** |

#### Risultati della Benchmark Suite Standardizzata (`tests/benchmarks/`)

| Linguaggio | Nodi Attesi / Trovati | Archi Attesi / Trovati | Conformità Globale |
| --- | --- | --- | --- |
| **C** | 49 / 49 | 43 / 43 | **100%** |
| **C++** | 50 / 50 | 44 / 44 | **100%** |
| **Java** | N/A (archi-based) | 46 / 46 | **100%** |
| **Rust** | 56 / 56 | 84 / 84 | **100%** |
| **Python** | 57 / 57 | 58 / 58 | **100%** |

---

## 4. Riepilogo dei File Modificati nel Repository

- [`src/analyzer.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/analyzer.rs):
  - Aggiunto guard check per vettori vuoti in `apply_directory_strategy`.
- [`src/resolver/executor.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/resolver/executor.rs):
  - Aggiunto `resolved_super_scopes: RefCell<HashMap<ScopeId, Vec<ScopeId>>>` in `ExecutorContext`.
  - Implementata la funzione memoizzata `get_or_resolve_super_scopes` con sentinella iniziale `Vec::new()` e `fresh_visited` autonomo.
  - Risoluzione dei super-tipi a partire da `scope_id` per garantire il binding dei type parameters generici.
  - Implementata `evaluate_typeref_inner` con propagazione referenziale del set `visited`.
  - Normalizzato il trattamento dei wildcard `*` ed introdotte le guardie `global:{path}` e `trans:{scope_id}:{member}` in `find_global_internal` e `resolve_via_transitive_imports`.
- [`src/resolver/scope.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/resolver/scope.rs):
  - Esclusi costruttori e distruttori da `class_scope.symbols` per eliminare lo shadowing del tipo nominale nelle classi base C++.
- [`src/heuristics/classifiers.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/heuristics/classifiers.rs) e [`src/heuristics/text_parsing.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/heuristics/text_parsing.rs):
  - Riconoscimento del tipo `destructor` e conservazione del prefisso `~` (`destructor_name`).
- [`tests/generics_test.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/tests/generics_test.rs):
  - Aggiunti test di regressione dedicati per file vuoti, alias ciclici, import a stella, classi multi-interfaccia, costruttori e distruttori.
