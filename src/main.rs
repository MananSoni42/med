use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::cmp;

#[derive(Debug)]
struct SubEditor {
    cursor: usize,
    curr_line: usize,
    pre: Vec<char>,
    post: Vec<char>,
    lines: Vec<(usize, usize)>
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

impl SubEditor {

    fn init() -> SubEditor {
        SubEditor {
            cursor: 0,
            curr_line: 0,
            pre: Vec::new(),
            post: Vec::new(),
            lines: Vec::new(),  
        }
    }

    fn open(&mut self, path: &str) {
        let path = Path::new(path);

        let lines = match read_lines(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(lines) => lines,            
        };

        for line in lines {
            match line {
                Ok(content) => self.insert_line(&content),
                Err(_) => ()
            }
        }      
    }

    fn adjust_buffer(&mut self) {
        let buff_cursor = self.pre.len();
        if self.cursor == buff_cursor { 
            return; 
        }
        else {
            let deficit = if self.cursor > buff_cursor { self.cursor - buff_cursor } else { buff_cursor - self.cursor };
            for _ in 0..deficit {
                if self.cursor > buff_cursor {
                    self.post.push(self.pre.pop().unwrap()) 
                } else {
                    self.pre.push(self.post.pop().unwrap()) 
                }
            }
        }
    }

    fn move_l(&mut self) {
        self.cursor -= 1;
    }
    fn move_r(&mut self) {
        self.cursor += 1;
    }
    fn move_u(&mut self) {
        self.curr_line = cmp::max(0, self.curr_line-1);
        self.cursor = self.lines[self.curr_line].0
    }
    fn move_d(&mut self) {
        self.curr_line = cmp::min(self.lines.len()-1, self.curr_line+1);
        self.cursor = self.lines[self.curr_line].0
    }

    fn move_hor(&mut self, n: usize) {
    }
    fn move_ver(&mut self, n: usize) {
    }

    fn insert(&mut self, newchar: &char) {
    }

    fn insert_word(&mut self, newword: &str) {
    }

    fn insert_line(&mut self, newline: &str) {
        self.lines.push((self.cursor, newline.len()));
        for ch in newline.chars() {
            self.pre.push(ch);
        }
        self.cursor += newline.len();
        self.curr_line += 1
    }

    fn show(&self) {
        println!("line: {} | cursor: {}", self.curr_line, self.cursor);
        println!("pre: {:?}", self.pre);
        println!("post: {:?}", self.post);
    }
}

fn main() {
    let mut editor = SubEditor::init();
    editor.open("/home/manan/Projects/rust-editor/files/test1.txt");
    editor.show();

    editor.move_d();
    for _ in 0..5 {
        editor.move_r();
    }
    editor.adjust_buffer();
    editor.show();
}
