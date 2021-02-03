## version 0.6.0
- Removal of tab ids, this release replace the usage of the internal generic key used in `cursive-tabs` with the `cursive` native `NamedView`, this implies that only `NamedView`s can be added now to tabs.

Any code adding views will need to be migrated. This migration can easily be done by using the `cursive::view::Nameable` trait, a small example on how to do this is shown below.

```rust
// old style add
tabs.add_tab(42, TextView::new("Crabs! 🦀"));

// new add
tabs.add(TextView::new("Crabs! 🦀").with_name("42"));
```

Notable differences between both styles is that only strings are taken as identifier, for most use cases this will be perfectly fine. 
Other use cases requiring the usage of more complex structures can use the `format!` macro to attain a `String` value.

```rust
tabs.add(TextView::new("Crabs! 🦀").with_name(format!("{}", my_super_key));
```

- Many small changes were required to allow for a more `cursive` like interface, mainly to do with other methods receiving identifiers to remove, or switch views, all methods operating on keys now receive `&str`

Affected Methods:
```rust
pub fn remove_tab(&mut self, id: &str) -> Result<(),()>
pub fn swap_tabs(&mut self, fst: &str, snd: &str)
pub fn add_tab_at<T: View>(&mut self, view: NamedView<T>, pos: usize)
pub fn with_tab_at<T: View>(mut self, view: NamedView<T>, pos: usize) -> Self
pub fn add_tab<T: View>(&mut self, view: NamedView<T>)
pub fn with_tab<T: View>(mut self, view: NamedView<T>) -> Self
pub fn with_active_tab(mut self, id: &str) -> Result<Self, Self>
pub fn set_active_tab(&mut self, id: &str) -> Result<(), ()>
```
## version 0.5.0

- Change `remove_tab` and `swap_tab` to take references to values instead of values itself
- Change trait requirements to less specific `Clone` instead of `Copy`
- `with_active_tab` will now return also in error cases `self` encapsulated in the `Result`

## version 0.3.0

- Addition of `set_bar_placement` and `with_bar_placement` to control the position of the bar inside of the `TabPanel`
- Change `HAlign` to `Align` to represent all directions. Possible values are `Start`, `Center` and `End`
- New examples can be found that show the updated usage
