use cursive::views::TextView;
use cursive_tabs::{Align, TabPanel};

fn main() {
    let mut siv = cursive::default();
    let mut tabs = TabPanel::new()
        .with_tab("Stonks", TextView::new("Pshhhh"))
        .with_tab_at("So", TextView::new("Fooooo"), 0)
        .with_tab_at("Much", TextView::new("Ahhhhh"), 1)
        .with_bar_alignment(Align::Center);
    tabs.swap_tabs("So", "Stonks");
    siv.add_layer(tabs);
    siv.run();
}
