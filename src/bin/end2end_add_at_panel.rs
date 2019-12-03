use cursive::views::TextView;
use cursive_tabs::TabPanel;

use cursive::{align::HAlign, Cursive};

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabPanel::new()
        .with_tab("Stonks", TextView::new("Pshhhh"))
        .with_tab_at("So", TextView::new("Fooooo"), 0)
        .with_tab_at("Much", TextView::new("Ahhhhh"), 1)
        .with_bar_alignment(HAlign::Center);
    siv.add_layer(tabs);
    siv.run();
}
