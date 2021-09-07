use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead};
use std::iter::Iterator;
mod line;
use line::Line;

// The output is wrapped in a Result to allow matching on errors
// Returns an Iteratfalseor to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub enum DEL {
    Yes,
    No,
    NewLine(usize)
}

#[derive(Debug)]
pub struct SubEditor {
    prelines: Vec<Line>,
    postlines: Vec<Line>
}

impl SubEditor {

    pub fn open(path: &str) -> Result<SubEditor, io::Error> {

        let mut subed = SubEditor {
            prelines: Vec::new(),
            postlines: Vec::new()
        };

        let path = Path::new(path);

        match read_lines(&path) {
            Ok(lines) => { 
                let mut first_line = true;
                for line in lines {
                    if first_line {
                        subed.prelines.push(Line::init_with_line(line?)); 
                        first_line = false;
                    } else {
                        subed.postlines.push(Line::init_with_line(line?));
                    }
                }        
            },
            Err(_) => { 
                std::fs::create_dir_all(path)?;
                subed.prelines.push(Line::init());
            }
        };

        subed.postlines.reverse();
        Ok(subed)
    }

    pub fn curr_line_num(&self) -> usize {
        self.prelines.len() - 1 as usize
    }

    pub fn num_lines(&self) -> usize {
        self.prelines.len() + self.postlines.len()
    }

    pub fn curr_line(&self) -> String {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].show()
    }

    pub fn get_lines(&self) -> Vec<&line::Line> {
        self.prelines.iter().chain(self.postlines.iter().rev()).collect()
    }

    pub fn get_post_lines(&self) -> &Vec<Line> {
        &self.postlines
    }

    pub fn linelen(&self) -> usize {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].len()
    }

    pub fn cursor(&self) -> usize {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].cursor()
    }

    pub fn move_left(&mut self) -> bool {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].move_l()
    }

    pub fn move_right(&mut self) -> bool {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].move_r()
    }

    pub fn move_down(&mut self) -> bool {
        let old_cursor = self.cursor();
        if self.postlines.len() > 0 {
            self.prelines.push(self.postlines.pop().unwrap());
            if self.linelen() < old_cursor { self.move_end(); }
            else if self.cursor() < old_cursor {
                while self.cursor() < old_cursor { self.move_right(); }
            } 
            else if self.cursor() > old_cursor {
                while self.cursor() > old_cursor { self.move_left(); }
            } 
            true
        } else {
            false
        }
    }
 
    pub fn move_up(&mut self) -> bool {
        let old_cursor = self.cursor();
        if self.prelines.len() > 1 {
            self.postlines.push(self.prelines.pop().unwrap());
            if self.linelen() < old_cursor { self.move_end(); }
            else if self.cursor() < old_cursor {
                while self.cursor() < old_cursor { self.move_right(); }
            } 
            else if self.cursor() > old_cursor {
                while self.cursor() > old_cursor { self.move_left(); }
            } 
            true
        } else {
            false
        }
    }

    pub fn move_start(&mut self) {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].move_start()
    }

    pub fn move_end(&mut self) {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].move_end()
    }

    pub fn move_first(&mut self) {
        while self.prelines.len() > 1 { self.postlines.push(self.prelines.pop().unwrap()); }
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].move_start();
    }

    pub fn move_last(&mut self) {
        while self.postlines.len() > 0 { self.prelines.push(self.postlines.pop().unwrap()); }
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].move_start();
    }

    pub fn backspace(&mut self) -> DEL {
        let curr_line = self.curr_line_num();
        if self.prelines[curr_line].backspace() {
            DEL::Yes
        } else if self.prelines.len() > 1 {
            let nline = self.prelines[curr_line].show();
            self.prelines.pop();
            let curr_line = self.curr_line_num();
            self.prelines[curr_line].move_end();
            let linelen = self.linelen();
            for ch in nline.chars() { self.prelines[curr_line].insert(ch) }
            for _ in (linelen..self.linelen()) { self.move_left(); }
            DEL::NewLine(linelen)
        } else {
            DEL::No
        }
    }

    pub fn delete(&mut self) -> DEL {
        let curr_line = self.curr_line_num();
        if self.prelines[curr_line].delete() {
            DEL::Yes
        } else if self.postlines.len() > 0 {
            let nline = self.postlines.last().unwrap().show();
            self.prelines[curr_line].move_end();
            self.postlines.pop();
            let linelen = self.linelen();
            for ch in nline.chars() { self.prelines[curr_line].insert(ch) }
            for _ in (linelen..self.linelen()) { self.move_left(); }
            DEL::NewLine(linelen)
        } else {
            DEL::No
        }
    }

    pub fn remove_empty_line(&mut self) -> bool {
        if self.linelen() == 0 && self.num_lines() > 1 {
            self.prelines.pop();
            if self.prelines.len() == 0 { 
                self.prelines.push(self.postlines.pop().unwrap()); 
                let curr_line = self.curr_line_num();
                self.prelines[curr_line].move_start();
                return false;
            } else { 
                let curr_line = self.curr_line_num();
                self.prelines[curr_line].move_start();        
                return true;
            }
        } 
        return false;
    }

    pub fn insert(&mut self, newchar: char) {
        let curr_line = self.curr_line_num();
        self.prelines[curr_line].insert(newchar);
    }

    pub fn insert_newline(&mut self) -> String {
        let mut newline = String::new();
        let curr_line = self.curr_line_num();
        let cline = &mut self.prelines[curr_line];
        for i in cline.get_post()+1..cline.get_len() {
            newline.push(cline.get_text(i)); 
        }

        cline.set_post(cline.get_len()-1);
        let prevline = cline.show();
        self.prelines.push(Line::init_with_line(newline));

        prevline
    }

    pub fn show_curr_line(&mut self) -> String {
        self.prelines[self.curr_line_num()].show()
    }

    pub fn show_curr_post_line(&mut self) -> String {
        let mut post_line = String::new();
        let cline = &self.prelines[self.curr_line_num()];
        for i in cline.get_post()+1..cline.get_len() { 
            post_line.push(self.prelines[self.curr_line_num()].get_text(i)); 
        }

        post_line
    }

    pub fn show(&self) -> String { // use to see internal state of editor
        let mut ed_state = String::new();
        ed_state.push_str(
            &format!("line: {}, cursor: {}", self.curr_line_num() + 1 as usize, self.cursor() + 1 as usize)
        );
        ed_state.push('\n');
        for (i,cline) in self.get_lines().iter().enumerate() {
            ed_state.push_str(&format!("{} | {}", i+1, cline.show()));
        }

        ed_state.to_string()
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {

        let path = Path::new(path);
        let mut file = File::create(path)?;

        for cline in self.get_lines() {
            cline.save(&mut file)?;
        }

        Ok(())
    }
}    
