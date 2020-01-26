use crossbeam::channel::{Receiver, Sender};
use cursive::backend::puppet::observed::ObservedScreen;
use cursive::backend::puppet::Backend;
use cursive::event::{Event, Key};
use cursive::views::TextView;
use cursive::Vec2;
use cursive_tabs::{Align, Placement, TabPanel, TabView};

fn setup_test_environment<F>(cb: F) -> (Receiver<ObservedScreen>, Sender<Option<Event>>)
where
    F: FnOnce(&mut cursive::Cursive),
{
    let backend = Backend::init(Some(Vec2::new(80, 24)));
    let frames = backend.stream();
    let input = backend.input();
    let mut siv = cursive::Cursive::new(|| backend);
    cb(&mut siv);
    input
        .send(Some(Event::Refresh))
        .expect("Refresh not accepted, backend not valid");
    siv.step();
    (frames, input)
}

struct TestCursive {
    siv: cursive::Cursive,
    frames: Receiver<ObservedScreen>,
    input: Sender<Option<Event>>,
}

impl TestCursive {
    fn new<F>(cb: F) -> Self
    where
        F: FnOnce(&mut cursive::Cursive),
    {
        let backend = Backend::init(Some(Vec2::new(80, 24)));
        let frames = backend.stream();
        let input = backend.input();
        let mut siv = cursive::Cursive::new(|| backend);
        cb(&mut siv);
        input
            .send(Some(Event::Refresh))
            .expect("Refresh not accepted, backend not valid");
        siv.step();
        Self { siv, frames, input }
    }
    fn call_on<F>(&mut self, cb: F)
    where
        F: FnOnce(&mut cursive::Cursive),
    {
        cb(&mut self.siv);
    }

    fn input(&mut self, event: Event) {
        self.input
            .send(Some(event))
            .expect("Refresh not accepted, backend could not react");
        self.step();
    }

    fn step(&mut self) {
        self.input
            .send(Some(Event::Refresh))
            .expect("Refresh not accepted, backend could not react");
        self.siv.step();
    }

    fn last_screen(&mut self) -> ObservedScreen {
        self.frames.try_iter().last().unwrap()
    }
}

#[test]
fn test_puppet_screen() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        siv.add_fullscreen_layer(TextView::new(
            "This is a smoke test for the puppet cursive backend.",
        ))
    });
    frames.try_iter().last().unwrap().print_stdout();
    // TODO snapshot test
}

#[test]
fn end2end_add_at() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let tabs = TabView::new()
            .with_tab_at(0, TextView::new("Third"), 0)
            .with_tab_at(1, TextView::new("First"), 0)
            .with_tab_at(2, TextView::new("Second"), 1);
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_add_at_action_change_tab() {
    let mut tsiv = TestCursive::new(|siv: &mut cursive::Cursive| {
        let tabs = TabView::new()
            .with_tab_at(0, TextView::new("Third"), 0)
            .with_tab_at(1, TextView::new("First"), 0)
            .with_tab_at(2, TextView::new("Second"), 1);
        siv.add_layer(tabs);
    });
    tsiv.input(Event::Key(Key::Up));
    tsiv.last_screen().print_stdout();
}

#[test]
fn end2end_add_at_panel() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let tabs = TabPanel::new()
            .with_tab("Stonks", TextView::new("Pshhhh"))
            .with_tab_at("So", TextView::new("Fooooo"), 0)
            .with_tab_at("Much", TextView::new("Ahhhhh"), 1)
            .with_bar_alignment(Align::Center);
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_panel_smoke() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let tabs = TabPanel::new()
            .with_tab("Stronk test", TextView::new("Pshhhh"))
            .with_active_tab("Stronk test")
            .expect("Setting active tab has failed")
            .with_bar_alignment(Align::Center);
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_remove_active() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let mut tabs = TabView::new()
            .with_tab(0, TextView::new("First"))
            .with_tab(1, TextView::new("Second"));
        tabs.remove_tab(1).expect("Removal of active tab failed");
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_remove_inactive() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let mut tabs = TabView::new()
            .with_tab(0, TextView::new("First"))
            .with_tab(1, TextView::new("Second"));
        tabs.remove_tab(0).expect("Removal failed.");
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_swap() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let mut tabs = TabPanel::new()
            .with_tab("Stonks", TextView::new("Pshhhh"))
            .with_tab_at("So", TextView::new("Fooooo"), 0)
            .with_tab_at("Much", TextView::new("Ahhhhh"), 1)
            .with_bar_alignment(Align::Center);
        tabs.swap_tabs("So", "Stonks");
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_switch() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let tabs = TabView::new()
            .with_tab(0, TextView::new("First"))
            .with_tab(1, TextView::new("Second"))
            .with_active_tab(0)
            .expect("Setting tab has failed");
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_vertical_left() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let tabs = TabPanel::new()
            .with_tab("Stronk test", TextView::new("Pshhhh"))
            .with_tab("Stronker test", TextView::new("Pshhhh"))
            .with_active_tab("Stronk test")
            .expect("Setting active tab has failed")
            .with_bar_alignment(Align::Center)
            .with_bar_placement(Placement::VerticalLeft);
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}

#[test]
fn end2end_vertical_left_with_action_change_tab() {
    let mut tsiv = TestCursive::new(|siv: &mut cursive::Cursive| {
        let tabs = TabPanel::new()
            .with_tab("Stronk test", TextView::new("Pshhhh"))
            .with_tab("Stronker test", TextView::new("Pshhhh"))
            .with_active_tab("Stronk test")
            .expect("Setting active tab has failed")
            .with_bar_alignment(Align::Center)
            .with_bar_placement(Placement::VerticalLeft);
        siv.add_layer(tabs);
    });
    tsiv.input(Event::Key(Key::Up));
    tsiv.last_screen().print_stdout();
}

#[test]
fn end2end_vertical_right() {
    let (frames, input) = setup_test_environment(|siv: &mut cursive::Cursive| {
        let tabs = TabPanel::new()
            .with_tab("Stronk test", TextView::new("Pshhhh"))
            .with_active_tab("Stronk test")
            .expect("Setting active tab has failed")
            .with_bar_alignment(Align::Center)
            .with_bar_placement(Placement::VerticalRight);
        siv.add_layer(tabs);
    });
    frames.try_iter().last().unwrap().print_stdout();
}
