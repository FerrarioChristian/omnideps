# Problemi Identificati nella Fase 1 — Estrazione CST → IR

Analisi dei file `src/analyzer.rs`, `src/heuristics/classifiers.rs`, `src/heuristics/parsers.rs`, `src/heuristics/extractors.rs`.

---

## 🔴 Problema 1: `is_function` ha pattern troppo generici — falsi positivi su nodi non-funzione

**Gravità: Alta** — Può causare classificazione errata di nodi CST.

### Descrizione
Il classificatore `is_function` (`classifiers.rs:38-48`) usa `contains("def")`:

```rust
pub fn is_function(node: Node) -> bool {
    // ...
    kind.contains("function") || kind.contains("method")
     || kind.contains("fn_item") || kind.contains("def") || kind.contains("func")
}
```

### Conseguenza
Diversi tipi di nodi CST contengono la sottostringa `"def"` senza essere funzioni:
- `"class_definition"` in Python → contiene `"def"` ✗ (è una classe)
- `"decorated_definition"` in Python → contiene `"def"` ✗ (è un decorator wrapper)
- `"default_parameter"` → contiene `"def"` ✗ (è un parametro)

In pratica, una `class_definition` Python verrebbe riconosciuta sia da `is_structured_type` (contiene `"class"`) sia da `is_function` (contiene `"def"`). Solo la priorità nel dispatcher (struct prima di function) evita il problema per le classi, ma `"decorated_definition"` potrebbe comunque generare falsi positivi.

### Soluzione suggerita
Usare un pattern più specifico, per esempio:

```rust
kind.contains("function") || kind.contains("method")
 || kind.contains("fn_item") || kind == "function_definition"
 || kind.starts_with("def") || kind.contains("func")
```

Oppure aggiungere esclusioni esplicite come in `is_structured_type`:
```rust
&& !kind.contains("class") && !kind.contains("decorated")
```

---

## 🟡 Problema 2: `is_impl_block` è troppo generico — potenziali falsi positivi

**Gravità: Media** — Mitigato dall'ordine del dispatcher.

### Descrizione
Il classificatore (`classifiers.rs:51-56`):

```rust
pub fn is_impl_block(node: Node) -> bool {
    if !node.is_named() { return false; }
    node.kind().contains("impl")
}
```

Qualsiasi nodo con `"impl"` nel kind verrà riconosciuto. Esempi di falsi positivi potenziali:
- `"impl_item"` → ✓ corretto
- `"implicit_parameter"` → ✗ (parametro implicito in Scala)
- `"implements_clause"` → ✗ (clausola in Java — ma Java non ha nodi con "impl" di solito)

### Mitigazione attuale
L'ordine del dispatcher mette `try_parse_impl_block` per ultimo, quindi import, moduli, struct e funzioni hanno la precedenza. Inoltre, questa è una feature principalmente per Rust, dove il kind è specificamente `"impl_item"`.

### Soluzione suggerita
Rendere il check più specifico:

```rust
pub fn is_impl_block(node: Node) -> bool {
    if !node.is_named() { return false; }
    let kind = node.kind();
    kind == "impl_item" || kind == "impl_block"
}
```

---

## 🟡 Problema 3: `extract_type_ref` non gestisce i tipi generici/parametrizzati

**Gravità: Media** — Perdita di informazione per tipi con generics.

### Descrizione
La funzione `extract_type_ref` (`extractors.rs:239-296`) estrae il testo di un nodo tipo e lo passa a `split_qualified_name`. Ma per un tipo come `Vec<String>` o `HashMap<String, i32>`:

```rust
// split_qualified_name("Vec<String>") 
// Risultato: ["Vec<String>"]  ← non viene splittato correttamente
```

Il nome qualificato include i parametri generici come parte del nome, il che potrebbe causare problemi nella name resolution (cercare `"Vec<String>"` invece di `"Vec"`).

### Conseguenza
I tipi parametrizzati potrebbero non essere risolti correttamente dal resolver, che cerca match esatti per nome.

### Soluzione suggerita
Pulire il testo dei tipi prima dello splitting, rimuovendo o separando i parametri generici:

```rust
fn clean_type_text(text: &str) -> &str {
    // Prendi solo fino al primo '<'
    text.split('<').next().unwrap_or(text).trim()
}
```

---

## 🟡 Problema 4: `extract_implements_trait` usa parsing testuale fragile

**Gravità: Media** — Parsing testuale soggetto a errori su codice complesso.

### Descrizione
La funzione (`extractors.rs:223-235`) usa `text.find("for ")` per trovare il trait in un `impl Trait for Type`:

```rust
pub fn extract_implements_trait(node: Node, source: &str) -> Option<TypeRef> {
    let text = node_text(node, source);
    if let Some(for_pos) = text.find("for ") {
        let before_for = text[..for_pos].trim();
        if before_for.contains("impl") && !before_for.ends_with("impl") {
            if let Some(name) = extract_name_from_text(before_for) {
                return Some(TypeRef::Unresolved(name));
            }
        }
    }
    None
}
```

### Problemi specifici
1. **`extract_name_from_text(before_for)`** dove `before_for` è ad esempio `"impl Display"` — il risultato include la keyword `"impl"` nel nome: `["impl", "Display"]` invece di `["Display"]`.
2. **`text.find("for ")`** corrisponde anche a usi di `for` nel corpo (es. `for` loop nel body dell'impl). Se il testo dell'intero nodo include il body, il primo `"for "` trovato potrebbe essere un for loop.
3. **Generics:** `impl<T> Display for Vec<T>` — il testo prima di `"for "` è `"impl<T> Display"`, che include parametri generici.

### Soluzione suggerita
Usare i named fields di Tree-sitter per i nodi `impl_item` di Rust, che hanno campi strutturati come `"trait"` e `"type"`:

```rust
if let Some(trait_node) = node.child_by_field_name("trait") {
    return Some(extract_type_ref(trait_node, source));
}
```

---

## 🟡 Problema 5: Le struct definite nel body delle funzioni vengono promosse a livello di modulo

**Gravità: Media** — Alterazione della gerarchia del codice nell'IR.

### Descrizione
In `walk_cst` (`analyzer.rs:48-56`), quando viene riconosciuta una funzione, il sistema esplora il body della funzione chiamando `walk_cst(child, source, modules)`:

```rust
crate::heuristics::ParsedItem::Component(crate::ir::Component::Function(ff)) => {
    modules[0].free_functions.push(ff);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind().contains("body") || child.kind().contains("block") {
            walk_cst(child, source, modules); // ← Usa 'modules' del genitore!
        }
    }
}
```

### Conseguenza
Se una struct è definita dentro il corpo di una funzione (possibile in linguaggi come Rust, Python, ecc.), essa viene inserita in `modules[0].structured_types` — cioè al **livello del modulo**, non dentro la funzione. Nell'IR risultante, la struct sembra essere una struct top-level del modulo, perdendo la relazione con la funzione contenitrice.

### Stato aggiornamento
**Parzialmente mitigato.** Con l'introduzione del tipo `Block` e della funzione `extract_block`, l'IR ora modella i blocchi lessicali interni alle funzioni (`body: Option<Block>` con `sub_blocks` ricorsivi). Tuttavia, `walk_cst` in `analyzer.rs` continua a promuovere le struct trovate nei body al livello del modulo. La nuova architettura `Block` traccia le *variabili locali* (`declarations`), le *chiamate* e le *istanziazioni* all'interno dei body, ma le definizioni di tipi strutturati continuano a essere promosse. Questo è accettabile come design choice dato che le struct locali alle funzioni sono comunque visibili a livello di modulo per la name resolution.

---

## 🟢 Problema 6: `extract_fields` riconosce solo un insieme fisso di node kind

**Gravità: Bassa** — Potrebbe non estrarre campi in linguaggi non coperti.

### Descrizione
La funzione `extract_fields` (`extractors.rs:118-131`) riconosce solo i nodi di tipo:

```rust
if matches!(child.kind(), "field_declaration" | "property_declaration" | "field") { ... }
```

### Conseguenza
Linguaggi che usano nomi diversi per le dichiarazioni di campo (es. `"variable_declaration"` in alcuni contesti, `"member_declaration"` in C#, `"attribute"` in Python) non verranno catturati.

### Mitigazione attuale
La funzione `extract_list_of` scende ricorsivamente nei nodi "body" e "list", quindi potrebbe trovare i campi a profondità maggiori. Tuttavia, se il kind del nodo campo non corrisponde, il campo verrà ignorato comunque.

### Soluzione suggerita
Estendere i kind riconosciuti o usare un approccio euristico:

```rust
if matches!(child.kind(),
    "field_declaration" | "property_declaration" | "field"
    | "member_declaration" | "variable_declarator"
) { ... }
```

---

## 🟢 Problema 7: Il modulo "root" perde i suoi import se non ci sono componenti nel file

**Gravità: Bassa** — Edge case specifico.

### Descrizione
In `walk_cst`, il modulo "root" viene creato solo quando `modules.is_empty()` **e** un componente viene riconosciuto dal dispatcher. Ma se un file contiene solo import e nessun altro componente, e gli import vengono riconosciuti come primo tipo di `ParsedItem`, il modulo "root" viene creato normalmente.

Tuttavia, se un file è completamente vuoto o contiene solo commenti, `generic_extract` restituisce un vettore vuoto `[]`. Questo non è necessariamente un bug, ma le fasi successive devono gestire il caso di un input senza moduli.

---

## 🟢 Problema 8: `split_qualified_name` splitta anche sugli spazi

**Gravità: Bassa** — Potrebbe generare segmenti inattesi nei nomi.

### Descrizione
La funzione (`extractors.rs:13-18`):

```rust
pub fn split_qualified_name(text: &str) -> QualifiedName {
    text.split(&[':', '.', ' '][..])
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}
```

Lo split su spazi è necessario per gestire casi come `"impl Trait"`, ma può causare problemi con testo come `"unsigned int"` (C/C++) che diventa `["unsigned", "int"]` invece di `["unsigned int"]`.

### Soluzione suggerita
Rimuovere lo spazio come separatore e gestire i casi speciali separatamente:

```rust
text.split(&[':', '.'][..])   // Solo : e .
```

---

## Riepilogo

| # | Problema | Gravità | File | Impatto | Status |
|---|----------|---------|------|---------|--------|
| 1 | `is_function` match su `"def"` troppo generico | 🔴 Alta | `classifiers.rs` | Falsi positivi su class_definition Python | **[Risolto]**: Sostituito con un controllo più specifico per ignorare le classi. |
| 2 | `is_impl_block` troppo generico | 🟡 Media | `classifiers.rs` | Potenziali falsi positivi | **[Risolto]**: Match esatto su `impl_item` o `impl_block`. |
| 3 | Tipi generici non gestiti in `extract_type_ref` | 🟡 Media | `extractors.rs` | Tipi parametrizzati non risolubili | **[Risolto]**: Introdotta pulizia con `<` in `split_qualified_name`. |
| 4 | `extract_implements_trait` parsing testuale fragile | 🟡 Media | `extractors.rs` | Include "impl" nel nome del trait | **[Risolto]**: Sostituito con accesso diretto ai named fields (`trait`) e pulizia testuale mirata. |
| 5 | Struct nel body promosse a livello modulo | 🟡 Media | `analyzer.rs` | Perdita di gerarchia | **[Parzialmente mitigato]**: Block modella i body ma walk_cst promuove ancora le struct. |
| 6 | `extract_fields` kind limitati | 🟢 Bassa | `extractors.rs` | Campi non estratti in alcuni linguaggi | **[Risolto]**: Estesa l'euristica ad `attribute`, `variable_declarator` ecc. |
| 7 | Modulo "root" e file vuoti | 🟢 Bassa | `analyzer.rs` | Edge case | **[Verificato]**: Il sistema gestisce nativamente input senza moduli senza crash. |
| 8 | `split_qualified_name` splitta su spazi | 🟢 Bassa | `extractors.rs` | Nomi con spazi generano segmenti inattesi | **[Risolto]**: Rimosso lo spazio dai separatori testuali. |

---

## ✅ Miglioramenti nel refactoring recente

### Nuovo tipo `Block` nell'IR
La struct `Function` non ha più `calls` e `instantiates` come campi diretti. Ora ha `body: Option<Block>`, dove `Block` è una struttura ricorsiva:

```rust
pub struct Block {
    pub declarations: Vec<Field>,    // Variabili locali (riuso di Field: name + type)
    pub calls: Vec<TypeRef>,          // Tipi i cui metodi vengono chiamati
    pub instantiates: Vec<TypeRef>,   // Tipi istanziati
    pub sub_blocks: Vec<Block>,       // Blocchi annidati (if, while, scope anonimi)
}
```

### Nuovo `extract_block` in extractors.rs
La vecchia funzione `traverse_for_body_deps` è stata sostituita da `extract_block`, che:
1. **Estrae le dichiarazioni locali** (`let`, `var`, ecc.) come `Field { name, ty }`
2. **Identifica i blocchi annidati** (`body`, `block`, `compound_statement`) e li modella ricorsivamente come `sub_blocks`
3. **Separa le dipendenze comportamentali** usando `find_behavioral_deps` che **salta i blocchi annidati** per evitare il double-counting

Questo miglioramento allinea l'estrazione al modello formale dove ogni coppia di `{}` genera uno scope lessicale distinto.
