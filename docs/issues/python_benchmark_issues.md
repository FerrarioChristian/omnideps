# Risoluzione Problemi Benchmark Python

Questo documento raggruppa e analizza le cause dei fallimenti riscontrati storicamente nei test del benchmark per il linguaggio Python, e le relative soluzioni implementate. Attualmente il benchmark Python registra un successo del 100%.

## Riepilogo

| # | Sintomo | Gravità | Status e Spiegazione Soluzione |
| --- | ---------- | --------- | -------------------------------- |
| 1 | Parsing degli Import Fallito | 🟢 Risolto | Ispezione dei rami `aliased_import` e `dotted_name` per concatenare la destinazione intera. |
| 2 | Chiamate a Metodo non Risolte | 🟢 Risolto | Riconoscimento esplicito del pattern `assignment` per inferire il tipo base del ricevitore. |
| 3 | Campi Dinamici ignorati | 🟢 Risolto | Intercettazione assegnamenti con `self_keyword` per creare i campi strutturali dinamicamente. |
| 4 | Risoluzione di `super()` fallita | 🟢 Risolto | Intercettazione query `super()` nel Query Engine con risalita dello ScopeTree per recuperare la superclasse originaria. |

---

## 🟢 Problema 1: Parsing degli Import Fallito
**Sintomo:** L'istruzione `from models import User` falliva nel creare l'arco corretto verso `models.User`, arrestandosi alla lettura del solo prefisso `models`.
**Soluzione Implementata:** La funzione `try_parse_imports` è stata estesa. Ora ispeziona in profondità i rami `aliased_import`, `dotted_name` e `identifier` per concatenare la destinazione intera all'istruzione base, risolvendo correttamente il modulo target effettivo.

---

## 🟢 Problema 2: Inferenza di Tipo e Chiamate a Metodo non Risolte
**Sintomo:** Le chiamate a metodi di istanza (es. `admin.get_info()`) non venivano risolte verso la classe originale perché l'analizzatore non riusciva a inferire il tipo della variabile `admin` dall'espressione `Admin(...)`.
**Soluzione Implementata:** L'analizzatore riconosce esplicitamente i pattern `assignment` all'interno dei blocchi di codice. Il processo di deduzione di tipo verifica se la parte destra dell'assegnazione è un'invocazione di costruttore (o chiamata a costrutto OOP) inferendone il tipo base da assegnare all'identificatore di sinistra. Ciò permette la successiva risoluzione locale di chiamate come `admin.elevate_privileges()` alla vera classe sorgente.

---

## 🟢 Problema 3: Estrazione dei Campi Dinamici (Fields)
**Sintomo:** I campi (es. `models.User.username`) non venivano rilevati poiché in Python i campi non sono solitamente dichiarati staticamente, bensì istanziati dinamicamente nei metodi (`self.username = username`).
**Soluzione Implementata:** Attivando l'estrazione dinamica (`extract_dynamic_fields`), l'estrattore strutturale intercetta i nodi `assignment` il cui ricevitore è la keyword nativa di auto-riferimento (`self`). L'assegnazione comporta la creazione immediata di un membro `Field` direttamente nella struttura della classe.

---

## 🟢 Problema 4: Risoluzione Classe vs Costruttore e `super()`
**Sintomo:** L'istruzione `super().get_info()` generava un percorso letterale `["super()", "get_info"]` inrisolvibile, impedendo il tracciamento delle chiamate ai metodi ereditati.
**Soluzione Implementata:** Introdotta una regola di intercettazione in `evaluate_query_find` all'interno dell'esecutore logico: incontrando i termini testuali `"super()"` o `"super"`, l'engine sospende la ricerca ordinaria e risale lo `ScopeTree` fino allo scope della classe contenitore, estraendone i `super_types` (la classe madre). Tramite valutazione dinamica, traduce la reference in un collegamento diretto al genitore, risolvendo i metodi ereditati. (Nota: Il tracciamento dell'istanziazione di classe direttamente verso il metodo `__init__` è gestito separatamente tramite euristiche del call graph).
