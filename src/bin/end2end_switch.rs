use cursive::views::TextView;
use cursive_tabs::TabView;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabView::new()
        .tab(0, TextView::new("First"))
        .tab(1, TextView::new("Second"))
        .active_tab(0).expect("Setting tab has failed");
    siv.add_layer(tabs);
    siv.run();
}
