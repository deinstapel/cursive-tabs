use cursive_tabs::TabView;
use cursive::views::TextView;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let mut tabs = TabView::new().with_view(0, TextView::new("First")).with_view(1, TextView::new("Second"));
    tabs.set_tab(0).expect("Setting tab has failed");
    siv.run();
}
