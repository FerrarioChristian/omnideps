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
| math_utils | file | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| shapes | file | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| main | file | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| math_utils.add | function | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| math_utils.multiply | function | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| shapes.Rectangle | struct | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| shapes.Rectangle.width | field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| shapes.Rectangle.height | field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| shapes.calculate_area | function | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| main.main | function | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |
| main.global_counter | variable | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | - |

## Edges
| Test Id | Source | Sink | Kind | Source Exists | Sink Exists | Edge Exists | Kind Is Correct |
| ------- | ------ | ---- | ---- | ------------- | ----------- | ----------- | --------------- |
| C-INC-1 | math_utils.c | math_utils.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-INC-2 | shapes.c | shapes.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-INC-3 | shapes.c | math_utils.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-INC-4 | main.c | shapes.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-INC-5 | main.c | math_utils.h | includes | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-CALL-1 | math_utils.multiply | math_utils.add | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-CALL-2 | shapes.calculate_area | math_utils.multiply | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-CALL-3 | main.main | shapes.calculate_area | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-CALL-4 | main.main | math_utils.add | calls | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-USE-1 | shapes.calculate_area | shapes.Rectangle | uses_type | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-USE-2 | main.main | shapes.Rectangle | uses_type | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-1 | shapes.calculate_area | shapes.Rectangle.width | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-2 | shapes.calculate_area | shapes.Rectangle.height | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-3 | main.main | shapes.Rectangle.width | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-4 | main.main | shapes.Rectangle.height | accesses_field | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-5 | main.main | main.global_counter | accesses_variable | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |

## Results 
| Count | Total | Found | Not Found | Error Rate |
| ----- | ----- | ----- | --------- | ---------- |
| Nodes | 11 | 1 | 10 | 0.9091 |
| Edges | 16 | 0 | 16 | 1.0000 |
