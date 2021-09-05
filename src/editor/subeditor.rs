use deepmesa::lists::{LinkedList, linkedlist::Node};
use std::fs::File;
use std::path::Path;
use std::io::{self,BufRead,Write};
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
    curr_line_num: usize,
    curr_line: Option<Node<Line>>,
    lines: LinkedList::<Line>
}

impl SubEditor {
    pub fn open(path: &str) -> SubEditor {

        let mut subed = SubEditor {
            curr_line_num: 0,
            curr_line: None,
            lines: LinkedList::<Line>::new()
        };

        let path = Path::new(path);

        let lines = match read_lines(&path) {
            Err(why) => panic!("couldn't open {}: {}", path.display(), why),
            Ok(lines) => lines,            
        };

        for line in lines {
            match line {
                Ok(content) => { 
                    subed.curr_line_num += 1;
                    subed.curr_line = Some(subed.lines.push_tail(Line::init_with_line(content)));
                },
                Err(_) => panic!("Could not read Line {} in file {}", subed.curr_line_num, path.display()),
            }
        }

        subed.move_first();

        return subed;
    }

    pub fn curr_line(&self) -> usize {
        self.curr_line_num
    }

    pub fn get_lines(&self) -> deepmesa::lists::linkedlist::Iter<Line> {
        self.lines.iter()
    }

    pub fn linelen(&self) -> usize {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node(&lineref).unwrap();
        
        line.len()
    }

    pub fn cursor(&self) -> usize {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node(&lineref).unwrap();

        line.cursor()
    }

    pub fn move_left(&mut self) -> bool {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.move_l()
    }

    pub fn move_right(&mut self) -> bool {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.move_r()
    }

    pub fn move_up(&mut self) -> bool {
        let lineref = self.curr_line.clone().unwrap();
        let final_cursor_pos = self.lines.node(&lineref).unwrap().cursor();

        match self.lines.prev_node(&lineref) {
            Some(v) => { 
                let mut curr_line = self.lines.node_mut(&v).unwrap();
                if final_cursor_pos > curr_line.len() { 
                    curr_line.move_end(); 
                } else if curr_line.cursor() > final_cursor_pos {
                    while curr_line.cursor() > final_cursor_pos { curr_line.move_l(); }
                } else {
                    while curr_line.cursor() < final_cursor_pos { curr_line.move_r(); }
                }

                self.curr_line = Some(v); 
                self.curr_line_num -= 1;

                true 
            },
            None => false 
        }
    }
 
    pub fn move_down(&mut self) -> bool {
        let lineref = self.curr_line.clone().unwrap();
        let final_cursor_pos = self.lines.node(&lineref).unwrap().cursor();

        match self.lines.next_node(&lineref) {
            Some(v) => { 
                let mut curr_line = self.lines.node_mut(&v).unwrap();
                if final_cursor_pos > curr_line.len() { 
                    curr_line.move_end(); 
                } else {
                    while curr_line.cursor() > final_cursor_pos { curr_line.move_l(); }
                    while curr_line.cursor() < final_cursor_pos { curr_line.move_r(); }
                }

                self.curr_line = Some(v); 
                self.curr_line_num += 1;
                
                true 
            },
            None => false 
        }
    }

    pub fn move_start(&mut self) {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.move_start()
    }

    pub fn move_end(&mut self) {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.move_end()
    }

    pub fn move_first(&mut self) {
        self.curr_line_num = 1;
        self.curr_line = Some(self.lines.head_node().unwrap());

        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.move_start();
    }

    pub fn move_last(&mut self) {
        self.curr_line_num = self.lines.len();
        self.curr_line = Some(self.lines.tail_node().unwrap());

        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.move_start();
    }

    pub fn backspace(&mut self) -> bool {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.backspace()
    }

    pub fn backspace_line(&mut self) -> bool {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();    

        if line.len() == 0 {
            if self.move_down() {
                let lineref = self.curr_line.clone().unwrap();
                self.lines.pop_prev(&lineref).unwrap();           
                self.curr_line_num -= 1;
                return false;

            } else if self.move_up() {
                let lineref = self.curr_line.clone().unwrap();
                self.lines.pop_next(&lineref).unwrap();                   
                return true;
            }
            else {
                panic!("Anakin start panikin, no lines")
            }

        }

        false
    }

    pub fn insert(&mut self, newchar: char) {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        line.insert(newchar);
    }

    pub fn insert_newline(&mut self) {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node_mut(&lineref).unwrap();
        let mut newline = String::new();
        for i in line.get_post()+1..line.get_len() { newline.push(line.get_text(i)); }

        line.set_post(line.get_len()-1);
        self.curr_line = Some(self.lines.push_next(&lineref, Line::init_with_line(newline)).unwrap());
        self.curr_line_num += 1;
    }

    pub fn show_curr_line(&mut self) -> String {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node(&lineref).unwrap();
        line.show()
    }

    pub fn show_curr_post_line(&mut self) -> String {
        let lineref = self.curr_line.clone().unwrap();
        let line = self.lines.node(&lineref).unwrap();
        
        let mut post_line = String::new();
        for i in line.get_post()+1..line.get_len() { post_line.push(line.get_text(i)); }

        post_line
    }

    pub fn show(&self) {
        println!("line: {}, cursor: {}", self.curr_line_num, self.cursor());
        for (i,line) in self.get_lines().enumerate() {
            println!("{} | {}", i+1, line.show());
        }
    }

    pub fn save(&self, path: &str) {

        let path = Path::new(path);
        let mut file = File::create(path).unwrap();

        for (i,line) in self.get_lines().enumerate() {
            line.save(&mut file);
        }
    }
}    
