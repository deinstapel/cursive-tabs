use cursive::view::{Nameable, Resizable};
use cursive::views::{Button, LinearLayout, NamedView, PaddedView, TextArea, TextView};
use cursive_tabs::{Align, TabPanel};

const TAB_0: &str =
    "With using the TabPanel you get a TabView and TabBar, preconfigured for you to use!
Simply create it with:

`cursive_tabs::TabPanel::new()`";

const TAB_1: &str = "You then can add views and configure your panel.";

const TAB_2: &str =
    "Ofcourse you can also use the provided TabView without the panel, simply create it with:

`cursive_tabs::TabView::new()`";

const TAB_3: &str = "All you have to do is add:

cursive-tabs = \"^0\"

to your Cargo.toml!
";

fn main() {
    let mut siv = cursive::default();
    let mut panel = TabPanel::new()
        .with_tab(TextView::new(TAB_0).with_name("0"))
        .with_tab(TextView::new(TAB_1).with_name("1"))
        .with_tab(TextView::new(TAB_2).with_name("2"))
        .with_tab(TextView::new(TAB_3).with_name("3"))
        .with_tab(PaddedView::lrtb(2, 2, 1, 1, TextArea::new()).with_name("4"))
        .with_bar_alignment(Align::End);

    let view = panel
        .active_view_mut()
        .unwrap()
        .downcast_mut::<NamedView<PaddedView<TextArea>>>()
        .unwrap();
    view.get_mut()
        .get_inner_mut()
        .set_content("This is additional text, set after the creation of the view!");
    panel.set_active_tab("0").expect("View not found");

    siv.add_layer(
        LinearLayout::vertical()
            .child(panel.with_name("Tabs").fixed_size((50, 10)))
            .child(
                LinearLayout::horizontal()
                    .child(Button::new("Prev", |siv| {
                        let mut tabs: cursive::views::ViewRef<TabPanel> =
                            siv.find_name("Tabs").expect("id not found");
                        tabs.prev();
                    }))
                    .child(Button::new("Next", |siv| {
                        let mut tabs: cursive::views::ViewRef<TabPanel> =
                            siv.find_name("Tabs").expect("id not found");
                        tabs.next();
                    }))
                    .child(Button::new("Switch", |siv| {
                        let mut tabs: cursive::views::ViewRef<TabPanel> =
                            siv.find_name("Tabs").expect("id not found");
                        tabs.swap_tabs("1", "2");
                    })),
            ),
    );

    siv.add_global_callback('q', |siv| siv.quit());

    siv.run();
}
