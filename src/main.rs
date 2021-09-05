mod editor;
use std::io::{self, BufRead, Write, stdout};

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

    let fname = "/home/manan/Projects/rust-editor/files/test1.txt";

    let mut ed = editor::Editor {
        term: &mut stdout(),
        subed: editor::subeditor::SubEditor::open(fname),
        fname: fname,
    };

    ed.init();
    
    ed.start();
    
    ed.exit();
}


