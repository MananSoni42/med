mod subeditor;

fn main() {
    let mut editor = subeditor::SubEditor::open("./files/test1.txt");
    editor.move_u();
    editor.move_u();
    editor.move_u();
    editor.move_d();
    editor.move_r();
    editor.move_r();
    editor.insert_newline();
    editor.show();
}

/*
mod term;
fn main() {
    term::editor_test();
}
*/
