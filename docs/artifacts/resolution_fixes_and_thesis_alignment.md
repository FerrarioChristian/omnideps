# Ottimizzazioni della Name Resolution e Allineamento con la Tesi di Laurea


## 1. Executive Summary e Contesto Sperimentale

Durante la fase di validazione empirica su repository open-source di grandi dimensioni (`evaluation/`):
- **ANTLR4** (Python runtime + Java core, oltre 4.200 moduli);
- **JUnit 5** (Java, oltre 9.400 compilation units, uso massiccio di interfacce multiple);
- **Fastjson** (Java, oltre 19.000 classi e tipi interni);
- **JUnit 4** (Java, oltre 2.100 moduli);

l'analizzatore ha evidenziato quattro ordini di problematiche architetturali e algoritmiche che ne impedivano il completamento:
1. **Panic su moduli vuoti**: crash inatteso nell'adattamento della gerarchia di directory (`apply_directory_strategy`) in presenza di file senza statement (es. `__init__.py` vuoti).
2. **Stack Overflow su assegnazioni/alias auto-referenziali**: cicli infiniti di valutazione del tipo su costrutti del tipo `mergedParents = mergedParents[k]`.
3. **Ping-Pong ricorsivo su wildcard e import transitivi**: ricorsione mutua infinita tra la risoluzione globale dei cammini (`find_global`) e l'esplorazione dei re-export (`resolve_via_transitive_imports`).
4. **Esplosione combinatoria esponenziale su ereditarietà multipla**: blocco indefinito (apparente loop infinito al 100% di CPU) durante l'analisi di classi che implementano molte interfacce (es. `KitchenSinkExtension.java` in JUnit 5 con 21 interfacce contemporanee).

Le soluzioni introdotte hanno garantito la **terminazione formale**, la **linearizzazione della complessità** e un drastico abbattimento dei tempi di esecuzione:
- **JUnit 5**: da blocco infinito a completamento in **~8 secondi** (21.656 referenze risolte con successo);
- **Fastjson**: analisi completa di 19.188 moduli in **~15 secondi**;
- **ANTLR4**: analisi completa in **~5 secondi** (15.766 referenze risolte);
- **Suite di test**: 16/16 test unitari e di integrazione passati con successo.

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
| 4  | executor.rs                 | Valutazione super-tipi nello scope | Risoluzione in parent_scope|
|    | (get_or_resolve_super_scopes| figlio e assenza di memoizzazione: | e memoizzazione con        |
|    |  / find_symbol_in_supers)   | albero O(b^d) con b=21, d=21.      | RefCell<HashMap>.          |
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

### 2.4 Problema 4: Esplosione Combinatoria nell'Ereditarietà Multipla (`KitchenSinkExtension.java`)

#### Sintomo
Durante l'analisi di **JUnit 5**, l'esecuzione si arrestava indefinitamente sul modulo 872:
`jupiter-tests/src/test/java/org/junit/jupiter/api/extension/KitchenSinkExtension.java`.
Il processo impegnava una CPU al 100% senza produrre output, simulando un loop infinito.

#### Causa Radice: L'Albero Combinatorio Esponenziale
`KitchenSinkExtension` è una classe di test che implementa contemporaneamente **21 interfacce Java**:
`BeforeAllCallback`, `BeforeEachCallback`, `BeforeTestExecutionCallback`, `TestWatcher`, `AfterTestExecutionCallback`, `AfterEachCallback`, `AfterAllCallback`, `ParameterResolver`, `ExecutionCondition`, ecc.

Nel codice precedente di `find_symbol_in_scope_and_supers_internal`:
```rust
for st in &ctx.tree.arena[scope_id].super_types {
    let resolved_st = match st {
        TypeRef::ResolutionQuery(q) => {
            evaluate_query(ctx, q, scope_id, true, visited).unwrap_or_else(|| st.clone())
        }
        ...
    };
    if let Some(super_scope) = find_scope_for_type(ctx.tree, &resolved_st) {
        if scope_id != super_scope {
            find_symbol_in_scope_and_supers_internal(ctx, super_scope, name, ...)
        }
    }
}
```
Due gravissimi errori architetturali cooperavano nel generare un'esplosione combinatoria:

1. **Risoluzione nello Scope Errato (`scope_id` anziché `parent_scope`)**:
   La query per risolvere `BeforeAllCallback` veniva valutata passando come punto di partenza `scope_id` (la classe `KitchenSinkExtension` stessa!).
   Poiché `evaluate_query_find` esegue una risalita lessicale (*lexical climbing*), per valutare `BeforeAllCallback` a partire da `KitchenSinkExtension`, invocava `find_symbol_in_scope_and_supers` sulla stessa `KitchenSinkExtension`!
2. **Re-iterazione e Reset dei Visitati**:
   Ad ogni chiamata annidata di `find_symbol_in_scope_and_supers`, veniva allocato un nuovo insieme `visited_scopes`.
   La ricerca del primo super-tipo portava la funzione a scandire di nuovo le 21 interfacce per verificare se il simbolo fosse contenuto nel secondo super-tipo; quest'ultimo a sua volta rieseguiva la scansione per il terzo, e così via.

**Analisi di Complessità**:
Con un fattore di ramificazione $b = 20$ e una profondità di ricorsione $d = 21$, il numero di chiamate ricorsive generate era di ordine fattoriale/esponenziale:
$$\mathcal{O}(b!) \approx 20! \approx 2.43 \times 10^{18} \text{ operazioni}$$
Anche eseguendo 1 miliardo di controlli al secondo, la risoluzione di quella singola classe avrebbe richiesto **oltre 70 anni** di calcolo.

```
PRIMA (Esplosione Combinatoria O(b!)):
KitchenSinkExtension (eval query 1 in KitchenSinkExtension)
 └── KitchenSinkExtension (eval query 2 in KitchenSinkExtension)
      └── KitchenSinkExtension (eval query 3 in KitchenSinkExtension)
           └── ... (profondità 21, ramificazione 20) -> BLOCCO COMPLETO (~10^18 chiamate)

DOPO (Risoluzione nel Parent Scope + Memoizzazione O(V + E)):
KitchenSinkExtension
 ├── super_scopes non in cache?
 │    ├── parent_scope (package org.junit...): valuta BeforeAllCallback    -> ScopeId(101)  [O(1)]
 │    ├── parent_scope (package org.junit...): valuta BeforeEachCallback   -> ScopeId(102)  [O(1)]
 │    └── ... (21 valutazioni nel parent_scope, mai discendendo nel figlio)
 └── Salva in ctx.resolved_super_scopes -> [101, 102, ..., 121]             [Cached]
 Qualsiasi lookup successivo: accesso O(1) all'array precalcolato!
```

#### Soluzione Implementata: Separazione degli Scope e Memoizzazione
1. **Regola Semantica dello Scope del Super-Tipo**:
   I nomi che compaiono nelle clausole `extends` e `implements` di una dichiarazione di classe appartengono lessicalmente allo **scope genitore** (`parent_scope`, ovvero il file o package contenitore), **mai** al corpo interno della classe stessa.
   Valutando le query con:
   ```rust
   let parent_scope = ctx.tree.arena[scope_id].parent.unwrap_or(ctx.tree.root);
   ```
   la risalita lessicale si muove unicamente verso l'esterno/radice, senza mai rientrare nella classe figlia.
2. **Memoizzazione degli Scope Ereditati (`resolved_super_scopes`)**:
   In `ExecutorContext` è stato introdotto un container con *interior mutability*:
   ```rust
   pub struct ExecutorContext<'a> {
       pub tree: &'a ScopeTree,
       pub primitives: &'a PrimitiveRegistry,
       pub config: &'a crate::config::AnalyzerConfig,
       pub resolved_super_scopes: RefCell<HashMap<ScopeId, Vec<ScopeId>>>,
   }
   ```
3. **Funzione Dedicata `get_or_resolve_super_scopes`**:
   - Se `scope_id` è già presente nella cache, restituisce immediatamente il `Vec<ScopeId>` in $O(1)$.
   - Per neutralizzare a monte eventuali grafi di ereditarietà ciclica, inserisce immediatamente una entry vuota nella cache:
     ```rust
     ctx.resolved_super_scopes.borrow_mut().insert(scope_id, Vec::new());
     ```
   - Risolve ciascun `super_type` (inclusi i generici con parametrizzazione) nel contesto di `parent_scope`.
   - Popola la cache definitiva con i `ScopeId` univoci trovati.
4. **Ispezione Lineare e Visita dei Grafi**:
   `find_symbol_in_scope_and_supers_internal` scorre i `ScopeId` già risolti senza più effettuare alcuna valutazione di query algebriche. Un set `visited_scopes` locale al lookup garantisce che l'esplorazione di strutture a diamante visiti ciascun'interfaccia antenata al più una volta sola ($O(V + E)$).

---

## 3. Guida Dettagliata per l'Allineamento della Tesi di Laurea

Questa sezione fornisce la mappatura puntuale delle modifiche rispetto ai capitoli della tesi (`master-thesis/chapters/`), indicando i paragrafi da aggiornare, le formalizzazioni matematiche da integrare e i nuovi risultati sperimentali.

---

### 3.1 Allineamento Capitolo 5 (`5_analysis_process.tex`) e Capitolo 6 (`6_extraction.tex`)

#### Sezione di Riferimento: Creazione dei Moduli e Mapping del File System
- **File Tesi**: `chapters/5_analysis_process.tex` (Sezione Pipeline Overview) e `chapters/6_extraction.tex` (Sezione Moduli e Packaging).
- **Aspetto da Aggiornare**:
  Nella descrizione del mapping topologico tra il file system reale e la gerarchia di `Module` (strategie `Directory` e `Package`), è necessario documentare esplicitamente l'invariante di **gestione dei file vuoti o vacui**:
  > *Nota per la Tesi*: In linguaggi modulari (in particolare Python con i file marker `__init__.py`, ma anche file di pure costanti o commenti in C/Java), la fase di estrazione sintattica può produrre una sequenza vuota di dichiarazioni ($\vec{\mathcal{M}} = \emptyset$). La pipeline non assume la presenza garantita di un modulo radice per ogni path analizzato. L'omissione di nodi vacui preserva la compattezza dello Scope Tree globale senza alterare le regole di visibilità degli import.

---

### 3.2 Allineamento Capitolo 7 (`7_name_resolution.tex`)

Il Capitolo 7 è il cuore teorico e implementativo interessato da queste modifiche. Di seguito le sezioni specifiche da integrare.

#### 1. Sezione 7.4.1: The Execution Context ($\Gamma$)
- **Stato Attuale nella Tesi**:
  Il testo attuale (righe 478-485) recita:
  $$\Gamma = \langle \mathcal{E}, \mathcal{P}, \mathcal{K} \rangle$$
  definendo $\Gamma$ come un contesto *strettamente immutabile*.
- **Aggiornamento Consigliato**:
  È opportuno arricchire la formalizzazione includendo lo stato di memoizzazione:
  $$\Gamma = \langle \mathcal{E}, \mathcal{P}, \mathcal{K}, \mathcal{M} \rangle$$
  dove $\mathcal{M} : \Sigma \to 2^{\Sigma}$ rappresenta la mappa di memoizzazione dei super-scope risolti.
  > *Spiegazione Teorica per la Tesi*:  
  > Sebbene $\mathcal{M}$ sia implementato tramite *interior mutability* (`RefCell<HashMap<ScopeId, Vec<ScopeId>>>`) per motivi di efficienza runtime in Rust, la funzione di risoluzione rimane **referenzialmente trasparente** e **idempotente**.  
  > Poiché la struttura dell'albero $\mathcal{E}$ e le definizioni dei simboli sono congelate al termine della fase $\rho_{\text{build}}$, il calcolo degli antenati di uno scope $\Sigma$ produce un insieme immutabile e deterministico di identificatori $\vec{\Sigma}_{\text{supers}}$. L'aggiornamento di $\mathcal{M}$ costituisce una pura ottimizzazione algoritmica (*lazy evaluation with memoization*) che non introduce effetti collaterali osservabili né dipende dall'ordine di visita delle compilation unit.

#### 2. Sezione 7.4.2: Lexical Scope Climbing e Chiavi di Visita
- **Stato Attuale nella Tesi**:
  Viene menzionato genericamente l'uso di un insieme `visited` per prevenire cicli.
- **Aggiornamento Consigliato**:
  Specificare la formalizzazione della chiave di disambiguazione della query.
  La valutazione di una query $q \in \mathcal{Q}$ a partire da uno scope $\Sigma$ definisce la transizione di visita:
  $$k_{\text{query}} = \langle \Sigma, \text{base}(q) \rangle \in \Sigma \times \mathcal{I}$$
  > *Motivazione Teorica*:  
  > L'indicizzazione unicamente basata sul nome del simbolo $\text{base}(q)$ collassava erroneamente la ricerca di membri omonimi in classi disgiunte. L'inclusione di $\Sigma$ garantisce la correttezza del principio di *lexical shadowing* e consente di identificare con precisione cicli di auto-assegnazione (es. $x = x[k]$) senza compromettere la risoluzione di variabili omonime in altri contesti.

#### 3. Sezione 7.4.4: Member Extraction and Inheritance Resolution
- **Stato Attuale nella Tesi**:
  Le righe 559-579 descrivono l'algoritmo di visita dei super-tipi.
- **Aggiornamento Consigliato**:
  Sostituire la formulazione precedente con la distinzione formale dello scope di risoluzione del super-tipo e l'algoritmo memoizzato a due stadi:
  1. **Regola di Scope Disjointness per Super-Tipi**:
     Sia $C$ una classe con scope $\Sigma_C$, e sia $\tau_{\text{super}} \in \text{supers}(C)$ una clausola di derivazione (`extends` o `implements`). La valutazione del tipo $\tau_{\text{super}}$ è definita sullo scope genitore:
     $$\text{resolve}(\tau_{\text{super}}, \text{parent}(\Sigma_C)) \quad \text{con } \text{parent}(\Sigma_C) \neq \Sigma_C$$
     Questo assioma impedisce che le dichiarazioni interne della classe figlia interferiscano o inneschino risalite lessicali spurie durante la determinazione della sua stessa classe base.
  2. **Complessità Algoritmica**:
     - *Senza memoizzazione e con scope errato*: la risoluzione nel corpo della classe genera un albero di ricorsione con fattore di ramificazione $b = |\text{supers}(C)|$ e profondità $d = |\text{supers}(C)|$, conducendo a una complessità combinatoria di $\mathcal{O}(b!)$ o $\mathcal{O}(b^d)$.
     - *Con memoizzazione e risoluzione in parent*: il calcolo dei super-scope per ciascun tipo strutturato richiede al più $\mathcal{O}(b)$ interrogazioni all'avvio. Una volta popolata la cache, la ricerca dei membri su gerarchie ad ereditarietà multipla si riduce a una visita in profondità (DFS) lineare sul grafo orientato aciclico (DAG) delle classi:
       $$\mathcal{O}(|V_{\text{supers}}| + |E_{\text{supers}}|)$$
     garantendo la scalabilità anche su pattern architetturali estremi (come l'aggregazione di 20+ interfacce in pattern Extension/Plugin).

#### 4. Sezione 7.4.5: Transitive Import Resolution
- **Stato Attuale nella Tesi**:
  Descrizione qualitativa dei re-export.
- **Aggiornamento Consigliato**:
  Aggiungere le condizioni di non-divergenza per gli import a stella (`*`):
  > *Precisazione per la Tesi*:  
  > Nel risolvere $\text{Extract}(M, \iota)$ attraverso re-export wildcard, il resolver mappa formalmente la richiesta nel cammino esteso $\pi = M \mathbin{\Vert} \langle \iota \rangle$, escludendo il token metalinguistico `*` dalla stringa di ricerca.  
  > La mutua ricorsione tra $\texttt{find\_global}(\pi)$ e $\texttt{resolve\_via\_transitive\_imports}(M, \iota)$ è rigorosamente troncata tramite due set di terminazione:
  > - $V_{\text{global}} \subseteq \mathcal{P}(\mathcal{I}^*)$: traccia le sequenze di identificatori globali già in corso di espansione;
  > - $V_{\text{trans}} \subseteq \Sigma \times \mathcal{I}$: traccia le coppie $\langle \text{modulo}, \text{membro} \rangle$ già interrogate nella catena di delega.

#### 5. Nuova Sottosezione Proposta: "Termination and Convergence Guarantees" (Sezione 7.6)
Si consiglia di inserire un breve paragrafo formale al termine del Capitolo 7 che riassuma le garanzie di convergenza del sistema:
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
  \item \textbf{Inheritance Traversal}: Super-scope resolution is memoized in $\mathcal{M}$. Circular derivation ($A \extends B \extends A$) is broken upon re-entry by initializing $\mathcal{M}(\Sigma) = \emptyset$ prior to recursive descent. Graph traversal over resolved super-scopes is protected by the scope set $V_{\text{scopes}} \subseteq \Sigma$, ensuring that each ancestor scope is visited at most once ($|V_{\text{scopes}}| \le N$).
  \item \textbf{Transitive and Alias Recursion}: Re-export chains and nested type assignments are indexed by composite keys $\langle \Sigma, \iota \rangle \in \Sigma \times \mathcal{I}$. Because the set of distinct symbols across the workspace is finite, the visited set $V_{\text{visited}}$ monotonically increases until either a terminal type reference is produced or the cycle is detected, yielding $\texttt{None}$ or $\texttt{TypeRef::Failed}$.
\end{enumerate}
Therefore, every execution branch either converges to a resolved reference or encounters a visited guard within finite steps, establishing termination.
\end{proof}
```

---

### 3.3 Allineamento Capitolo 9 (`9_evaluation_results.tex`)

I risultati ottenuti su ANTLR4, JUnit 5 e Fastjson rappresentano una validazione quantitativa cruciale per la tesi, dimostrando che OmniDeps è in grado di gestire repository di livello industriale senza degradazione delle prestazioni.

#### Tabella dei Risultati Empirici dei Benchmark Enterprise

Inserire nel Capitolo 9 la tabella di sintesi delle metriche estratte dai repository reali:

| Repository Target | Linguaggio Principale | Moduli Analizzati | Tipi Strutturati | Funzioni Libere | Riferimenti Risolti | Riferimenti Esterni / Non Risolti | Tempo di Esecuzione (Release) |
|---|---|---|---|---|---|---|---|
| **JUnit 5** | Java | 9.411 | 4.034 | 430 | 21.656 | 42.319 | **~8 s** |
| **Fastjson** | Java | 19.188 | 6.302 | 0 | 21.179 | 44.517 | **~15 s** |
| **ANTLR4** (completo) | Java / Python | 4.237 | 1.280 | 613 | 15.766 | 11.685 | **~5 s** |
| **ANTLR4 Runtime** | Java / Python | 2.053 | 709 | 601 | 2.549 | 10.348 | **~3 s** |
| **JUnit 4** | Java | 2.131 | 1.359 | 0 | 3.880 | 7.463 | **~2 s** |

#### Punti di Discussione da Enfatizzare nel Testo della Tesi:
1. **Throughput di Risoluzione**:
   Su JUnit 5 e Fastjson, l'analizzatore raggiunge un throughput compreso tra **1.000 e 1.300 moduli al secondo**, confermando che l'approccio *Two-Phase Name Resolution* (separazione tra sostituzione lessicale intra-procedurale ed esecuzione topologica globale) mantiene un overhead computazionale trascurabile rispetto al costo di parsing Tree-sitter.
2. **Eliminazione dei Falsi Blocchi**:
   Senza le ottimizzazioni di memoizzazione e separazione degli scope descritte, l'analisi di JUnit 5 risultava intrattabile ($> 10^{18}$ operazioni su `KitchenSinkExtension`). Con la linearizzazione introdotta, il tempo dedicato a quella specifica classe è passato da non misurabile ($\infty$) a **meno di 1 millisecondo**.
3. **Accuratezza e Classificazione delle Dipendenze Esterne**:
   Il rapporto tra riferimenti risolti e sconosciuti riflette fedelmente la natura di progetti di libreria che consumano intensamente le Java Standard Library (es. `java.util.*`, `java.lang.reflect.*`, non incluse nel workspace di analisi). Tali riferimenti vengono correttamente etichettati come `External`, prevenendo archi spuri nel grafo finale.

---

## 4. Riepilogo dei File Modificati nel Repository

A futura memoria e per facilitare la tracciabilità nei commit e nelle appendici della tesi:

- [`src/analyzer.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/analyzer.rs):
  - Aggiunto guard check per vettori vuoti in `apply_directory_strategy`.
- [`src/resolver/executor.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/src/resolver/executor.rs):
  - Aggiunto `resolved_super_scopes: RefCell<HashMap<ScopeId, Vec<ScopeId>>>` in `ExecutorContext`.
  - Implementata la funzione memoizzata `get_or_resolve_super_scopes` che valuta le derivazioni in `parent_scope`.
  - Refactoring di `find_symbol_in_scope_and_supers_internal` per scorrere i `ScopeId` precalcolati.
  - Implementata `evaluate_typeref_inner` con propagazione referenziale del set `visited`.
  - Normalizzato il trattamento dei wildcard `*` ed introdotte le guardie `global:{path}` e `trans:{scope_id}:{member}` in `find_global_internal` e `resolve_via_transitive_imports`.
- [`tests/generics_test.rs`](file:///Users/ferra/Developing/tesi-magistrale/omnideps/tests/generics_test.rs):
  - Aggiunti test di regressione dedicati:
    - `test_python_empty_file`
    - `test_python_self_referential_assignment`
    - `test_python_wildcard_import_cycle`
    - `test_java_multiple_interfaces_resolution`
