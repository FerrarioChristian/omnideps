# Dimostrazione Analitica: Overhead della Cache vs Global Registry

Questo documento fornisce una dimostrazione algoritmica e computazionale del motivo per cui l'uso di una *Cache Tree* (Memoization) per la Name Resolution è stato rimosso nella versione V3 dell'architettura in favore del *Lexical Scope Climbing* su un `GlobalRegistry`.

## Premessa Architetturale
Nelle architetture precedenti (V1/V2), la ricerca di un identificatore richiedeva la navigazione fisica dell'albero sintattico. Essendo un'operazione $\mathcal{O}(N)$ pesante, si era resa necessaria una **Cache** per memorizzare i risultati per ogni *Scope* lessicale. 

Con l'introduzione del **Query Engine Bifase (V3)**, l'analizzatore costruisce preventivamente un `GlobalRegistry`: una `HashMap` piatta globale che associa i percorsi assoluti alle entità. Questo trasforma il costo di validazione di un path da una scansione ad albero a un lookup in tempo costante $\mathcal{O}(1)$.

A fronte di questa ottimizzazione, interrogare e mantenere la Cache è diventato **più lento** del ricalcolo stesso. Ecco la dimostrazione.

---

## Analisi dei Costi (Big-O e Cicli CPU)

Definiamo le variabili per calcolare il costo della risoluzione di un identificatore $X$ (es. `Auto`) partendo da uno scope annidato (es. `root::core::calcola`):

* **$d$**: Profondità di annidamento dello scope lessicale corrente (es. `root::core::calcola` $\rightarrow d = 3$). Nel codice reale, $d$ è tipicamente piccolo ($d \le 5$).
* **$C_{hash}$**: Costo temporale di un hashing e lookup in una `HashMap` (operazione estremamente rapida, $\approx 20$ cicli CPU).
* **$C_{alloc}$**: Costo temporale di una *Heap Allocation*, ovvero la richiesta di nuova memoria RAM per clonare un dato o espandere un nodo della Cache (operazione complessa legata all'OS, $\approx 200$ cicli CPU).

### 1. Costo dell'approccio Attuale (Senza Cache)
Il *Lexical Scope Climbing* vettoriale usa operazioni in-place (`.pop()`) mutando un singolo vettore, azzerando le allocazioni di memoria. L'algoritmo fa al massimo $d+1$ tentativi nella Tabella Hash pre-calcolata:
1. `["root", "core", "calcola", "Auto"]` $\rightarrow$ fallisce ($1 \times C_{hash}$)
2. `["root", "core", "Auto"]` $\rightarrow$ trovato! ($1 \times C_{hash}$)

* **Costo Temporale Peggiore**: $\approx d \times C_{hash} \approx 3 \times 20 =$ **$60$ cicli CPU**.
* **Costo Spaziale (Overhead)**: **$0$ allocazioni Heap** extra per query.

### 2. Costo dell'approccio con Cache Tree
La cache deve associare il Contesto Attuale e la Variabile al suo Risultato Assoluto: `Cache(ScopePath, X) -> Result`.
Qual è il costo su un **Cache Miss** (quando l'elemento non è ancora in cache)?
1. Interrogare la cache per la chiave `(ScopePath, X)` $\rightarrow$ costo $C_{hash}$.
2. Subire il *Miss* e risolvere l'elemento pagando il costo del Lexical Scope Climbing descritto sopra $\rightarrow$ costo $d \times C_{hash}$.
3. **Salvare in Cache** il risultato per il futuro: la scrittura richiede di clonare il path risolto in un nuovo spazio di memoria RAM $\rightarrow$ costo $C_{alloc}$.

* **Costo Temporale su Miss**: $C_{hash} + (d \times C_{hash}) + C_{alloc} \approx 20 + 60 + 200 =$ **$280$ cicli CPU**.

---

## Il Paradosso dell'Hit-Rate

L'utilizzo di una Cache è vantaggioso unicamente se il costo ammortizzato (dato dall'alta percentuale di *Hit*) scende sotto il costo del ricalcolo. Ma nel Lexical Scoping, l'Hit-Rate **è intrinsecamente vicino allo zero**.

Il motivo risiede nella chiave di caching stessa: lo `ScopePath`.
Se l'analizzatore risolve la classe `Auto` dentro la funzione `funzione_A`, il path `["root", "core", "Auto"]` viene salvato nella cache relativa allo scope `root::funzione_A`. 
Quando il parser si sposta nella `funzione_B` e incontra nuovamente `Auto`, lo ScopePath è cambiato. La cache del nuovo scope è **fredda** (vuota) e l'algoritmo subisce un altro Miss garantito.

Si registra un Cache Hit *soltanto* se la **stessa funzione** istanzia `Auto` più di una volta al suo interno.

## Conclusione

Grazie all'introduzione del `GlobalRegistry`, il costo della risoluzione è passato da $\mathcal{O}(N)$ a una banale somma di piccole costanti Hash $\mathcal{O}(1)$. 

A fronte di ciò, gli $\approx 200$ cicli di memoria sprecati per allocare RAM a ogni inevitabile *Cache Miss* superano nettamente gli $\approx 60$ cicli necessari per interrogare a vuoto il `GlobalRegistry` con il Lexical Scope Climbing.
L'eliminazione della Cache Tree non rappresenta quindi un impoverimento, bensì **un'ottimizzazione critica**, che ha permesso di eliminare inutile entropia nel codice e massimizzare le performance sfruttando i rapidissimi lookup matematici in RAM contigua offerti dalle HashMap di sistema.