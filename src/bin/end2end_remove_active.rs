
use cursive_tabs::TabView;
use cursive::views::TextView;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let mut tabs = TabView::new().with_view(0, TextView::new("First")).with_view(1, TextView::new("Second"));
    tabs.remove_view(1).expect("Removal failed.");
    siv.run();
}
