use std::fs::File;
use std::path::Path;
use std::cmp;
use std::io::{self, stdout, Write, BufRead};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, Result,
    event,
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

    fn move_hor(&mut self, n: i32) {
        let min_cursor = self.lines[self.curr_line].0;
        let max_cursor = min_cursor + self.lines[self.curr_line].1;
        if n > 0 {
            self.cursor = cmp::min(max_cursor, self.cursor + n as usize);
        } else if n < 0 {
            self.cursor = cmp::max(min_cursor, self.cursor - n as usize);
        }
    }

    fn move_ver(&mut self, n: i32) {
        let max_lines = self.lines.len() as i32;
        self.curr_line = cmp::max(0, cmp::min(max_lines, self.curr_line as i32 + n)) as usize; 
        self.cursor = self.lines[self.curr_line].0;
    }

    fn move_eol(&mut self) {
        self.cursor = self.lines[self.curr_line].0 + self.lines[self.curr_line].1 - 1;
    }

    fn insert(&mut self, newchar: char) {
        self.adjust_buffer();
        if newchar == '\n' {
            self.curr_line += 1;
            self.lines.push((self.cursor+1, 0));
        } else {
            self.pre.push(newchar);
            self.lines[self.curr_line].1 += 1;
            self.cursor += 1;
        }
    }

    fn insert_word(&mut self, newword: &str) {
        self.adjust_buffer();
        let mut curr_len = 0;
        let mut start = self.cursor;
        if newword.trim().len() > 0 {
            for ch in newword.chars() {
                if ch == '\n' {
                    self.lines.push((start, curr_len));
                    self.cursor += curr_len;
                    self.curr_line += 1;
                    start = self.cursor + 1;
                    curr_len = 0                 
                }
                else {
                self.pre.push(ch);
                curr_len += 1;
                }
            }
        }
    }

    fn insert_line(&mut self, newline: &str) {
        self.adjust_buffer();
        let mut curr_len = 0;
        let mut start = self.cursor;
        if newline.trim().len() > 0 {
            for ch in newline.chars() {
                if ch == '\n' {
                    self.lines.push((start, curr_len));
                    self.cursor += curr_len;
                    self.curr_line += 1;

                    start = self.cursor + 1;
                    curr_len = 0                 
                } else {
                    self.pre.push(ch);
                    curr_len += 1
                }
            }
            if curr_len > 0 {
                self.lines.push((start, curr_len));
                self.cursor += curr_len;
                self.curr_line += 1    
            }
        }
    }

    fn backspace(&mut self) {
        if self.cursor > self.lines[self.curr_line].0 {
            self.pre.pop();
            self.cursor -= 1;
            self.lines[self.curr_line].1 -= 1
        } 
    }

    fn delete(&mut self) {
        if self.cursor < self.lines[self.curr_line].0 + self.lines[self.curr_line].1 {
            self.post.pop();
            self.lines[self.curr_line].1 -= 1
        }
    }

    fn delete_line(&mut self) {
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
                let preline:String = if self.cursor > self.lines[ind].0 {
                    self.pre[self.lines[ind].0 .. self.cursor].iter().collect()
                } else {
                    String::new()
                };

                let start = self.post.len() + (self.cursor - self.lines[ind].0) - self.lines[ind].1;
                let postline:String = if self.post.len() > start {
                    self.post[start .. self.post.len()].iter().rev().collect()
                } else {
                    String::new()
                };
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

    editor.move_ver(1);
    editor.move_hor(5);
    editor.insert('w');
    editor.show();
    
    editor.move_hor(-1);
    editor.insert_word("Manan");
    editor.show();
}
