use cursive::align::HAlign;
use cursive::view::{Boxable, Identifiable};
use cursive::views::{Button, LinearLayout, PaddedView, TextArea, TextView};
use cursive::Cursive;
use cursive_tabs::TabPanel;

fn main() {
    let mut siv = Cursive::default();
    let mut panel = TabPanel::new()
        .with_view(0, TextView::new("With using the TabPanel you get a TabView and TabBar, preconfigured for you to use!
Simply create it with:

`cursive_tabs::TabPanel::new()`"))
        .with_view(1, TextView::new("You then can add views and configure your panel."))
        .with_view(2, TextView::new("Ofcourse you can also use the provided TabView without the panel, simply create it with:

`cursive_tabs::TabView::new()`"))
.with_view(3, TextView::new("All you have to do is add:

cursive-tabs = \"0^\"

to your Cargo.toml!
"))
        .with_view(4, PaddedView::new((2,2,1,1),TextArea::new()))
        .with_bar_align(HAlign::Right);
    panel.set_tab(0).expect("oh no");

    siv.add_layer(
        LinearLayout::vertical()
            .child(panel.with_id("Tabs").fixed_size((50, 10)))
            .child(
                LinearLayout::horizontal()
                    .child(Button::new("Prev", |siv| {
                        let mut tabs: cursive::views::ViewRef<TabPanel<i32>> =
                            siv.find_id("Tabs").expect("id not found");
                        tabs.prev();
                    }))
                    .child(Button::new("Next", |siv| {
                        let mut tabs: cursive::views::ViewRef<TabPanel<i32>> =
                            siv.find_id("Tabs").expect("id not found");
                        tabs.next();
                    })),
            ),
    );

    siv.add_global_callback('q', |siv| siv.quit());

    siv.run();
}
