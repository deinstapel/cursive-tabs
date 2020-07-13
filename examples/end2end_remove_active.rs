use cursive::views::TextView;
use cursive_tabs::TabView;

fn main() {
    let mut siv = cursive::default();
    let mut tabs = TabView::new()
        .with_tab(0, TextView::new("First"))
        .with_tab(1, TextView::new("Second"));
    tabs.remove_tab(&1).expect("Removal of active tab failed");
    siv.add_layer(tabs);
    siv.run();
}
