use cursive::views::TextView;
use cursive_tabs::TabPanel;

use cursive::{align::HAlign, Cursive};

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabPanel::new()
        .with_tab("Stronk test", TextView::new("Pshhhh"))
        .with_active_tab("Stronk test")
        .expect("Setting active tab has failed")
        .with_bar_alignment(HAlign::Center);
    siv.add_layer(tabs);
    siv.run();
}
