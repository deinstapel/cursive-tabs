use cursive::views::TextView;
use cursive_tabs::TabView;
use cursive::CursiveExt;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabView::new()
        .with_tab_at(0, TextView::new("Third"), 0)
        .with_tab_at(1, TextView::new("First"), 0)
        .with_tab_at(2, TextView::new("Second"), 1);
    siv.add_layer(tabs);
    siv.run();
}
