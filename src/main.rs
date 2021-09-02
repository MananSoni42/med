mod editor;
use std::io::{self, BufRead, Write, stdout};
use std::time::Duration;
use std::thread;

/*
fn main() {
    let mut editor = subeditor::SubEditor::open("./files/test1.txt");
    editor.move_first();
    editor.move_down();
    editor.move_right(); editor.move_right();
    editor.insert_newline();
    editor.insert('m');
    editor.move_right(); editor.move_right();
    editor.move_up(); editor.move_up();
    editor.show();
}
*/

fn main() {
    let mut ed = editor::Editor {
        term: &mut stdout(),
        subed: editor::subeditor::SubEditor::open("/home/manan/Projects/rust-editor/files/test1.txt")
    };
    ed.init();
    ed.show();
    thread::sleep(Duration::from_millis(4000));
    ed.exit();
}


