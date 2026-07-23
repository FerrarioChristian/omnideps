# Report
## Files
- src/main.c
- src/math_utils.h
- src/math_utils.c
- src/shapes.h
- src/shapes.c
- src/pointers.h
- src/pointers.c
- src/nested_structs.h
- src/nested_structs.c

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
| Point | struct | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Struct |
| Point.x | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| Point.y | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| move_point | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| create_point | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |
| BoundingBox | struct | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Struct |
| BoundingBox.top_left | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| BoundingBox.bottom_right | field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Field |
| get_width | function | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | Function |

## Edges
| Test Id | Source | Sink | Kind | Source Exists | Sink Exists | Edge Exists | Kind Is Correct |
| ------- | ------ | ---- | ---- | ------------- | ----------- | ----------- | --------------- |
| C-CALL-1 | multiply | add | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-CALL-2 | calculate_area | multiply | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-3 | main | calculate_area | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-4 | main | add | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-5 | main | create_point | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-6 | main | move_point | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-CALL-7 | main | get_width | calls | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-USE-1 | calculate_area | Rectangle | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-USE-2 | main | Rectangle | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-USE-3 | move_point | Point | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-USE-4 | create_point | Point | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-USE-5 | BoundingBox.top_left | Point | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-USE-6 | get_width | BoundingBox | uses_type | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) |
| C-ACC-1 | calculate_area | Rectangle.width | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-2 | calculate_area | Rectangle.height | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-3 | main | Rectangle.width | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-4 | main | Rectangle.height | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-5 | move_point | Point.x | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-6 | get_width | BoundingBox.bottom_right | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |
| C-ACC-7 | get_width | Point.x | accesses_field | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![OK](https://github.githubassets.com/images/icons/emoji/unicode/2714.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) | ![KO](https://github.githubassets.com/images/icons/emoji/unicode/274c.png?v8) |

## Results 
| Count | Total | Found | Not Found | Error Rate |
| ----- | ----- | ----- | --------- | ---------- |
| Nodes | 16 | 16 | 0 | 0.0000 |
| Edges | 20 | 9 | 11 | 0.5500 |
