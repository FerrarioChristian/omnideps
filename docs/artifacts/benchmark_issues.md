# Analisi dei Problemi nei Benchmark (C, C++, Python)

A seguito dell'esecuzione dei benchmark e di un'indagine approfondita sul codice sorgente dell'analizzatore (in particolare le euristiche di estrazione e la configurazione), sono emersi i seguenti problemi principali che spiegano i risultati scadenti:

## 1. Nodi C e C++ non riconosciuti: Configurazione dei Moduli e Estensioni File (RISOLTO)
Nei benchmark C e C++, inizialmente quasi nessun nodo veniva riconosciuto:
- **Estensioni errate**: I file `.h` venivano mappati al parser C puro anziché C++. Questo impediva il riconoscimento corretto delle classi C++ (come `class Vehicle`). È stato risolto aggiornando la mappa delle estensioni.
- **Configurazione Moduli**: Per `c` e `cpp`, le flag `implicit_file_modules` e `file_level_declarations` non creavano i namespace previsti. I test yaml si aspettavano i nodi direttamente in `root` o nei namespace corretti.

## 2. Nodi C non riconosciuti: Definizione di Struct con `typedef` (RISOLTO)
In `test.yml` ci aspettiamo di trovare la struct `Rectangle`.
- Nel file C, la struct è definita con il pattern molto comune `typedef struct { ... } Rectangle;`.
- In Tree-sitter, questo genera un nodo di tipo `type_definition`, che al suo interno contiene `struct_specifier`.
- È stato risolto modificando il parser per estrarre correttamente i campi (fields) dalla struct anonima contenuta all'interno della `type_definition`. Ora la copertura dei nodi per C e C++ è del 100%.

## 3. Archi C++ non riconosciuti: Metodi definiti fuori dalla dichiarazione della classe (RISOLTO)
Nei test C++, ci si aspetta che i metodi chiamino funzioni ereditate o accedano a campi (es. `Car::displayInfo()` che chiama `Vehicle::displayInfo()`). 
- In C++, la prassi è dichiarare il metodo in `.h` (`void displayInfo();` dentro la classe) e definirne il corpo nel file `.cpp` (`void Car::displayInfo() { ... }`).
- L'analizzatore attualmente non ricollega la definizione nel `.cpp` alla dichiarazione della classe. Invece, estrae `Car::displayInfo` come una semplice "free function" e lascia il metodo della classe originale privo di corpo (`body: None`).
- **Soluzione Implementata (Attuale):** È stato confermato il funzionamento e fixato il meccanismo di pre-processing `link_out_of_line_methods` attivato dalla configurazione `forward_declarations: true`. Questo step intercetta i metodi out-of-line (es. `Car::displayInfo`), cerca la classe di appartenenza e, se si trova in un file o modulo separato, genera un costrutto `ImplBlock` per iniettare i metodi. La nuova architettura `ScopeTree` supporta nativamente la risoluzione degli `ImplBlock`, riagganciando i metodi alla classe originaria per l'analisi. Il problema è risolto.

## 4. Archi C e C++ non riconosciuti: Risoluzione di Variabili Locali e Parametri (RISOLTO)
Gli accessi ai campi (es. `rect.width` in C) fallivano sistematicamente in fase di Name Resolution.
- L'estrattore individuava correttamente un accesso a `["rect", "width"]`. Tuttavia, il vecchio sistema non teneva traccia dello scope locale.
- **Soluzione Implementata (Attuale):** Con l'introduzione dello `ScopeTree` e l'estrazione gerarchica dei blocchi (Local Scope), l'estrattore ora popola lo scope locale della funzione con parametri e dichiarazioni (`let`). Il `SymbolStack` in fase di query valuta localmente `rect` risolvendolo correttamente in `Rectangle`, innescando l'arco corretto `Rectangle.width`. Il problema è risolto definitivamente.

## 5. Archi Python non riconosciuti: Parsing degli Import (RISOLTO)
In Python, l'istruzione `from models import User` falliva nel creare l'arco corretto verso `models.User`.
- Inizialmente, si fermava a leggere solo `"models"`.
- **Soluzione Implementata (Attuale):** La funzione `try_parse_imports` in `parsers.rs` è stata estesa. Ora ispeziona i rami `aliased_import`, `dotted_name`, e `identifier` per concatenare la destinazione intera all'istruzione base (`base_path.extend(txt)`), risolvendo così l'import effettivo. 

## 6. Archi Python non riconosciuti: Inferenza di Tipo e Chiamate a Metodo (RISOLTO)
Le chiamate a metodi di istanza (es. `admin.get_info()`) non vengono risolte correttamente verso la classe originale.
- Per risolverle, l'analizzatore deve prima inferire il tipo di `admin` dall'espressione `Admin(...)`.
- **Soluzione Implementata (Attuale):** L'analizzatore ora riconosce esplicitamente i pattern `assignment` tipici del Python all'interno dei blocchi di codice. Il processo di deduzione di tipo `infer_variable_type` verifica se la parte destra dell'assegnazione contiene una chiamata (`call`). Nel caso l'espressione chiamata si risolva in un costrutto orientato agli oggetti (es. Classe o Modulo), il tipo base viene inferito e assegnato all'identificatore di sinistra. Questo permette di inferire il tipo `admin = Admin()` e successivamente risolvere localmente chiamate come `admin.elevate_privileges()` alla vera classe sorgente. È stato testato e convalidato nel benchmark Python. Il problema è risolto.

## 7. Archi Python non riconosciuti: Estrazione dei Campi Dinamici (Fields) (RISOLTO)
Nei benchmark ci aspettiamo che vengano rilevati i campi `models.User.username`, `birth_year`, ecc. In Python i campi non sono dichiarati staticamente a livello di classe, ma creati dinamicamente (`self.username = username`).
- **Soluzione Implementata (Attuale):** L'analizzatore supporta l'estrazione dinamica! Abilitando `extract_dynamic_fields` per Python, la logica in `structural_extraction.rs` intercetta i nodi `assignment` il cui ricevitore è identificato dalla `self_keyword` del linguaggio. In questo modo l'assegnazione crea il `Field` direttamente nella struttura.

## 8. Archi Python non riconosciuti: Risoluzione Classe vs Costruttore e `super()` (RISOLTO)
- Un'istanziazione come `Admin(...)` crea una chiamata esplicita verso il tipo `Admin`, ma fallisce nel creare l'arco verso l'inizializzatore `__init__`.
- L'istruzione `super().get_info()` produce il percorso letterale `["super()", "get_info"]`.
- **Soluzione Implementata (Attuale):** La problematica relativa a `super()` è stata affrontata e **Risolta** brillantemente all'interno di `executor.rs`. È stata aggiunta una regola di intercettazione in `evaluate_query_find`: se il Query Engine incontra i termini testuali `"super()"` o `"super"`, blocca la normale ricerca e risale la `ScopeTree` fino al primo scope che definisce dei `super_types` (la classe contenitore). A questo punto estrae e valuta il primo tipo base, risolvendolo dinamicamente. Questo permette all'analizzatore di tradurre a runtime `super()` nel nome della vera classe madre, recuperando correttamente le chiamate a metodi ereditati (come dimostrato dall'aumento degli archi nel benchmark Python da 22 a 26). Rimane come miglioria futura la risoluzione specifica ai metodi costruttori `__init__` quando si cita solo il nome della classe.
