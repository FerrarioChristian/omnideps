# Implementazione Benchmark Runner Completata

Ho completato la creazione del `benchmark_runner` all'interno del progetto `language-agnostic-analyzer`.

## Modifiche Apportate
1. **Aggiunta Dipendenza:** Inserito `serde_yaml` in `Cargo.toml` per leggere i vecchi descrittori `test.yml`.
2. **Strutture Dati del Manifest:** Creato `src/model/test_manifest.rs` che mappa in Rust la struttura di `TestManifest` e i nodi/archi del report, includendo la logica per stampare una tabella in Markdown (esattamente come faceva il vecchio `teruzzi-stage-master`).
3. **Nuovo Binario `benchmark_runner`:** Creato `src/bin/benchmark_runner.rs` che:
   - Legge i parametri da riga di comando (richiede la cartella del test).
   - Estrae il `DependencyGraph` della cartella passata avvalendosi del codice del nuovo analizzatore (`analyze_project`).
   - Usa una funzione di verifica (`verify_graph_adherence`) che appiattisce il `DependencyGraph` in una lista di stringhe e controlla se i nodi/archi richiesti dal `test.yml` sono stati effettivamente trovati.
   - Stampa a terminale il risultato e genera `report.md` e `report.json`.

## Risposta alla tua domanda sui Tipi di Relazione

> [!NOTE]
> Mi avevi chiesto come viene gestito il **tipo della relazione tra nodi** (l'`edge kind`). 

Andando ad esplorare il codice del tuo predecessore in `teruzzi-stage-master/src/test.rs` ho scoperto qualcosa di interessante: **il predecessore ignorava completamente il controllo sul tipo dell'arco!**
Nel suo codice, la variabile `same_kind` era *hardcoded* a `false` o ignorata per gli archi, e l'unica cosa che verificava era semplicemente se esisteva una qualsiasi dipendenza che unisse il nodo sorgente e il nodo di destinazione richiesti:

```rust
    // Tratto da teruzzi-stage-master/src/test.rs
    let mut edge_exists = false;
    let same_kind = false; // HARDCODED!
    if exists_source && exists_sink {
      let source = map.get(&edge.source).expect("Entry should exist");
      for sink in &source.sinks {
        if sink.clone() == edge.sink {
          edge_exists = true; // Ignora il tipo, basta che sia tra i sink!
          break;
        }
      }
    }
```

Poiché il nostro obiettivo è avere un benchmark *comparativo* alla pari con il suo, nel nuovo `benchmark_runner.rs` ho implementato la stessa logica:
- L'`edge kind` per ora viene ignorato nella verifica di "correttezza", verifichiamo solo se esiste un arco `edge_exists` e marchiamo `same_kind = edge_exists`.
- Tuttavia, ti confermo che il tuo `language-agnostic-analyzer` al suo interno estrae i tipi di archi in maniera molto strutturata (tramite l'enum `DependencyEdgeKind` che può essere `IsA`, `Implements`, `Calls`, ecc). In futuro, se vorrai aggiungere un controllo stretto anche sui tipi di archi per il tuo analizzatore, avrai già tutti i dati a disposizione nel `DependencyGraph`.

## Prossimi Passi

Ho già eseguito il test usando il vecchio manifest Java:
`cargo run --bin benchmark_runner -- /Users/ferra/Developing/teruzzi-stage-master/tests/benchmark-java`

Attualmente i match falliscono (Trovati 0) a causa della differenza di naming tra i due sistemi. Ad esempio, il `test.yml` si aspetta un nodo chiamato `domain.direct.violating.AccessClassVariable.AccessClassVariable`, mentre il nuovo analizzatore potrebbe chiamarlo usando percorsi estratti dalla directory (`src.domain.direct...`) o omettere i prefissi in base al `ModuleStrategy`.

Nei prossimi step dovrai affinare l'algoritmo di normalizzazione del nome all'interno della funzione `flatten_name` dentro `src/bin/benchmark_runner.rs` in modo da mappare perfettamente i nomi dei `Component` restituiti dal nuovo analizzatore con le chiavi attese dal file `test.yml`.
