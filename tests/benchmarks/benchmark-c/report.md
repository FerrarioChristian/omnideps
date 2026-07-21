# Report
## Files
- src/main.c
- src/math_utils.h
- src/math_utils.c
- src/shapes.h
- src/shapes.c

## Nodes
| Name | Kind | Node Exists | Kind Is Correct | Actual Kind |
| ---- | ---- | ----------- | --------------- | ----------- |
| add | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| multiply | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| Rectangle | struct | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Struct |
| Rectangle.width | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| Rectangle.height | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| calculate_area | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| main | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |

## Edges
| Test Id | Source | Sink | Kind | Source Exists | Sink Exists | Edge Exists | Kind Is Correct |
| ------- | ------ | ---- | ---- | ------------- | ----------- | ----------- | --------------- |
| C-CALL-1 | multiply | add | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-CALL-2 | calculate_area | multiply | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-3 | main | calculate_area | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-4 | main | add | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-USE-1 | calculate_area | Rectangle | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-USE-2 | main | Rectangle | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-1 | calculate_area | Rectangle.width | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-2 | calculate_area | Rectangle.height | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-3 | main | Rectangle.width | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-4 | main | Rectangle.height | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |

## Results 
| Count | Total | Found | Not Found | Error Rate |
| ----- | ----- | ----- | --------- | ---------- |
| Nodes | 7 | 7 | 0 | 0.0000 |
| Edges | 10 | 4 | 6 | 0.6000 |
