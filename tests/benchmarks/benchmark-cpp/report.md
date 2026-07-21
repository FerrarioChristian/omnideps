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
| Transport.Vehicle | class | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Class |
| Transport.Car | class | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Class |
| Transport.Vehicle.brand | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| Transport.Vehicle.speed | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| Transport.Car.numDoors | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| Transport.Vehicle.accelerate | method | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| Transport.Vehicle.displayInfo | method | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| Transport.Vehicle.getBrand | method | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| Transport.Car.accelerate | method | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| Transport.Car.displayInfo | method | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| main | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |

## Edges
| Test Id | Source | Sink | Kind | Source Exists | Sink Exists | Edge Exists | Kind Is Correct |
| ------- | ------ | ---- | ---- | ------------- | ----------- | ----------- | --------------- |
| CPP-INH-1 | Transport.Car | Transport.Vehicle | inherits | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| CPP-CALL-1 | Transport.Car.displayInfo | Transport.Vehicle.displayInfo | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-CALL-2 | main | Transport.Car.accelerate | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-CALL-3 | main | Transport.Car.displayInfo | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-USE-1 | Transport.Car | Transport.Vehicle | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| CPP-USE-2 | main | Transport.Vehicle | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-USE-3 | main | Transport.Car | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-1 | Transport.Vehicle.displayInfo | Transport.Vehicle.brand | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-2 | Transport.Vehicle.displayInfo | Transport.Vehicle.speed | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-3 | Transport.Car.accelerate | Transport.Vehicle.speed | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| CPP-ACC-4 | Transport.Car.displayInfo | Transport.Car.numDoors | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |

## Results 
| Count | Total | Found | Not Found | Error Rate |
| ----- | ----- | ----- | --------- | ---------- |
| Nodes | 12 | 12 | 0 | 0.0000 |
| Edges | 11 | 2 | 9 | 0.8182 |
