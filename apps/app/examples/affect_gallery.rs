#[path = "affect_gallery/main.rs"]
mod gallery;

pub use gallery::{Item, Section, glyph};

fn main() {
    gallery::main();
}
