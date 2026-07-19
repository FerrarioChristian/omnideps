# Report
## Files
- src/main.cpp
- src/Vehicle.h
- src/Vehicle.cpp
- src/Car.h
- src/Car.cpp

## Nodes
| Name | Kind | Node Exists | Kind Is Correct | Actual Kind |
| ---- | ---- | ----------- | --------------- | ----------- |
| Transport | namespace | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Module |
| Transport.Vehicle | class | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Car | class | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Vehicle.brand | field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Vehicle.speed | field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Car.numDoors | field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Vehicle.accelerate | method | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Vehicle.displayInfo | method | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Vehicle.getBrand | method | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Car.accelerate | method | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| Transport.Car.displayInfo | method | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| main.main | function | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |

## Edges
| Test Id | Source | Sink | Kind | Source Exists | Sink Exists | Edge Exists | Kind Is Correct |
| ------- | ------ | ---- | ---- | ------------- | ----------- | ----------- | --------------- |
| CPP-INC-1 | Vehicle.cpp | Vehicle.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-INC-2 | Car.h | Vehicle.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-INC-3 | Car.cpp | Car.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-INC-4 | main.cpp | Car.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-INH-1 | Transport.Car | Transport.Vehicle | inherits | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-CALL-1 | Transport.Car.displayInfo | Transport.Vehicle.displayInfo | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-CALL-2 | main.main | Transport.Car.accelerate | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-CALL-3 | main.main | Transport.Car.displayInfo | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-USE-1 | Transport.Car | Transport.Vehicle | uses_type | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-USE-2 | main.main | Transport.Vehicle | uses_type | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-USE-3 | main.main | Transport.Car | uses_type | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-1 | Transport.Vehicle.displayInfo | Transport.Vehicle.brand | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-2 | Transport.Vehicle.displayInfo | Transport.Vehicle.speed | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-3 | Transport.Car.accelerate | Transport.Vehicle.speed | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-4 | Transport.Car.displayInfo | Transport.Car.numDoors | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |

## Results 
| Count | Total | Found | Not Found | Error Rate |
| ----- | ----- | ----- | --------- | ---------- |
| Nodes | 12 | 1 | 11 | 0.9167 |
| Edges | 15 | 0 | 15 | 1.0000 |
