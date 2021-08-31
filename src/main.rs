use std::fs::File;
use std::path::Path;
use std::cmp;
use std::io::{self, BufRead, Write, stdout};
use std::time::Duration;
use crossterm::{
    cursor::{self,position},
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers },
    execute, queue, style,
    terminal::{self, ClearType},
    Command, ExecutableCommand, Result
};

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
struct SubEditor {
    pre: usize,
    post: usize,
    text: Vec<char>
}

impl SubEditor {

    fn init() -> SubEditor {
        SubEditor {
            pre: 0,
            post: 32-1,
            text: vec!['\0'; 32]
        }
    }

    fn open(&mut self, path: &str) {
        let path = Path::new(path);

        let lines = match read_lines(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(lines) => lines,            
        };

        let mut i = 1;
        for line in lines {
            match line {
                Ok(content) => { 
                    for ch in content.chars() {
                        self.insert(ch);
                    } self.insert('\n'); 
                    i += 1;
                },
                Err(_) => panic!("Could not read Line {} in file {}", i+1, path.display()),
            }
        } self.move_start(); 
    }

    fn adjust_buffer(&mut self) {
        let free = self.post + 1 - self.pre;
        let cap = self.text.len();

        if free <= 4 {
            self.text.resize(2*cap, '\0');
            let post_len = cap-1-self.post;
            for i in 0..post_len {
                self.text[2*cap-1-i] = self.text[cap-1-i];
            }
            self.post += cap;

        } 
        else if free > 32 && free as f32 > 0.75*self.text.len() as f32 {
            let post_len = cap-1-self.post;
            let mut post_text: Vec<char> = Vec::with_capacity(post_len);
            
            for i in 0..post_len {
                post_text[i] = self.text[cap-1-i];
            } 

            self.text.resize(cap/2, '\0');
            
            for i in 0..post_len {
                self.text[cap/2-1-i] = post_text[i];
            }
            self.post -= cap/2
        }

    }

    fn insert(&mut self, newchar: char) {
        self.adjust_buffer();
        self.text[self.pre] = newchar;
        self.pre += 1;
    }

    fn backspace(&mut self) -> bool {
        if self.pre > 0 {
            self.pre -= 1;
            return true;
        } return false;
    }

    fn move_l(&mut self) -> bool {
        if self.pre > 0 {
            self.pre -= 1;
            self.text[self.post] = self.text[self.pre];
            self.post -= 1;
            return true;
        }
        return false;
    }

    fn move_r(&mut self) -> bool {
        if self.post + 1 < self.text.len() {
            self.post += 1;
            self.text[self.pre] = self.text[self.post];
            self.pre += 1;
            return true;
        }
        return false;
    }

    fn move_start(&mut self) {
        while self.pre > 0 {
            self.move_l();
        }
    }

    fn move_end(&mut self) {
        while self.post < self.text.len() {
            self.move_r();
        }
    }
}

fn main() {
    let mut editor = SubEditor::init();
    editor.open("/home/manan/Projects/rust-editor/files/test1.txt");
    for ch in "manan1".chars() {
        editor.insert(ch);
    }
    editor.backspace();
    editor.move_r();

    println!("{:?}", editor.text);
    println!("({}, {}), {} / {}", editor.pre, editor.post, editor.pre + editor.text.len() - 1 - editor.post, editor.text.len());
}

