# Domande sull'Architettura del Language-Agnostic Analyzer

Di seguito le risposte strutturate alle domande sul funzionamento interno e le scelte architetturali dell'analizzatore.

---

### 1. La costruzione del grafo avviene solo dopo il completamento della name resolution o parallelamente passo passo? Se fosse il primo caso, dove viene salvato il risultato intermedio della risoluzione?

La costruzione del grafo avviene **rigorosamente dopo** il completamento totale della Name Resolution sull'intero progetto. L'architettura segue una pipeline strettamente sequenziale, formalizzata dall'equazione $\Phi = G \circ R_{exec} \circ R_{build} \circ \text{Agg} \circ E$. Non c'è parallelismo logico tra risoluzione e costruzione del grafo.

Il risultato intermedio della risoluzione non viene salvato in una struttura dati esterna o in un database separato, ma **muta direttamente l'IR (Intermediate Representation) in memoria**.
La funzione `resolve_type_refs` prende letteralmente possesso dell'intero albero del progetto (`Vec<Module>`), attraversa tutti i componenti e **sovrascrive** il campo `TypeRef` (ovvero $\tau$) di ogni parametro, campo o valore di ritorno.
Alla fine della risoluzione, la funzione restituisce lo stesso `Vec<Module>`, ma ora ogni `TypeRef` al suo interno è stato promosso dallo stato `Unresolved` allo stato definitivo (`Resolved`, `External`, `Primitive` o `Failed`). È questo IR "purificato" e annotato che viene poi passato in blocco alla fase 3 ($G$) per la costruzione del grafo.

---

### 2. Qual è il risultato della Name Resolution? Cosa collega l’identificatore iniziale al tipo vero e proprio?

Il risultato matematico e logico della Name Resolution è la **trasformazione di un nome locale ambiguo in un percorso assoluto e univoco** ($\mathcal{QN}$).

Ciò che collega l'identificatore iniziale ("come l'ha scritto il programmatore") al tipo vero e proprio ("l'entità strutturale nel progetto") è la **Macchina a Stati dell'enum `TypeRef`**, guidata dal Query Engine:

1. **Il Punto di Partenza:** L'estrattore legge `base: Veicolo` e crea `TypeRef::Unresolved(["Veicolo"])`.
2. **La Traslazione dell'Intento (Fase 2a):** Il Builder, usando lo Scope effimero (il `SymbolStack` o $\rho$), converte il nome in una formula matematica che descrive "come trovarlo". Il tipo diventa `TypeRef::ResolutionQuery(Find("Veicolo"))`.
3. **Il Collegamento Reale (Fase 2b):** L'Executor naviga il `GlobalRegistry` ($\mathcal{GR}$, l'indice di tutti i percorsi assoluti del progetto). Valutando l'equazione `Find("Veicolo")`, applica l'algoritmo di *lexical scope climbing* (risalita degli scope). Quando trova una corrispondenza esatta nell'indice, il collegamento si concretizza: il tipo viene sovrascritto definitivamente in `TypeRef::Resolved(["root", "core", "Veicolo"])`.

Il collegamento è quindi il percorso stringa assoluto: `["root", "core", "Veicolo"]` è l'ID univoco che unisce il chiamante alla definizione del chiamato.

---

### 3. Perché si dice che “Le variabili locali, sebbene fondamentali per il tracciamento del flusso dei dati, non costituiscono componenti architetturali”?

Questa affermazione è il pilastro che distingue un analizzatore *architetturale* da un analizzatore di *flusso di controllo* (come un debugger o un profiler).

L'architettura del software si occupa delle **API pubbliche**, dell'accoppiamento strutturale tra moduli e delle dipendenze di alto livello tra le classi.
Le variabili locali (es. `let x = new Database();`) sono **dettagli implementativi privati** di una specifica funzione. Nascono e muoiono durante l'esecuzione del metodo e sono invisibili dal resto del sistema. Nel grafo delle dipendenze $\mathcal{G}$, non avrebbe senso avere un nodo chiamato `x`.

Tuttavia, sono "fondamentali per il tracciamento del flusso dei dati" perché se la funzione fa `x.query()`, l'analizzatore deve sapere che `x` è di tipo `Database` per poter dedurre che la funzione corrente sta chimando il metodo `query` del `Database`.
Quindi, l'analizzatore estrae e "usa" temporaneamente le variabili locali nella Fase 2a per decodificare chi sta chiamando chi, ma **non le esporta come nodi** nel grafo finale. L'arco generato nel grafo sarà direttamente: `Funzione -> Calls -> Database`, saltando l'intermediario effimero `x`.

---

### 4. Nella formalizzazione c’è scritto che la fase di risoluzione delle dipendenze viene fatta a partire dall'IR, ed è completamente indipendente dal linguaggio sorgente. È vero?

**Assolutamente sì, ed è la più grande conquista tecnica dell'intero progetto.**

Se guardi il codice sorgente della cartella `src/resolver/` (Fasi 2a e 2b), scoprirai che non c'è una singola istruzione `if language == Java` o `match lang { Rust => ... }`. Il resolver è completamente cieco riguardo al linguaggio originale.

Questo è possibile perché la Fase 1 ($E$, Extraction) assorbe tutte le complessità sintattiche e grammaticali dei singoli linguaggi (le differenze tra classi Java, struct C, package, namespace, impl blocks) e le "schiaccia" forzatamente nel modello standardizzato IR ($\mathcal{M}$, $\mathcal{S}$, $\mathcal{F}$).

Quando il flusso arriva al Resolver, quest'ultimo opera basandosi unicamente sulle regole universali della semantica lessicale:

- **Scope gerarchici** (tramite il `SymbolStack`).
- **Percorsi assoluti** (tramite il `GlobalRegistry`).
- **Algebra relazionale** (tramite `Find`, `Extract`, `Call`).

La logica di risoluzione (ad esempio il *Method Resolution Order* Depth-First per cercare i metodi nelle classi genitore) è applicata in modo matematico sulle strutture IR, permettendo all'algoritmo di inferire i tipi e risolvere le dipendenze in C++ esattamente con lo stesso identico codice con cui le risolve in Python.

---

Il nostro analizzatore è in grado di tracciare le dipendenze tra file in linguaggi diversi?
11:32

La risposta breve è: in teoria sì, in pratica no (non ancora).

Ecco perché:

Cosa c'è già ✅
Guardando
analyze_directory
 (riga 92–146), il sistema già:

Scansiona ricorsivamente una cartella con WalkDir
Rileva il linguaggio di ogni file indipendentemente (riga 105: SupportedLanguage::from_path)
Estrae i moduli IR da ciascun file con parse_source — ogni file può essere di un linguaggio diverso
Accumula tutti i moduli in un unico Vec<Module> (riga 110: all_modules.append)
Unifica i registri di primitivi (riga 111: combined_primitives.merge)
Passa tutto insieme ad analyze_project → risoluzione unificata + grafo unico
Quindi l'infrastruttura per mescolare linguaggi in un unico grafo è presente.

Cosa manca ❌
Il problema è che gli import tra linguaggi diversi non esistono a livello sintattico. Il sistema risolve le dipendenze cross-file tramite gli import (use, import, #include) che sono costrutti intra-linguaggio. Non esiste un import Python che importi un modulo Rust, o un use Rust che importi una classe Java.

In pratica, i moduli di linguaggi diversi finirebbero nello stesso GlobalRegistry, ma nessun import li collegherebbe — quindi il resolver non troverebbe mai un percorso per attraversare i confini linguistici.

Per fare un esempio concreto
Se avessi una cartella con server.java e client.py, il sistema produrrebbe:

Nodi per le classi Java e le classi Python ✅
Archi interni Java→Java e Python→Python ✅
Archi Java→Python o Python→Java ❌ (nessun import li collega)
Per supportare dipendenze cross-language servirebbe un meccanismo aggiuntivo — ad esempio, un file di configurazione che dichiari le interfacce condivise (gRPC, REST, FFI), o un'euristica che cerchi naming conventions comuni. Questo potrebbe essere un punto interessante da menzionare negli sviluppi futuri, se vuoi.

