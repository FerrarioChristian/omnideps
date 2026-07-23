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

## 3. Archi C++ non riconosciuti: Metodi definiti fuori dalla dichiarazione della classe
Nei test C++, ci si aspetta che i metodi chiamino funzioni ereditate o accedano a campi (es. `Car::displayInfo()` che chiama `Vehicle::displayInfo()`). 
- In C++, la prassi è dichiarare il metodo in `.h` (`void displayInfo();` dentro la classe) e definirne il corpo nel file `.cpp` (`void Car::displayInfo() { ... }`).
- L'analizzatore non ricollega la definizione nel `.cpp` alla dichiarazione della classe. Invece, estrae `Car::displayInfo` come una semplice "free function" (funzione globale) e lascia il metodo della classe originale privo di corpo (`body: None`).
- Poiché il metodo della classe risulta vuoto, tutte le chiamate a funzioni e gli accessi a campi che vi avvengono all'interno vengono persi.

## 4. Archi C e C++ non riconosciuti: Risoluzione di Variabili Locali e Parametri (Local Scope)
Gli accessi ai campi (es. `rect.width` in C) falliscono sistematicamente in fase di Name Resolution.
- L'estrattore individua correttamente un accesso a `["rect", "width"]`.
- Tuttavia, il `GlobalRegistry` (che viene usato per risolvere i percorsi) indicizza soltanto Moduli, Classi, Funzioni e Campi globali/di classe. **Non tiene traccia delle variabili locali o dei parametri** delle funzioni.
- Di conseguenza, quando cerca di risolvere `rect` per capirne il tipo e trovare il suo campo `width`, la ricerca fallisce. Il sistema deduce che `rect` non esiste e non crea l'arco verso `Rectangle.width`. Questo problema affligge ogni linguaggio tipizzato (C, C++, Java) in cui si accede ai campi tramite istanze locali.

## 5. Archi Python non riconosciuti: Parsing degli Import
In Python, l'istruzione `from models import User` fallisce nel creare l'arco corretto verso `models.User`.
- In `parsers.rs` (funzione `try_parse_import`), per determinare il percorso importato viene letto il campo `module_name` (che vale `"models"`), e si ferma lì, ignorando del tutto la classe effettivamente importata (che si trova nel campo `name` o come lista di alias).
- L'analizzatore registra quindi un arco di dipendenza generico verso `models` e non verso `models.User`.

## 6. Archi Python non riconosciuti: Inferenza di Tipo e Chiamate a Metodo
Le chiamate a metodi di istanza (es. `admin.get_info()`) non vengono risolte correttamente verso la classe originale.
- Per risolverle, l'analizzatore deve prima capire di che tipo è la variabile `admin`. In Python, l'assegnamento avviene tramite un nodo `assignment` e la creazione dell'oggetto è un nodo `call` (es. `Admin(...)`).
- In `body_extraction.rs`, la funzione `infer_variable_type` cerca espressioni come `new_expression` o `object_creation_expression` (tipiche di Java/C++), ma non gestisce le chiamate a funzione semplici che in Python fungono da costruttori. Di conseguenza, il tipo di `admin` rimane sconosciuto (`Failed`).

## 7. Archi Python non riconosciuti: Estrazione dei Campi (Fields)
Nei benchmark ci aspettiamo che vengano rilevati i campi `models.User.username`, `birth_year`, ecc. (e di conseguenza gli accessi in lettura/scrittura `accesses_field`).
- In Python i campi non sono dichiarati staticamente a livello di classe, ma vengono creati dinamicamente tramite assegnamenti a variabili di istanza (es. `self.username = username`) all'interno di metodi come `__init__`.
- Attualmente la funzione `extract_fields` cerca specifici tipi di nodo AST (come `property_declaration`), ignorando i normali nodi `assignment` dinamici tipici di Python. 

## 8. Archi Python non riconosciuti: Risoluzione Classe vs Costruttore e `super()`
- Un'istanziazione come `Admin(...)` crea una chiamata esplicita verso il tipo `Admin`. Il sistema non deduce implicitamente che debba puntare al metodo interno `__init__`, facendo fallire i test che si aspettano un arco diretto a `__init__`.
- L'istruzione `super().get_info()` produce il percorso letterale `["super()", "get_info"]`. Il Resolver attualmente non possiede la logica per interpretare la parola chiave `super()` sostituendola con il super-tipo corretto della classe. L'arco rimane dunque irrisolto.
