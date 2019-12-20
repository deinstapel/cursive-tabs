use cursive::views::TextView;
use cursive_tabs::{Align, TabPanel};

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabPanel::new()
        .with_tab("Stronk test", TextView::new("Pshhhh"))
        .with_active_tab("Stronk test")
        .expect("Setting active tab has failed")
        .with_bar_alignment(Align::Center);
    siv.add_layer(tabs);
    siv.run();
}
