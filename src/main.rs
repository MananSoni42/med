mod editor;
use std::io::{self,stdout};

fn main() -> Result<(), io::Error>{

    let fname = "/home/manan/Projects/rust-editor/files/test1.txt";

    let mut ed = editor::Editor {
        term: &mut stdout(),
        subed: editor::subeditor::SubEditor::open(fname).unwrap(),
        fname: fname,
    };

    if let Err(e) = ed.start() {
        println!("Med stopped unexpectedly :( ({})", e);
        ed.exit()?;
        std::process::exit(1);
    }
    ed.exit()?;
    
    Ok(())
}


