use cursive::views::TextView;
use cursive_tabs::{Align, Placement, TabPanel};
use cursive::CursiveExt;

use cursive::Cursive;

fn main() {
    let mut siv = Cursive::default();
    let tabs = TabPanel::new()
        .with_tab("Si!", TextView::new("Pshhhh"))
        .with_tab("Ridiculous!", TextView::new("Pshhhh"))
        .with_tab("A much shorter one", TextView::new("Pshhhh"))
        .with_tab(
            "A very long and strong description",
            TextView::new("Pshhhh"),
        )
        .with_active_tab("Si!")
        .expect("Setting active tab has failed")
        .with_bar_alignment(Align::Center)
        .with_bar_placement(Placement::VerticalRight);
    siv.add_layer(tabs);
    siv.run();
}
