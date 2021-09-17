mod editor;
use std::io::{self,stdout};
use crossterm::{terminal};

fn main() -> Result<(), io::Error> {

    let fname = "/home/manan/Projects/rust-editor/files/test1.txt";

    let mut ed = editor::Editor {
        term: &mut stdout(),
        subed: editor::subeditor::SubEditor::open(fname).unwrap(),
        fname: fname,
        xscroll: 0,
        yscroll: 0,
        size: terminal::size().unwrap()
    };

    if let Err(e) = ed.start() {
        ed.exit()?;
        println!("Med stopped unexpectedly!");
        println!("Error: {}", e);
        std::process::exit(1);
    }
    ed.exit()?;
    
    Ok(())
}


