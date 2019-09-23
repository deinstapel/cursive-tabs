use cursive::align::HAlign;
use cursive::view::{Boxable, Identifiable};
use cursive::views::{Button, LinearLayout, PaddedView, TextArea, TextView};
use cursive::Cursive;
use cursive_tabs::TabPanel;

const TAB_0: &'static str =
    "With using the TabPanel you get a TabView and TabBar, preconfigured for you to use!
Simply create it with:

`cursive_tabs::TabPanel::new()`";

const TAB_1: &'static str = "You then can add views and configure your panel.";

const TAB_2: &'static str =
    "Ofcourse you can also use the provided TabView without the panel, simply create it with:

`cursive_tabs::TabView::new()`";

const TAB_3: &'static str = "All you have to do is add:

cursive-tabs = \"0^\"

to your Cargo.toml!
";

fn main() {
    let mut siv = Cursive::default();
    let panel = TabPanel::new()
        .with_tab(0, TextView::new(TAB_0))
        .with_tab(1, TextView::new(TAB_1))
        .with_tab(2, TextView::new(TAB_2))
        .with_tab(3, TextView::new(TAB_3))
        .with_tab(4, PaddedView::new((2, 2, 1, 1), TextArea::new()))
        .with_bar_alignment(HAlign::Right)
        .with_active_tab(0)
        .expect("oh no");

    siv.add_fullscreen_layer(
        LinearLayout::vertical()
            .child(panel.with_id("Tabs").full_screen())
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
