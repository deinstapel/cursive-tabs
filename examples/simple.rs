use cursive::view::{Boxable, Identifiable};
use cursive::views::{Button, LinearLayout, PaddedView, TextArea, TextView};
use cursive::Cursive;
use cursive_tabs::TabPanel;

fn main() {
    let mut siv = Cursive::default();
    let mut panel = TabPanel::new()
        .with_view(0, TextView::new("This is one of the first views, definetly not the last. There will be more to come in due time, but at the moment that is all we have, but in the future there shall be plenty more!"))
        .with_view(1, PaddedView::new((2,2,1,1),TextArea::new()))
        .with_view(2, TextView::new("This is the third view!"))
        .with_view(3, TextView::new("This is the fourth view!"));
    panel.set_tab(0).expect("oh no");

    siv.add_layer(
        LinearLayout::vertical()
            .child(panel.with_id("Tabs").fixed_size((30, 20)))
            .child(Button::new("Next", |siv| {
                let mut tabs: cursive::views::ViewRef<TabPanel<i32>> =
                    siv.find_id("Tabs").expect("id not found");
                let pos = (tabs.tab().unwrap() + 1) % 4;
                tabs.set_tab(pos).expect("Switch refused");
            })),
    );

    siv.add_global_callback('q', |siv| siv.quit());

    siv.run();
}
