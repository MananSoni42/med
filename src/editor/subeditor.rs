use std::fs::File;
use std::path::Path;
use std::io::{self,BufRead,Write};
use std::iter::Iterator;
mod line;
use line::Line;


// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
pub struct SubEditor {
    prelines: Vec<Line>,
    postlines: Vec<Line>
}

impl SubEditor {
    pub fn open(path: &str) -> SubEditor {

        let mut subed = SubEditor {
            prelines: Vec::new(),
            postlines: Vec::new()
        };

        let path = Path::new(path);

        let lines = match read_lines(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(lines) => lines,            
        };

        let mut first_line = true;
        for line in lines {
            match line {
                Ok(content) => {
                    if first_line {
                        subed.prelines.push(Line::init_with_line(content)); 
                        first_line = false;
                    } else {
                        subed.postlines.push(Line::init_with_line(content));
                    }
                },
                Err(_) => panic!("Could not read Line {} in file {}", subed.curr_line() + 1 as usize, path.display()),
            }
        }
        subed.postlines.reverse();
        return subed;
    }

    pub fn curr_line(&self) -> usize {
        self.prelines.len() - 1 as usize
    }

    pub fn get_lines(&self) -> Vec<&line::Line> {
        self.prelines.iter().chain(self.postlines.iter().rev()).collect()
    }

    pub fn linelen(&self) -> usize {
        let curr_line = self.curr_line();
        self.prelines[curr_line].len()
    }

    pub fn cursor(&self) -> usize {
        let curr_line = self.curr_line();
        self.prelines[curr_line].cursor()
    }

    pub fn move_left(&mut self) -> bool {
        let curr_line = self.curr_line();
        self.prelines[curr_line].move_l()
    }

    pub fn move_right(&mut self) -> bool {
        let curr_line = self.curr_line();
        self.prelines[curr_line].move_r()
    }

    pub fn move_down(&mut self) -> bool {
        if self.postlines.len() > 0 {
            self.prelines.push(self.postlines.pop().unwrap());
            true
        } else {
            false
        }
    }
 
    pub fn move_up(&mut self) -> bool {
        if self.prelines.len() > 1 {
            self.postlines.push(self.prelines.pop().unwrap());
            true
        } else {
            false
        }
    }

    pub fn move_start(&mut self) {
        let curr_line = self.curr_line();
        self.prelines[curr_line].move_start()
    }

    pub fn move_end(&mut self) {
        let curr_line = self.curr_line();
        self.prelines[curr_line].move_end()
    }

    pub fn move_first(&mut self) {
        while self.move_up() { }
        let curr_line = self.curr_line();
        self.prelines[curr_line].move_start();
    }

    pub fn move_last(&mut self) {
        while self.move_down() { }
        let curr_line = self.curr_line();
        self.prelines[curr_line].move_start();
    }

    pub fn backspace(&mut self) -> bool {
        let curr_line = self.curr_line();
        self.prelines[curr_line].backspace()
    }

    pub fn delete(&mut self) -> bool {
        if self.move_right() {
            let curr_line = self.curr_line();
            return self.prelines[curr_line].backspace();
        }
        return false;
    }

    pub fn backspace_line(&mut self) -> bool {
        let curr_line = self.curr_line();
        if self.prelines[curr_line].len() == 0 {
            if self.move_down() {
                self.move_up();
                self.prelines.pop();                
                self.move_down();
                return false;
            } else if self.move_up() {
                self.move_down();
                self.postlines.pop();
                self.move_up();
                return true;
            }
            else {
                panic!("Anakin start panikin, no lines")
            }
        }

        false
    }

    pub fn insert(&mut self, newchar: char) {
        let curr_line = self.curr_line();
        self.prelines[curr_line].insert(newchar);
    }

    pub fn insert_newline(&mut self) {
        let mut newline = String::new();
        let curr_line = self.curr_line();
        let mut cline = &mut self.prelines[curr_line];
        for i in cline.get_post()+1..cline.get_len() {
            newline.push(cline.get_text(i)); 
        }

        cline.set_post(cline.get_len()-1);
        self.prelines.push(Line::init_with_line(newline));
    }

    pub fn show_curr_line(&mut self) -> String {
        self.prelines[self.curr_line()].show()
    }

    pub fn show_curr_post_line(&mut self) -> String {
        let mut post_line = String::new();
        let cline = &self.prelines[self.curr_line()];
        for i in cline.get_post()+1..cline.get_len() { 
            post_line.push(self.prelines[self.curr_line()].get_text(i)); 
        }

        post_line
    }

    pub fn show(&self) {
        println!("line: {}, cursor: {}", self.curr_line() + 1 as usize, self.cursor() + 1 as usize);
        for (i,cline) in self.get_lines().iter().enumerate() {
            println!("{} | {}", i+1, cline.show());
        }
    }

    pub fn save(&self, path: &str) {

        let path = Path::new(path);
        let mut file = File::create(path).unwrap();

        for cline in self.get_lines() {
            cline.save(&mut file);
        }
    }
}    
