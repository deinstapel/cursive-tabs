<h1 align="center">Welcome to cursive-tabs 👋</h1>
<p align="center">
  <a href="https://travis-ci.org/deinstapel/cursive-tabs">
    <img src="https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fdeinstapel.github.io%2Fcursive-tabs%2Fstable-build.json" alt="stable build">
  </a>
  <a href="https://travis-ci.org/deinstapel/cursive-tabs">
    <img src="https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fdeinstapel.github.io%2Fcursive-tabs%2Fnightly-build.json" alt="nightly build">
  </a>
  <a href="https://crates.io/crates/cursive-tabs">
    <img alt="crates.io" src="https://img.shields.io/crates/v/cursive-tabs.svg">
  </a>
  <a href="https://docs.rs/cursive-tabs">
    <img alt="Docs.rs" src="https://docs.rs/cursive-tabs/badge.svg">
  </a>
  <a href="https://github.com/deinstapel/cursive-tabs/blob/master/LICENSE">
    <img alt="GitHub" src="https://img.shields.io/github/license/deinstapel/cursive-tabs.svg">
  </a>
  <a href="http://makeapullrequest.com">
    <img alt="PRs Welcome" src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg">
  </a>
  <br>
  <i>A tab view wrapper for
  <a href="https://github.com/gyscos/cursive">gyscos/cursive</a>
  views</i>
</p>

---

> This project is work-in-progress

This project provides a wrapper view to be able to easily handle multiple tabs that can be switched to at any time without having to change the order of the views for [gyscos/cursive](https://github.com/gyscos/cursive) views.

## How does it look like? `demo` [![terminalizer](https://img.shields.io/badge/GIF-terminalizer-blueviolet.svg)](https://github.com/faressoft/terminalizer)

<details>
  <summary>Expand to view</summary>
  <img src="assets/demo.gif" alt="tabs demo">
</details>

## Usage

Simply add to your `Cargo.toml`

```toml
[dependencies]
cursive-tabs = "^0"
```

### Creating your `TabView` and add tabs

This crate provides a struct `TabView` you can use to add tabs and switch between them.

```rust
use cursive::{views::TextView, Cursive};
use cursive_tabs::TabView;

let mut siv = Cursive::default();
let tabs = TabView::new().with_view(0, TextView::new("Our first tab!"));
// We can continue to add as many tabs as we want!

siv.add_layer(tabs);
siv.run();
```

## Troubleshooting

If you find any bugs/unexpected behaviour or you have a proposition for future changes open an issue describing the current behaviour and what you expected.

## Development [![cargo test](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fdeinstapel.github.io%2Fcursive-tabs%2Fcargo-test.json)](https://travis-ci.org/deinstapel/cursive-tabs) [![shellshot](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fdeinstapel.github.io%2Fcursive-tabs%2Fshellshot.json)](https://github.com/fin-ger/shellshot)

> TBD

### Running the tests

> :bangbang: **CAUTION** :bangbang: This crate uses Tmux for end2end testing and will **kill your Tmux server** during testing!

#### Preparing integration tests

In order to run the integration tests, you first need to install a recent version of `>=npm-10` and `>=tmux-2.6`!

After `npm` and `tmux` are installed, install required dependencies:

```
$ ./scripts/prepare-end2end-tests.sh
```

This will use `npm` to install `jest` and `shellshot` in the `tests` folder.

#### Running all test suites

Just run

```
$ cargo test
```

to execute all available tests.

#### shields.io endpoints

[shields.io](https://shields.io) endpoints are generated inside the `./target/shields` folder. They are used in this README.

## Authors

**Fin Christensen**

> [:octocat: `@fin-ger`](https://github.com/fin-ger)  
> [:elephant: `@fin_ger@mastodon.social`](https://mastodon.social/web/accounts/787945)  
> [:bird: `@fin_ger_github`](https://twitter.com/fin_ger_github)  

<br>

**Johannes Wünsche**

> [:octocat: `@jwuensche`](https://github.com/jwuensche)  
> [:elephant: `@fredowald@mastodon.social`](https://mastodon.social/web/accounts/843376)  
> [:bird: `@Fredowald`](https://twitter.com/fredowald)  

## Show your support

Give a :star: if this project helped you!
