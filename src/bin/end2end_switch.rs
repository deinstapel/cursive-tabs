use cursive::views::TextView;
use cursive_tabs::TabView;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabView::new()
        .with_tab(0, TextView::new("First"))
        .with_tab(1, TextView::new("Second"))
        .with_active_tab(0)
        .expect("Setting tab has failed");
    siv.add_layer(tabs);
    siv.run();
}
