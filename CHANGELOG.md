## version 0.6.0

## version 0.5.0

- Change `remove_tab` and `swap_tab` to take references to values instead of values itself
- Change trait requirements to less specific `Clone` instead of `Copy`
- `with_active_tab` will now return also in error cases `self` encapsulated in the `Result`

## version 0.3.0

- Addition of `set_bar_placement` and `with_bar_placement` to control the position of the bar inside of the `TabPanel`
- Change `HAlign` to `Align` to represent all directions. Possible values are `Start`, `Center` and `End`
- New examples can be found that show the updated usage
