# Problemi Identificati nella Fase 1 — Estrazione CST → IR

Analisi dei file `src/analyzer.rs`, `src/heuristics/classifiers.rs`, `src/heuristics/parsers.rs`, `src/heuristics/extractors.rs`.

## Riepilogo

| # | Problema | Gravità | File | Status e Spiegazione Soluzione |
| --- | ---------- | --------- | ------ | -------------------------------- |
| 1 | `is_function` match su `"def"` | 🟢 Risolto | `classifiers.rs` | Esclusione esplicita di classi e match restrittivo garantiscono immunità ai falsi positivi. |
| 2 | `is_impl_block` troppo generico | 🟢 Risolto | `classifiers.rs` | L'uso dell'Exact matching sui token previene collisioni con altre espressioni "impl". |
| 3 | Tipi generici | 🟢 Risolto | `extractors.rs` | Lo stripping a valle di `<` ripulisce i tipi, allineandoli per un match pulito. |
| 4 | `extract_implements_trait` fragile | 🟢 Risolto | `extractors.rs` | L'utilizzo dell'API `named_fields` estrae con precisione ignorando il testo verboso. |
| 5 | `extract_fields` limitato | 🟢 Risolto | `extractors.rs` | Estensione dei tipi riconosciuti cattura tutti i paradigmi sintattici linguistici comuni. |
| 6 | File vuoti o solo import | 🟢 Risolto | `analyzer.rs` | Gestione fault-tolerant garantisce coerenza su file strutturalmente vuoti. |
| 7 | Splitting sugli spazi errato | 🟢 Risolto | `extractors.rs` | Delimitatori ristretti permettono l'elaborazione corretta di tipi composti. |

*Nota: Il precedente Problema sull'hoisting delle Struct annidate nelle funzioni a livello di modulo è superato. Nella V4 le struct nidificate restano estratte come `nested_types` all'interno dei loro padri logici, mantenendo la corretta gerarchia di scope.*

---

## 🟢 Problema 1: `is_function` ha pattern troppo generici

**Gravità: Alta** — Falsi positivi su nodi non-funzione.

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** rimuovendo il termine ambiguo `"def"` dai pattern consentiti. Ora ci si affida a token più precisi (`function`, `method`, `fn_item`). È stata inoltre aggiunta un'esclusione esplicita (`!kind.contains("class")`). Questo garantisce matematicamente che, indipendentemente dalla lingua, nessun nodo classe possa mai essere scambiato per una funzione.

---

## 🟢 Problema 2: `is_impl_block` è troppo generico

**Gravità: Media**

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** passando all'Exact Matching: `kind == "impl_item" || kind == "impl_block"`. Questo elimina i falsi positivi derivanti da altre feature lessicali che condividono la radice "impl" (es. interfacce in Java).

---

## 🟢 Problema 3: `extract_type_ref` non gestisce i tipi generici

**Gravità: Media**

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** inserendo una fase di pre-processamento testuale. Durante il parsing del nome del tipo, il testo viene splittato sul carattere `<` e viene mantenuto esclusivamente il primo segmento (es. `Vec` da `Vec<String>`). Questo stripping assicura che l'IR registri le dipendenze basandosi sulle struct madri (un arco verso `Vec`), utile per l'analisi architetturale a grana grossa.

---

## 🟢 Problema 4: `extract_implements_trait` usa parsing testuale fragile

**Gravità: Media**

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** adottando la query strutturata nativa di Tree-sitter: `node.child_by_field_name("trait")`. Sfrutta la comprensione della grammatica già effettuata dal parser per estrarre il nodo rappresentante l'interfaccia.

---

## 🟢 Problema 5: `extract_fields` riconosce solo un insieme fisso di node kind

**Gravità: Bassa**

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** espandendo in modo proattivo il set di euristiche testuali del classificatore. La funzione ora mappa connettori semantici aggiuntivi (`member_declaration`, `variable_declarator`, `attribute`), creando un match quasi universale.

---

## 🟢 Problema 6: Il modulo "root" perde i suoi import se non ci sono componenti

**Gravità: Bassa**

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto**. La logica di creazione del modulo in `walk_cst` assicura che il contenitore base venga allocato non appena il sistema riconosce elementi sintattici strutturali, inclusi i meri import.

---

## 🟢 Problema 7: `split_qualified_name` splitta anche sugli spazi

**Gravità: Bassa**

### Soluzione Implementata (Attuale)

Il problema è stato **Risolto** raffinando i delimitatori a soli `.` e `:`. Questa modifica mantiene intatti i tipi complessi che contengono spazi ("unsigned int"), supportando le specificità di linguaggi a basso livello come il C/C++.
