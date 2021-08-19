use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::cmp;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
struct SubEditor {
    cursor: usize,
    curr_line: usize,
    pre: Vec<char>,
    post: Vec<char>,
    lines: Vec<(usize, usize)>
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
        self.cursor = 0;
        self.curr_line = 0;
        self.adjust_buffer(); 
    }

    fn adjust_buffer(&mut self) {
        let buff_cursor = self.pre.len();
        if self.cursor == buff_cursor { 
            return; 
        }
        else {
            let deficit = if self.cursor > buff_cursor { self.cursor - buff_cursor } else { buff_cursor - self.cursor };
            for _ in 0..deficit {
                if self.cursor < buff_cursor {
                    self.post.push(self.pre.pop().unwrap());
                } else {
                    self.pre.push(self.post.pop().unwrap());
                }
            }
        }
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

    fn move_eol(&mut self) {
        self.cursor = self.lines[self.curr_line].0 + self.lines[self.curr_line].1;
    }

    fn insert(&mut self, newchar: &char) {
        self.adjust_buffer();
    }

    fn insert_word(&mut self, newword: &str) {
        self.adjust_buffer();
    }

    fn insert_line(&mut self, newline: &str) {
        self.adjust_buffer();
        let linelen = newline.len();
        if linelen > 0 {
            self.lines.push((self.cursor, linelen));
            for ch in newline.chars() {
                self.pre.push(ch);
            }
            self.cursor += linelen;
            self.curr_line += 1    
        }
    }

    fn show(&mut self) {
        self.adjust_buffer();
        println!("--- Med: v0.1 ---");
        for (ind,line) in self.lines.iter().enumerate() {
            let mut currline = String::new();
            if ind < self.curr_line {
                currline = self.pre[self.lines[ind].0 .. self.lines[ind].0+self.lines[ind].1].iter().collect();
            } else if ind > self.curr_line {
                let end = self.post.len() - (self.lines[ind].0 - self.cursor);
                currline = self.post[end-self.lines[ind].1 .. end].iter().rev().collect();
            } else {
                let preline:String = self.pre[self.lines[ind].0 .. self.cursor].iter().collect();
                let start = self.post.len() - self.lines[ind].1 + (self.cursor - self.lines[ind].0);
                let postline:String = self.post[start .. self.post.len()].iter().rev().collect();
                currline = format!("{}{}",preline,postline);
            }
            println!("{}| {:?}",ind, currline);
        }
        println!("-----------------\n");
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
    editor.show();
}
