#[derive(Debug)]
pub struct Line {
    pre: usize,
    post: usize,
    text: Vec<char>
}

impl Line {

    pub fn init() -> Line {
        Line {
            pre: 0,
            post: 32-1,
            text: vec!['\0'; 32]
        }
    }

    pub fn len(&self) -> usize {
        self.pre + self.text.len() - 1 - self.post
    }

    pub fn cursor(&self) -> usize {
        self.pre
    }

    pub fn get_pre(&self) -> usize {
        self.pre
    }

    pub fn get_post(&self) -> usize {
        self.post
    }

    pub fn set_post(&mut self, newpost: usize) {
        self.post = newpost;
    }

    pub fn get_len(&self) -> usize {
        self.text.len()
    }

    pub fn get_text(&self, i: usize) -> char {
        self.text[i]
    }

    pub fn init_with_line(newline: String) -> Line {
        let mut len = 32;
        while len < newline.len() { len*= 2; }
        let mut text: Vec<char> = vec!['\0'; len];
        let offset = len - newline.len();
        for (i,ch) in newline.chars().enumerate() {
            text[offset+i] = ch;
        }

        Line {
            pre: 0,
            post: offset-1,
            text: text
        }
    }

    pub fn adjust_buffer(&mut self) {
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

    pub fn insert(&mut self, newchar: char) {
        self.adjust_buffer();
        self.text[self.pre] = newchar;
        self.pre += 1;
    }

    pub fn backspace(&mut self) -> bool {
        if self.pre > 0 {
            self.pre -= 1;
            return true;
        } return false;
    }

    pub fn move_l(&mut self) -> bool {
        if self.pre > 0 {
            self.pre -= 1;
            self.text[self.post] = self.text[self.pre];
            self.post -= 1;
            return true;
        }
        return false;
    }

    pub fn move_r(&mut self) -> bool {
        if self.post + 1 < self.text.len() {
            self.post += 1;
            self.text[self.pre] = self.text[self.post];
            self.pre += 1;
            return true;
        }
        return false;
    }

    pub fn move_start(&mut self) {
        while self.pre > 0 {
            self.move_l();
        }
    }

    pub fn move_end(&mut self) {
        while self.post + 1 != self.text.len() {
            self.move_r();
        }
    }

    pub fn show(&self) -> String {
        let mut line = String::new();
        for i in 0..self.pre { line.push(self.text[i]); }
        for i in self.post+1..self.text.len() { line.push(self.text[i]); }
        
        line
    }
}    
