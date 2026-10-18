use econbox_art::ui::{self, Picture, TabMark, Well};
use econbox_art::{GLYPHS, SPRITE, glyph, png};
use std::env;
use std::fs;

fn paste(onto: &mut Picture, piece: &Picture, left: u32, top: u32) {
    for y in 0..piece.height {
        for x in 0..piece.width {
            let pixel = piece.get(x, y);
            if pixel[3] > 0 {
                onto.set(left + x, top + y, pixel);
            }
        }
    }
}

fn main() {
    let out = env::args()
        .nth(1)
        .unwrap_or_else(|| "ui_sheet.png".to_string());
    let columns = 13;
    let cell = 34;
    let rows = GLYPHS.len().div_ceil(columns) as u32;
    let mut sheet = Picture::blank(columns as u32 * cell, rows * cell + 330);

    for y in 0..sheet.height {
        for x in 0..sheet.width {
            sheet.set(x, y, [0x30, 0x34, 0x2e, 255]);
        }
    }

    let well = ui::button(Well::Plain);
    for (at, kind) in GLYPHS.iter().enumerate() {
        let left = (at % columns) as u32 * cell + 1;
        let top = (at / columns) as u32 * cell + 1;
        paste(&mut sheet, &well, left, top);
        let art = ui::framed(&glyph(*kind, None), SPRITE as u32, 32);
        paste(&mut sheet, &art, left, top);
    }

    let base = rows * cell + 4;
    let bar = ui::bar();
    for repeat in 0..(sheet.width / bar.width) {
        paste(&mut sheet, &bar, repeat * bar.width, base);
    }

    for (at, mark) in [
        TabMark::Terrain,
        TabMark::Minds,
        TabMark::Creatures,
        TabMark::Nature,
        TabMark::Ruin,
        TabMark::Misc,
    ]
    .iter()
    .enumerate()
    {
        let left = 40 + at as u32 * 44;
        let tab = ui::tab(at == 2);
        paste(&mut sheet, &tab, left, base + 140);
        paste(&mut sheet, &ui::tab_mark(*mark), left + 5, base + 143);
    }

    paste(&mut sheet, &ui::button(Well::Studded), 20, base + 170);
    paste(&mut sheet, &ui::button(Well::Present), 56, base + 170);
    paste(&mut sheet, &ui::selection(), 92, base + 170);
    paste(&mut sheet, &ui::arrow(), 130, base + 170);
    paste(&mut sheet, &ui::tooltip_body(), 150, base + 170);
    paste(&mut sheet, &ui::tooltip_head(), 174, base + 170);
    paste(&mut sheet, &ui::window_frame(), 200, base + 170);
    paste(&mut sheet, &ui::close_tab(), 270, base + 170);
    paste(&mut sheet, &ui::red_plate(32, 68), 316, base + 170);

    let mut badge = ui::button(Well::Plain);
    ui::stamp_text(&mut badge, "x0.5", 29, 29);
    paste(&mut sheet, &badge, 360, base + 170);

    fs::write(&out, png(sheet.width, sheet.height, &sheet.bytes())).expect("write sheet");
    println!("{out}");
}
