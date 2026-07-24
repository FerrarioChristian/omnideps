# Dimostrazione Analitica: Overhead della Cache vs ScopeTree

Questo documento fornisce una dimostrazione algoritmica e computazionale del motivo per cui l'uso di una *Cache Tree* (Memoization) per la Name Resolution è stato rimosso in favore del *Lexical Scope Climbing* dinamico sull'architettura `ScopeTree` (V4).

## Premessa Architetturale
Nelle architetture primordiali (V1/V2), la ricerca di un identificatore richiedeva la navigazione fisica e ripetuta dell'albero sintattico (AST). Essendo un'operazione $\mathcal{O}(N)$ pesante, si era resa necessaria una **Cache** per memorizzare i risultati per ogni *Scope* lessicale. 

Con l'introduzione dello **ScopeTree gerarchico (V4)**, l'analizzatore costruisce preventivamente una foresta di nodi allocati in una `Arena`. Ogni nodo (Scope) contiene i propri simboli locali e un puntatore `parent` allo Scope genitore. Questo trasforma il costo di validazione di un identificatore da una complessa scansione ad albero a una risalita di puntatori in memoria contigua, combinata a lookup locali.

A fronte di questa ottimizzazione strutturale, interrogare e mantenere una Cache è matematicamente e fisicamente **più lento** del ricalcolo stesso. Ecco la dimostrazione.

---

## Analisi dei Costi (Big-O e Cicli CPU)

Definiamo le variabili per calcolare il costo della risoluzione di un identificatore $X$ (es. `Auto`) partendo da uno scope annidato (es. una variabile locale in `root::core::calcola`):

* **$d$**: Profondità di annidamento dello scope lessicale corrente (es. Blocco $\rightarrow$ Funzione $\rightarrow$ Modulo $\rightarrow$ Root, quindi $d \approx 4$).
* **$C_{lookup}$**: Costo temporale per saltare al parent e cercare in un vettore/hashmap locale (operazione rapidissima grazie alla CPU cache line data dall'Arena, $\approx 10$ cicli CPU).
* **$C_{alloc}$**: Costo temporale di una *Heap Allocation*, ovvero la richiesta di nuova memoria RAM per clonare un path risolto o espandere la mappa di Cache (operazione complessa legata all'OS, $\approx 200$ cicli CPU).

### 1. Costo dell'approccio Attuale (Lexical Climbing su ScopeTree)
Il *Lexical Scope Climbing* sfrutta i puntatori dell'Arena per risalire l'albero. Se $X$ non è nel blocco corrente, l'algoritmo passa in tempo costante $\mathcal{O}(1)$ allo scope genitore e ripete la ricerca, effettuando al massimo $d$ salti:
1. `Scope 12 (Blocco if)` $\rightarrow$ cerca `Auto` $\rightarrow$ fallisce ($1 \times C_{lookup}$)
2. `Scope 8 (Funzione calcola)` $\rightarrow$ cerca `Auto` $\rightarrow$ fallisce ($1 \times C_{lookup}$)
3. `Scope 3 (Modulo core)` $\rightarrow$ cerca `Auto` $\rightarrow$ trovato! ($1 \times C_{lookup}$)

* **Costo Temporale Peggiore**: $\approx d \times C_{lookup} \approx 4 \times 10 =$ **$40$ cicli CPU**.
* **Costo Spaziale (Overhead)**: **$0$ allocazioni Heap** extra per query (tutto avviene per reference mutabile nell'Arena).

### 2. Costo dell'approccio con Cache Tree
La cache dovrebbe associare l'ID del Contesto Attuale e la Variabile al suo Risultato Assoluto: `Cache(ScopeId, X) -> Result`.
Qual è il costo su un **Cache Miss** (quando l'elemento non è ancora in cache)?
1. Interrogare la cache per la chiave `(ScopeId, X)` $\rightarrow$ costo $C_{lookup}$.
2. Subire il *Miss* e risolvere l'elemento pagando il costo del Lexical Scope Climbing descritto sopra $\rightarrow$ costo $d \times C_{lookup}$.
3. **Salvare in Cache** il risultato per il futuro: la scrittura richiede di clonare il path/tipo risolto in un nuovo spazio di memoria RAM $\rightarrow$ costo $C_{alloc}$.

* **Costo Temporale su Miss**: $C_{lookup} + (d \times C_{lookup}) + C_{alloc} \approx 10 + 40 + 200 =$ **$250$ cicli CPU**.

---

## Il Paradosso dell'Hit-Rate

L'utilizzo di una Cache è vantaggioso unicamente se il costo ammortizzato (dato dall'alta percentuale di *Hit*) scende sotto il costo del ricalcolo. Ma nel Lexical Scoping su AST reali, l'Hit-Rate per scope locale **è intrinsecamente basso**.

Il motivo risiede nella chiave di caching stessa: l'`ScopeId` che identifica univocamente un blocco lessicale (ad es. un blocco `if` in una funzione).
Quando l'analizzatore attraversa sequenzialmente l'AST, cambia `ScopeId` continuamente. Se la classe `Auto` viene risolta dentro la `funzione_A`, l'hit viene registrato lì. Quando entra nella `funzione_B`, lo scope cambia e la cache per quel nuovo Scope è **fredda** (vuota), forzando un Miss garantito.
Si registra un Cache Hit *soltanto* se all'interno dello *stesso identico blocco lessicale* lo stesso identificatore viene interrogato multipli volte, il che accade di rado data la grana dell'analisi architetturale, che non ispeziona istruzione per istruzione.

## Conclusione

Grazie all'architettura `ScopeTree` ad allocazione su Arena, il costo della risoluzione è passato dall'esplorazione del parsing tree $\mathcal{O}(N)$ a una velocissima iterazione su vettori contigui in RAM $\mathcal{O}(d)$. 

A fronte di ciò, i circa $200$ cicli di processore sprecati per chiamare il memory allocator a ogni inevitabile *Cache Miss* superano di 6 volte i minuscoli $\approx 40$ cicli necessari per esplorare fisicamente a vuoto i parent dello `ScopeTree`.
L'eliminazione della Cache Tree non rappresenta quindi un impoverimento funzionale, bensì **un'ottimizzazione critica a livello di bare-metal**, che ha permesso di tagliare inutile entropia (allocation spaziale e tracking stateful) e massimizzare le performance sfruttando i rapidissimi salti sui puntatori delle strutture contigue in Rust.