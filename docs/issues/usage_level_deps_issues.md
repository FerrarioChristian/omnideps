# Analisi delle Dipendenze da Accesso (Usage-Level Dependencies) alla luce del Query Engine V3

Dopo una revisione approfondita dell'architettura attuale (il Query Engine bifase con **ScopeTree** e risoluzione ad albero descritto in `2_name_resolution_explained.html`), è emerso che le limitazioni descritte in precedenza per le dipendenze di "Usage-Level" (Accesso a istanze, accessi statici ed enumeratori) sono in realtà **già state risolte nativamente dal nuovo motore v3**. 

Di seguito viene illustrato come il nuovo motore gestisce questi costrutti superando i vecchi limiti (rendendoli tutti con stato 🟩 **Sì**), e quali sono le uniche reali mancanze a livello di grafo finale.

---

## 1. Accesso a Proprietà/Campi d'Istanza (Ora: 🟩 Sì)

**Il Problema Storico**
Nelle vecchie versioni dell'analizzatore (con Global Registry piatto e risoluzione monofase), l'estrattore raccoglieva un'espressione come `user.name = "Mario"` estraendo la stringa `"user.name"`. Tuttavia, `user` essendo una variabile locale non era mappata, per cui il resolver non poteva inferire la classe proprietaria senza eseguire un'analisi complessa del data-flow. Di conseguenza, il tracciamento era disabilitato.

**Come lo risolve il Query Engine V3**
Grazie alla fase di Sostituzione Lessicale (`builder.rs`) e all'utilizzo degli scope gerarchici (`ScopeTree`), le variabili locali sono ora perfettamente tracciate nell'Environment (`SymbolStack`).
1. L'estrattore cattura un `Accesses` con token testuale `["user", "name"]`.
2. Nella **Fase 2a**, il Builder controlla il token sinistro (`user`) nel `SymbolStack` locale. Se `user` è stato dichiarato nel blocco (es. `let user = User();`), il Builder sostituisce `["user", "name"]` con una query algebrica formale: `Query::Extract(Query::Find("user"), "name")`.
3. Nella **Fase 2b**, la funzione `evaluate_query_extract` dell'Executor esegue una risoluzione esatta a due passaggi: prima valuta `Find("user")` (che naviga lo ScopeTree dal blocco corrente verso l'alto e trova la dichiarazione locale di `user`, estraendone il tipo `User`), dopodiché cerca lo scope della classe `User` e verifica l'esistenza del campo `"name"` al suo interno.
In questo modo la dipendenza verso `User.name` viene risolta accuratamente, in modo 100% statico.

---

## 2. Accesso Statico / Variabili Globali (Ora: 🟩 Sì)

**Il Problema Storico**
Costrutti come `Math.PI` venivano anch'essi intercettati, ma esportati in modo generico senza convalidare che `Math` fosse effettivamente una classe nota e `PI` un suo campo statico, limitandosi ad un'esportazione "pigra".

**Come lo risolve il Query Engine V3**
Il comportamento è identico a quanto descritto sopra. Il Builder trasforma l'accesso in `Query::Extract(Query::Find("Math"), "PI")`.
Poiché `Math` non è una variabile locale nel `SymbolStack`, `Find("Math")` innescherà una ricerca lessicale (`lexical climbing`) sull'albero degli scope. Salirà dal blocco corrente, alla funzione, alla classe, al modulo, fino ad arrivare alle direttive di import (tramite le quali potrà determinare se `Math` viene da un altro file). Una volta trovato il modulo o la classe `Math` esatta, estrarrà dal suo scope il campo `PI`. Anche in questo caso, la validazione a 2-step è perfettamente implementata dal motore delle query.

---

## 3. Enum Usage (Ora: 🟩 Sì)

**Il Problema Storico**
L'uso delle varianti enum, come `Status.ACTIVE`, subiva le stesse carenze dell'accesso statico.

**Come lo risolve il Query Engine V3**
Poiché per il motore di Name Resolution di Antigravity moduli, classi ed enums sono tutti rappresentati come ScopeNodes (con i relativi campi/varianti salvati come `Symbol::Value`), la risoluzione `Extract(Find("Status"), "ACTIVE")` avviene in maniera analoga all'accesso statico, garantendo una completa tracciabilità.

---

## L'Unica Vera Limitazione Rimanente (Edge Type nel Grafo)

Sebbene il Motore di Name Resolution v3 sia oggi in grado di identificare esplicitamente, staticamente e correttamente le destinazioni di questi costrutti, l'analizzatore soffre di un'imprecisione nel momento in cui **traduce il risultato della risoluzione in archi del Dependency Graph**.

Attualmente in `graph_builder.rs`, il builder cicla l'array `accesses` del blocco e, a prescindere dal fatto che il destinatario finale risolto sia un Campo d'Istanza, un Campo Statico, o una Variante Enum, **emette per tutti indiscriminatamente un arco generico di tipo `DependencyEdgeKind::AccessesField`**.

### Soluzione Architetturale
Il vero lavoro rimasto per concludere questa feature non è nel motore di risoluzione (che fa già il lavoro duro), bensì nell'esportatore del Grafo:
1. In `graph_builder.rs`, quando si generano gli archi per `accesses`, si dovrebbe analizzare l'IR originale della destinazione (accedendo allo ScopeTree per sapere se la destinazione è dentro una classe, un enum o se è un'istanza locale).
2. Sulla base di questo, l'arco generato dovrebbe essere arricchito diventando `AccessesStatic`, `UsesEnumVariant` o rimanere `AccessesField` a seconda della natura del nodo target, migliorando esponenzialmente la granularità e l'espressività dell'analisi architetturale esportata. 

*(Questa nota è stata inserita anche come aggiornamento direttamente all'interno della roadmap `0_features_roadmap.html`)*
