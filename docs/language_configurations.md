# Analisi Critica delle Configurazioni Semantiche dei Linguaggi

Questo documento formalizza e analizza criticamente la configurazione semantica (`LanguageConfig`) adottata da **Omnideps** per i 5 linguaggi di programmazione supportati: **Java**, **Rust**, **Python**, **C++** e **C**.

Il modello si fonda sul paradigma delle **Language Product Lines**: invece di implementare pipeline di analisi separate o inserire diramazioni condizionali cablate (`if lang == "python"`), l'analizzatore scompone le caratteristiche dei linguaggi in una tupla ortogonale di feature semantiche indipendenti (`src/config.rs`). Ciascuna fase del motore (estrazione euristica, costruzione dell'albero di scope, risoluzione algebrica delle query e appiattimento del grafo) è guidata puramente da questi parametri.

---

## 1. Tabelle Sinottiche Generali

### Tabella 1.1: Strategie di Organizzazione dei Moduli (`ModuleConfig`)

| Linguaggio | `FileBased` | `DirBased` | `PkgDecl` | `Namespace` | `InlineMod` |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Java** | False | False | **True** | False | False |
| **Rust** | **True** | **True** | False | False | **True** |
| **Python** | **True** | **True** | False | False | False |
| **C++** | False | False | False | **True** | False |
| **C** | False | False | False | False | False |

---

### Tabella 1.2: Collegamento delle Dichiarazioni e Visibilità

| Linguaggio | `TransitiveImports` | `SupportImplBlocks` | `ForwardDeclarations` |
| :--- | :---: | :---: | :---: |
| **Java** | False | False | False |
| **Rust** | False | **True** | False |
| **Python** | **True** | False | False |
| **C++** | False | **True** | **True** |
| **C** | False | False | **True** |

---

### Tabella 1.3: Semantica delle Istanze e Risoluzione del Ricevitore

| Linguaggio | `SelfKeyword` | `ImplicitSelf` | `DynamicFields` | `SelfType` | `DerefTarget` |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Java** | `this` | False | False | *None* | *None* |
| **Rust** | `self` | False | False | `Self` | `Target` |
| **Python** | *None* | **True** | **True** | *None* | *None* |
| **C++** | `this` | False | False | *None* | *None* |
| **C** | *None* | False | False | *None* | *None* |

---

## 2. Analisi Critica Dettagliata per Linguaggio

### 2.1 Java

| Parametro | Valore | Giustificazione Semantica Reale e Architetturale |
| :--- | :---: | :--- |
| **`package_decl_based`** | **True** | In Java, l'appartenenza a un package è formalmente stabilita dalla direttiva sintattica `package com.foo.bar;` all'inizio dell'AST del file sorgente. |
| **`file_based`** | **False** | Un singolo file `.java` non introduce un namespace o un sottomodulo: più file nello stesso package condividono esattamente lo stesso ambito di visibilità package-private. |
| **`directory_based`** | **False** | Sebbene i file Java rispecchino per convenzione la gerarchia delle cartelle, l'autorità semantica per il Fully Qualified Name è la dichiarazione sintattica. L'estrazione basata su directory rischierebbe di includere prefissi spuri del filesystem del progetto (es. `src.main.java`). |
| **`namespace_based`** | **False** | Java non possiede costrutti `namespace` in stile C++. |
| **`inline_mod_based`** | **False** | Java non permette di dichiarare package o moduli inline nello stesso file. |
| **`transitive_imports`** | **False** | Gli import (`import java.util.List;`) hanno visibilità strettamente locale alla compilation unit; non riesportano simboli ai moduli o alle classi clienti. |
| **`support_impl_blocks`** | **False** | Java impone che tutti i metodi siano definiti rigidamente all'interno del blocco `class` o `interface`. Non esistono blocchi di implementazione disgiunti. |
| **`forward_declarations`**| **False** | Il compilatore Java risolve i tipi in modo globale a due passi; non esistono file `.h` separati né prototipi preventivi. |
| **`self_keyword`** | **`Some("this")`** | Parola chiave riservata del linguaggio utilizzata per accedere esplicitamente ai membri dell'istanza corrente (`this.field`, `this.method()`). |
| **`implicit_first_param_as_self`** | **False** | I metodi d'istanza Java non dichiarano il ricevitore tra i parametri formali dell'utente. |
| **`extract_dynamic_fields`** | **False** | Linguaggio rigidamente e staticamente tipizzato: tutti i campi devono essere dichiarati esplicitamente nel corpo della classe prima della compilazione. |
| **`self_type_keyword`** | **`None`** | Java non possiede una parola chiave nativa `Self` per riferirsi al tipo implementante corrente. |
| **`deref_coercion_target_name`** | **`None`** | Java non supporta il sovraccarico dell'operatore di dereferenziazione o la deref coercion implicita. |

---

### 2.2 Rust

| Parametro | Valore | Giustificazione Semantica Reale e Architetturale |
| :--- | :---: | :--- |
| **`file_based`** | **True** | In Rust, un file `foo.rs` corrisponde canonicamente al sottomodulo `foo` all'interno dell'albero del crate. |
| **`directory_based`** | **True** | Una cartella `foo/` contenente `mod.rs` (o affiancata da `foo.rs`) definisce un nodo intermedio nella gerarchia dei moduli. |
| **`package_decl_based`** | **False** | Rust non possiede direttive di package all'interno dei file sorgente; la configurazione del pacchetto è delegata a `Cargo.toml`. |
| **`namespace_based`** | **False** | Rust usa moduli (`mod`) e path separati da `::`, non blocchi `namespace` in stile C++. |
| **`inline_mod_based`** | **True** | Rust supporta esplicitamente moduli annidati inline all'interno dello stesso file sorgente (`mod tests { ... }`). |
| **`transitive_imports`** | **False** | Gli import `use foo::Bar;` sono privati per impostazione predefinita del modulo e non sono transitivi verso l'esterno a meno che non siano dichiarati esplicitamente con `pub use`. |
| **`support_impl_blocks`** | **True** | **Peculiarità fondamentale di Rust**: la dichiarazione dei dati (`struct`, `enum`) è rigidamente separata dal comportamento (`impl Point { ... }` o `impl Trait for Point { ... }`). I blocchi `impl` possono essere multipli e distribuiti su più file o sottomoduli. |
| **`forward_declarations`**| **False** | L'ordine delle dichiarazioni all'interno di un modulo non ha rilevanza per il compilatore `rustc`; non esistono header file. |
| **`self_keyword`** | **`Some("self")`** | Token minuscolo utilizzato come ricevitore d'istanza esplicito nei metodi (`&self`, `&mut self`, `self`). |
| **`self_type_keyword`** | **`Some("Self")`** | Token maiuscolo che identifica il tipo concreto all'interno di un blocco `impl` o `trait`. Omnideps lo mappa al tipo strutturato durante la registrazione nello scope tree. |
| **`implicit_first_param_as_self`** | **False** | In Rust il ricevitore `self` è dichiarato esplicitamente nella firma (`fn foo(&self)`). Non si deve trattare arbitrariamente il primo parametro come ricevitore (ad es. `fn new(x: i32)` ha `x` come parametro normale, non come istanza). |
| **`extract_dynamic_fields`** | **False** | Linguaggio statico con layout di memoria prefissato: tutti i campi devono essere dichiarati nella `struct`. |
| **`deref_coercion_target_name`** | **`Some("Target")`** | Modella la *Deref Coercion* di Rust: i tipi che implementano il trait `Deref<Target = T>` (smart pointer come `Box`, `Arc`, `Rc`, `RefCell`) delegano trasparentemente la chiamata dei metodi a `T`. Omnideps registra `Target` come supertipo nello scope tree per consentire la corretta risoluzione delle chiamate dereferenziate. |

---

### 2.3 Python

| Parametro | Valore | Giustificazione Semantica Reale e Architetturale |
| :--- | :---: | :--- |
| **`file_based`** | **True** | Ogni file `.py` costituisce un modulo indipendente all'interno dell'ecosistema Python. |
| **`directory_based`** | **True** | Ogni directory contenente un file `__init__.py` (o namespace package PEP 420) costituisce un package. |
| **`package_decl_based`** | **False** | Python non possiede direttive sintattiche per dichiarare package all'interno del sorgente. |
| **`namespace_based`** | **False** | Python non supporta blocchi `namespace` dichiarativi; i namespace coincidono con i moduli e gli oggetti. |
| **`inline_mod_based`** | **False** | Non è possibile definire moduli inline all'interno dello stesso file sorgente. |
| **`transitive_imports`** | **True** | **Meccanismo cruciale in Python**: all'interno di `package/__init__.py` è convenzione standard riesportare simboli da sottomoduli privati (`from .submodule import Service`), rendendoli consumabili dall'esterno come `package.Service`. Senza questo flag, la risoluzione delle dipendenze architetturali su package Python reali fallirebbe sistematicamente. |
| **`support_impl_blocks`** | **False** | Tutti i metodi di una classe sono definiti sintatticamente all'interno del corpo di `class`. |
| **`forward_declarations`**| **False** | Python è interpretato a caricamento dinamico; non adotta prototipi né file di intestazione. |
| **`self_keyword`** | **`None`** | **Scelta formale rigorosa**: in Python `self` **non è una parola chiave riservata del linguaggio**, ma una pura convenzione raccomandata (PEP 8). Metodi definiti come `def foo(this): this.x = 1` o `def bar(me): me.y = 2` sono codice Python perfettamente valido. |
| **`implicit_first_param_as_self`** | **True** | Si interfaccia con il punto precedente: poiché il runtime Python passa l'istanza al primo parametro posizionale di qualsiasi metodo d'istanza, Omnideps lega il primo parametro formale (qualunque sia il suo identificatore) all'istanza della classe, simulando fedelmente il modello a oggetti di Python. |
| **`extract_dynamic_fields`** | **True** | In Python le classi non dichiarano formalmente i propri campi d'istanza a livello di classe; gli attributi vengono istanziati dinamicamente durante l'esecuzione del costruttore (`self.x = val`). Omnideps analizza il corpo del costruttore `__init__` per estrarre questi attributi e promuoverli a membri `Field` del modello architetturale. |
| **`self_type_keyword`** | **`None`** | Python non ha un costrutto sintattico nativo a livello di linguaggio per riferirsi al tipo `Self` (pur essendo stato introdotto `typing.Self` in PEP 673 a fini di type checking statico facoltativo). |
| **`deref_coercion_target_name`** | **`None`** | Python non possiede deref coercion statica basata su tipi associati (utilizza metodi dunder dinamici come `__getattr__`). |

---

### 2.4 C++

| Parametro | Valore | Giustificazione Semantica Reale e Architetturale |
| :--- | :---: | :--- |
| **`namespace_based`** | **True** | In C++ la partizione logica dell'architettura è determinata dai blocchi `namespace foo { ... }`, i quali possono essere riaperti in più file e sono totalmente ortogonali rispetto alla struttura su disco. |
| **`file_based` / `DirBased`** | **False** | La suddivisione in cartelle o nomi file non incide sull'identificatore qualificato (FQN) delle classi o delle funzioni. |
| **`package_decl_based`** | **False** | C++ classico non possiede dichiarazioni di package (i moduli C++20 sono gestiti a livello di unità di esportazione ma l'analizzatore copre il modello canonico basato su namespace e header). |
| **`inline_mod_based`** | **False** | I namespace possono essere annidati, ma sono gestiti dalla strategia `namespace_based`. |
| **`transitive_imports`** | **False** | Le direttive `using namespace` o `using A::B` hanno effetto locale all'ambito di visibilità in cui sono dichiarate; non riesportano transitivamente l'intero namespace a moduli esterni. |
| **`support_impl_blocks`** | **True** | **Punto sottile ma essenziale**: sebbene C++ non possieda la keyword `impl`, adotta diffusamente la definizione di metodi *out-of-line* (`void MyClass::method() { ... }`) nei file `.cpp`, separata dalla dichiarazione della classe nei file `.hpp`. Omnideps modella questi metodi come un blocco di implementazione estesa ($\mathcal{X}$) per poi fonderli nella `StructuredType` $\mathcal{S}$ (`MyClass`) durante la fase di risoluzione. |
| **`forward_declarations`**| **True** | C++ richiede la dichiarazione prima dell'uso e adotta la classica separazione tra dichiarazioni nei file header (`.hpp`) e definizioni nei file sorgente (`.cpp`). Questo flag abilita il pairing tra dichiarazioni orfane e definizioni effettive. |
| **`self_keyword`** | **`Some("this")`** | Parola chiave riservata del linguaggio per il puntatore all'istanza corrente (`this->field`, `this->method()`). |
| **`implicit_first_param_as_self`** | **False** | Nei metodi C++ `this` non compare nella lista dei parametri formali dichiarati dal programmatore (a meno della sintassi avanzata `this Self&&` di C++23, non usata nei benchmark). |
| **`extract_dynamic_fields`** | **False** | Tutti i membri dato devono essere dichiarati nella definizione della classe o struct prima della compilazione. |
| **`self_type_keyword`** | **`None`** | C++ non possiede una keyword nativa `Self`. |
| **`deref_coercion_target_name`** | **`None`** | Il sovraccarico di `operator->` in C++ è gestito tramite puntatori intelligenti standard e non tramite tipi associati nominativi fissi. |

---

### 2.5 C

| Parametro | Valore | Giustificazione Semantica Reale e Architetturale |
| :--- | :---: | :--- |
| **Tutti i flag Modulo** | **False** | **Soluzione formale ineccepibile**: C non possiede moduli, né namespace, né package. Tutte le funzioni non statiche e le variabili globali condividono lo stesso identico spazio di nomi piatto al momento del linking. Impostando tutti i 5 flag a `False`, Omnideps colloca tutti i componenti C in un unico modulo radice globale, simulando fedelmente il comportamento del linker C. |
| **`transitive_imports`** | **False** | C non possiede un sistema di import modulare (usa solo inclusioni testuali `#include` del preprocessore). |
| **`support_impl_blocks`** | **False** | C è puramente procedurale: non esistono classi né metodi. |
| **`forward_declarations`**| **True** | In C i prototipi di funzione (`int compute(int);`) e le dichiarazioni incomplete di struct (`struct Node;`) nei file `.h` sono indispensabili per consentire l'uso di funzioni e tipi prima della loro definizione. |
| **Tutti i flag OOP/Instance**| **False / `None`** | C non possiede concetti di programmazione orientata agli oggetti: nessun ricevitore d'istanza, nessuna keyword `this`/`self`, nessuna allocazione dinamica di campi a livello di tipo. |

---

## 3. Punti di Integrazione nel Motore di Omnideps

La configurazione dichiarativa `LanguageConfig` interviene direttamente nei seguenti moduli del codice Rust:

1. **Estrazione Strutturale Euristica (`src/heuristics/structural_extraction.rs`)**:
   - `extract_dynamic_fields`: se attivo (Python), l'estrattore scende all'interno del corpo dei costruttori per catturare assegnazioni `self.x = val` e registrarle come campi del tipo strutturato.
   - `self_keyword` e `implicit_first_param_as_self`: usati per identificare il ricevitore dell'istanza all'interno delle funzioni membro.
   - `support_impl_blocks`: determina se estrarre definizioni di metodi out-of-line o blocchi `impl` come estensioni provvisorie `ImplBlock` ($\mathcal{X}$).

2. **Costruzione dello Scope Tree (`src/resolver/scope.rs`)**:
   - `self_type_keyword`: se presente (`Self` in Rust), registra il tipo strutturato corrente con tale identificatore nello scope della classe, abilitando la risoluzione di `Self::foo()` o tipi di ritorno `-> Self`.
   - `deref_coercion_target_name`: se presente (`Target` in Rust), durante la registrazione dell'implementazione del trait `Deref` inietta il tipo target tra i `super_types` della classe, estendendo la visibilità dell'ereditarietà virtuale.

3. **Risoluzione Algebrica delle Query (`src/resolver/executor.rs`)**:
   - `transitive_imports`: in `resolve_via_transitive_imports`, durante la risoluzione di percorsi qualificati `A.B`, se il modulo `A` ha questo flag attivo (Python), ispeziona anche i simboli importati all'interno di `A`.
   - `self_keyword`: in `resolve_self_keyword`, risolve gli accessi tramite `this` o `self` risalendo lo scope fino alla classe contenitrice.

4. **Accoppiamento Forward Declarations (`src/analyzer.rs`)**:
   - `forward_declarations`: se abilitato (C e C++), al termine dell'estrazione attiva la funzione di riconciliazione tra prototipi e definizioni concrete, prevenendo la generazione di vertici o archi duplicati nel grafo finale.
