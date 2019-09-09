use cursive::views::TextView;
use cursive_tabs::TabView;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let mut tabs = TabView::new()
        .tab(0, TextView::new("First"))
        .tab(1, TextView::new("Second"));
    tabs.remove_tab(1).expect("Removal failed.");
    siv.add_layer(tabs);
    siv.run();
}
