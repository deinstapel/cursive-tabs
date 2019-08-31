use cursive::views::{Dialog, TextView};
use cursive::view::Identifiable;
use cursive::Cursive;
use cursive_tabs::TabView;

fn main() {
    let mut siv = Cursive::default();
    let mut tab = TabView::new()
            .with_view(0, TextView::new("This is the first view!"))
            .with_view(1, TextView::new("This is the second view!"))
            .with_view(2, TextView::new("This is the third view!"))
        .with_view(3, TextView::new("This is the fourth view!"));
    tab.set_tab(0).expect("oh no");

    siv.add_layer(Dialog::around(tab.with_id("Tabs")).button("Next", |siv| {
        let mut tabs: cursive::views::ViewRef<TabView<i32>> = siv.find_id("Tabs").expect("id not found");
        let pos = (tabs.tab().unwrap() + 1) % 4;
        tabs.set_tab(pos).expect("Switch refused");
    }));

    siv.add_global_callback('q', |siv| {
        siv.quit()
    });

    siv.run();
}
