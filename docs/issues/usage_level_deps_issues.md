# Analisi delle Dipendenze da Accesso (Usage-Level Dependencies)

A seguito dell'introduzione del Query Engine V3 (basato su ScopeTree e risoluzione ad albero a due fasi descritto in `2_name_resolution_explained.html`), le precedenti limitazioni relative alle dipendenze di "Usage-Level" (Accesso a istanze, accessi statici ed enumeratori) risultano parzialmente superate.

Di seguito viene analizzata la gestione attuale di questi costrutti da parte del motore e illustrata la limitazione rimanente a livello di grafo esportato.

---

## 1. Accesso a Proprietà/Campi d'Istanza (Status Risoluzione: 🟩 Supportato)

**Stato Precedente:**
L'estrattore raccoglieva un'espressione come `user.name = "Mario"` estraendo la stringa `"user.name"`. Tuttavia, essendo `user` una variabile locale non mappata globalmente, il resolver non poteva inferire la classe proprietaria senza eseguire una complessa analisi di data-flow, disabilitandone il tracciamento.

**Stato Attuale (Query Engine V3):**
La fase di Sostituzione Lessicale e l'utilizzo degli scope gerarchici (`ScopeTree`) permettono il tracciamento delle variabili locali tramite il `SymbolStack`:
1. L'estrattore cattura un `Accesses` con token testuale `["user", "name"]`.
2. Il Builder controlla il token sinistro (`user`) nel `SymbolStack` locale. Se `user` è stato dichiarato nel blocco (es. `let user = User();`), il Builder sostituisce `["user", "name"]` con una query algebrica formale: `Query::Extract(Query::Find("user"), "name")`.
3. L'Executor esegue la risoluzione: prima valuta `Find("user")` (risalendo lo ScopeTree per trovare la dichiarazione locale ed estraendone il tipo `User`), dopodiché cerca lo scope della classe `User` e verifica l'esistenza del campo `"name"` al suo interno.
In questo modo la dipendenza verso `User.name` viene risolta accuratamente e staticamente.

---

## 2. Accesso Statico / Variabili Globali (Status Risoluzione: 🟩 Supportato)

**Stato Precedente:**
Costrutti come `Math.PI` venivano intercettati ed esportati in modo generico senza convalidare che `Math` fosse effettivamente una classe nota e `PI` un suo campo statico.

**Stato Attuale (Query Engine V3):**
Il Builder trasforma l'accesso in `Query::Extract(Query::Find("Math"), "PI")`. Poiché `Math` non è una variabile locale, `Find("Math")` innesca una ricerca lessicale (`lexical climbing`) sull'albero degli scope, risalendo progressivamente fino ai moduli e alle direttive di import. Identificata la classe/modulo esatta, ne estrae dal relativo scope il campo `PI`. La validazione a 2-step è interamente gestita dal motore delle query.

---

## 3. Enum Usage (Status Risoluzione: 🟩 Supportato)

**Stato Precedente:**
L'uso delle varianti enum, come `Status.ACTIVE`, subiva le stesse carenze dell'accesso statico.

**Stato Attuale (Query Engine V3):**
All'interno di Antigravity, moduli, classi ed enums sono rappresentati uniformemente come `ScopeNode` (con campi/varianti salvati come `Symbol::Value`). La risoluzione di `Extract(Find("Status"), "ACTIVE")` avviene specularmente a quella dell'accesso statico, garantendo una tracciabilità completa.

---

## 🟡 Limitazione Rimanente: Tipizzazione degli Archi nel Grafo Finale

Sebbene il Motore di Name Resolution v3 sia in grado di identificare staticamente e correttamente le destinazioni logiche di questi costrutti, l'analizzatore presenta una perdita di precisione durante la traduzione del risultato in archi del Dependency Graph.

Attualmente, all'interno del modulo `src/export/graph.rs`, il builder itera l'array `accesses` e, indipendentemente dalla natura del destinatario finale (Campo d'Istanza, Campo Statico o Variante Enum), **emette indiscriminatamente un arco generico di tipo `DependencyEdgeKind::AccessesField`**.

### Soluzione Architetturale Proposta
Il completamento di questa feature richiede un aggiornamento nell'esportatore del Grafo:
1. In `src/export/graph.rs`, durante la generazione degli archi per `accesses`, si dovrebbe ispezionare l'Intermediate Representation (IR) originaria della destinazione interrogando lo `ScopeTree` (es. per verificare se la destinazione appartiene a una classe, un enum o rappresenta un'istanza).
2. L'arco emesso verrebbe così specializzato e arricchito (es. diventando `AccessesStatic`, `UsesEnumVariant` o mantenendo `AccessesField`), incrementando significativamente la granularità architetturale del grafo esportato.
