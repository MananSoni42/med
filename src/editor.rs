use std::io::{self, Write};
use std::time::Duration;
use std::path::Path;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers },
    style,
    terminal,
    ExecutableCommand, Result
};

pub mod subeditor;

pub struct Editor<'a> {
    pub term: &'a mut dyn Write,
    pub subed: subeditor::SubEditor,
    pub fname: &'a str,
    pub xscroll: usize,
    pub yscroll: usize
}

static FNAME_WIDTH: usize = 20; // even, more than 3
static ROW_OFFSET: usize = 2;
static COL_OFFSET: usize = 5 ; // odd, more than 3

static LARROW: &str = "◂";
static RARROW: &str = "▸";
static DARROW: &str = "▾";

impl Editor<'_> {

    pub fn init(&mut self) -> Result<()> {
        self.term.execute(terminal::EnterAlternateScreen)?;
        self.term.execute(terminal::DisableLineWrap)?;
        terminal::enable_raw_mode()?;    

        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;    
        self.term.execute(terminal::EnableLineWrap)?;
        self.term.execute(terminal::LeaveAlternateScreen)?;

        Ok(())
    }

    fn disp_name(&self) -> Result<String> {
        let filename =  Path::new(self.fname).file_name()
                        .ok_or(io::Error::new(io::ErrorKind::PermissionDenied, "Could not get file name"))?
                        .to_str()
                        .ok_or(io::Error::new(io::ErrorKind::PermissionDenied, "Could not get file name"))?;
        let fnamelen = filename.len();
        let start = fnamelen + 2 - FNAME_WIDTH/2;

        if fnamelen + 1 < FNAME_WIDTH { 
            Ok(format!("{}", filename) )
        } else { 
            Ok(format!( "{}...{}", &filename[0..FNAME_WIDTH/2], &filename[start..fnamelen])) 
        }
    }

    fn setxscroll(&mut self, x: usize) -> Result<()> {
        if self.xscroll != x {
            self.xscroll = x;
            self.show_content()?;    
        }

        Ok(())
    }

    fn setyscroll(&mut self, y: usize) -> Result<()> {
        if self.yscroll != y {
            self.yscroll = y;
            self.show_header()?;    
            self.show_content()?;
        }

        Ok(())
    }

    fn scroll_up(&mut self) -> Result<()> {
        self.yscroll -= 1;
        let (_,rows) = terminal::size().unwrap();
        let rows = rows as usize;
        self.term.execute(terminal::ScrollDown(1));
        self.show_header();

        self.term.execute(cursor::MoveTo(0,(rows-1) as u16));        
        self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
        self.term.execute(style::SetForegroundColor(style::Color::White))?;
        print!(" {:^lwidth$} ", DARROW, lwidth=COL_OFFSET-2);
        self.term.execute(style::ResetColor)?;        
        self.term.execute(cursor::MoveTo(0,ROW_OFFSET as u16));
        Ok(())
    }

    fn scroll_down(&mut self) -> Result<()> {
        self.yscroll += 1;
        let (_,rows) = terminal::size().unwrap();
        let rows = rows as usize;
        self.term.execute(terminal::ScrollUp(1));
        self.show_header();
        self.term.execute(cursor::MoveToNextLine(1));
        self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
        self.term.execute(style::SetForegroundColor(style::Color::White))?;
        if self.subed.curr_line_num() + 1 != self.subed.num_lines() {
            print!(" {:^lwidth$} ", DARROW, lwidth=COL_OFFSET-2);
            self.term.execute(style::ResetColor)?;        
        }
        self.term.execute(cursor::MoveToPreviousLine(1));    
        Ok(())
    }

    pub fn get_line<'a>(&self, cline: &'a str) -> (&'a str, &'a str, bool) {
        let (cols,_) = terminal::size().unwrap();
        let xind = if self.xscroll > 0 { LARROW } else { " " };

        if self.xscroll < cline.len() {
            if COL_OFFSET + cline.len() + 1 <= self.xscroll + cols as usize {
                (xind, &cline[self.xscroll..], false)
            } else if self.xscroll < cline.len() {
                (xind, &cline[self.xscroll..self.xscroll + cols as usize - COL_OFFSET - 1], true)
            } else {
                (" ", "", false)
            }
        } else {
            (" ", "", false)
        }

    }

    fn show_header(&mut self) -> Result<()> {
        self.term.execute(cursor::SavePosition)?;
        self.term.execute(style::SetForegroundColor(style::Color::White))?;

        self.term.execute(cursor::MoveTo(0,0))?;
        
        let (cols,_) = terminal::size()?;
        let title_width: usize = cols as usize - FNAME_WIDTH - 3;

        self.term.execute(cursor::MoveTo(0,0))?;
        print!( "{:^twidth$} | {:^fwidth$}", 
                " Med v0.1 ", self.disp_name()?,
                twidth=title_width, fwidth=FNAME_WIDTH
        );
        self.term.execute(cursor::MoveToNextLine(1))?;
        print!("{}", vec!['¯'; cols as usize].iter().collect::<String>());

        self.term.execute(style::ResetColor)?;
        self.term.execute(cursor::RestorePosition)?; 
        
        Ok(())
    }

    fn show_content(&mut self) -> Result<()> {
        self.term.execute(cursor::SavePosition)?;        
        self.term.execute(cursor::MoveTo(0, ROW_OFFSET as u16))?;
        let (cols, rows) = terminal::size()?;        
        for (i,line) in self.subed.get_lines().iter().skip(self.yscroll).take(rows as usize - ROW_OFFSET - 1).enumerate() {
            self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
            let cline = line.show();
            let (xind, cline, longline) = self.get_line(&cline); 
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            print!("{}{:^lwidth$} ", xind, i + self.yscroll + 1, lwidth=COL_OFFSET-2);
            self.term.execute(style::ResetColor)?;
            print!("{}", cline);
            if longline {
                self.term.execute(style::SetForegroundColor(style::Color::White))?;
                print!("{}", RARROW);
                self.term.execute(style::ResetColor)?;        
            }
            self.term.execute(cursor::MoveToNextLine(1))?;
        }
        let (_,rcursor) = cursor::position()?;
        if rcursor+1 == rows && self.subed.curr_line_num() + 1 != self.subed.num_lines(){
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            print!(" {:^lwidth$} ", DARROW, lwidth=COL_OFFSET-2);
            self.term.execute(style::ResetColor)?;        
            self.term.execute(cursor::MoveToPreviousLine(1))?;
        }
        self.term.execute(cursor::RestorePosition)?; 
        
        Ok(())
    }

    fn show_post_content(&mut self) -> Result<()> {
        self.term.execute(cursor::SavePosition)?;        
        let cnum = self.subed.curr_line_num();
        let (cols, rows) = terminal::size()?;        
        let (_,rcursor) = cursor::position()?;

        for (i,line) in self.subed.get_post_lines().iter().rev().enumerate() {
            if rcursor + i as u16 + 1 == rows { break; }
            self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
            let cline = line.show();
            let (xind, cline, longline) = self.get_line(&cline); 
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            let xind = if self.xscroll > 0 { "🞀" } else { " " };
            print!("{}{:^lwidth$} ", xind, i+self.yscroll+self.subed.num_lines_pre() + 1, lwidth=COL_OFFSET-2);
            self.term.execute(style::ResetColor)?;
            print!("{}", cline);
            if longline {
                self.term.execute(style::SetForegroundColor(style::Color::White))?;
                print!("{}", RARROW);
                self.term.execute(style::ResetColor)?;        
            }
            self.term.execute(cursor::MoveToNextLine(1))?;
        }
        let (_,rcursor) = cursor::position()?;
        if rcursor+1 == rows && self.subed.curr_line_num() + 1 != self.subed.num_lines() {
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            print!(" {:^lwidth$} ", DARROW, lwidth=COL_OFFSET-2);
            self.term.execute(style::ResetColor)?;        
            self.term.execute(cursor::MoveToPreviousLine(1))?;
        }
        self.term.execute(cursor::RestorePosition)?;
        
        Ok(())
    }

    fn show_line(&mut self, linenum: usize, line: &str) -> Result<()> {
        self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
        self.term.execute(cursor::MoveToColumn(0));
        let (xind, cline, longline) = self.get_line(line); 
        self.term.execute(style::SetForegroundColor(style::Color::White))?;
        print!("{}{:^lwidth$} ", xind, linenum, lwidth=COL_OFFSET-2);
        self.term.execute(style::ResetColor)?;
        print!("{}", cline);
        if longline {
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            print!("{}", RARROW);
            self.term.execute(style::ResetColor)?;        
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {

        self.init()?;

        self.term.execute(terminal::Clear(terminal::ClearType::All))?;
        self.show_content()?;
        self.show_header()?;
        self.term.execute(cursor::MoveTo(COL_OFFSET as u16, ROW_OFFSET as u16))?;

        loop {
            // Wait up to 1s for another event
            if poll(Duration::from_millis(1_000))? {
                // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
                match read() {
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Left })) => {
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_start();
                            self.setxscroll(0);
                            self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + 1))?;
                        } else if self.subed.move_left() {
                            let (cpos,_) = cursor::position()?;
                            if cpos == COL_OFFSET as u16 && self.xscroll > 0 {
                                self.setxscroll(self.xscroll-1)?;
                            } else {
                                self.term.execute(cursor::MoveLeft(1))?;
                            }
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Right })) => {
                        let (cols,_) = terminal::size()?;        
                        let cols = cols as usize;
                        if keymod == KeyModifiers::CONTROL {
                            let llen = self.subed.linelen();
                            self.subed.move_end();
                            if COL_OFFSET + llen + 1 > cols {
                                self.setxscroll(llen + COL_OFFSET + 1 - cols);
                            }
                            self.term.execute(cursor::MoveToColumn((COL_OFFSET +llen + 1) as u16))?;
                        } else if self.subed.move_right() {
                            let (cpos,_) = cursor::position()?;
                            if cpos == (cols-2) as u16 {
                                self.setxscroll(self.xscroll+1);
                            } else {
                                self.term.execute(cursor::MoveRight(1))?;
                            }
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Up })) => {
                        let (_,rows) = terminal::size()?;
                        let rows = rows as usize;
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_first();
                            self.term.execute(cursor::MoveTo(COL_OFFSET as u16, ROW_OFFSET as u16))?;
                            self.setyscroll(0);
                        } else if self.subed.move_up() {
                            let (_,cpos) = cursor::position()?;   
                            let cpos = cpos as usize;  
                            if cpos == ROW_OFFSET && self.yscroll > 0 {
                                self.scroll_up();
                                self.show_line(self.subed.curr_line_num() + 1, &self.subed.curr_line())?;
                            } else {
                                self.term.execute(cursor::MoveToPreviousLine(1))?;
                            }                     
                            self.term.execute(cursor::MoveToColumn((COL_OFFSET + self.subed.cursor() + 1) as u16))?;                                
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Down })) => {
                        let (_,rows) = terminal::size()?;
                        let rows = rows as usize;
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_last();
                            self.term.execute(cursor::MoveTo(COL_OFFSET as u16, ROW_OFFSET as u16 + self.subed.num_lines() as u16 - 1))?;
                            let num_lines = self.subed.num_lines();
                            if ROW_OFFSET + num_lines + 1 > rows { 
                                self.setyscroll(ROW_OFFSET + num_lines + 1 - rows);
                                self.term.execute(cursor::MoveTo(0, (rows-2) as u16))?;
                            } 
                        } else if self.subed.move_down() {
                            let (_,cpos) = cursor::position()?; 
                            let cpos = cpos as usize;  
                            if cpos == rows - 2 {
                                self.scroll_down();
                                self.show_line(self.subed.curr_line_num() + 1, &self.subed.curr_line())?;
                            } else {
                                self.term.execute(cursor::MoveToNextLine(1))?;
                            }
                        }
                        self.term.execute(cursor::MoveToColumn((COL_OFFSET + self.subed.cursor() + 1) as u16))?;                            
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Enter })) => {
                        let prevline = self.subed.insert_newline();
                        self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
                        self.term.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;

                        let (_,cpos) = cursor::position()?;   
                        let (_,rows) = terminal::size()?;

                        self.show_line(self.subed.curr_line_num(), &prevline);

                        if cpos + 1 < rows {
                            self.term.execute(cursor::MoveToNextLine(1))?;

                            self.show_line(self.subed.curr_line_num()+1, &self.subed.curr_line());

                            if cpos + 2 < rows {
                                self.term.execute(cursor::MoveToNextLine(1))?;    
                                self.show_post_content()?;
                                self.term.execute(cursor::MoveToPreviousLine(1))?;
                            } 
                        }
                        self.term.execute(cursor::MoveToColumn((COL_OFFSET + self.subed.cursor() + 1) as u16))?;
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Backspace })) => {
                        match self.subed.backspace() {
                            subeditor::DEL::NewLine(newcursor) => {
                                self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
                                self.term.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                                
                                self.term.execute(cursor::MoveToPreviousLine(1))?; 
                                self.show_line(self.subed.curr_line_num()+1, &self.subed.curr_line());
                                
                                self.term.execute(cursor::MoveToNextLine(1))?;
                                self.show_post_content()?;
                                self.term.execute(cursor::MoveToPreviousLine(1))?;
                                let (cols,_) = terminal::size()?;   
                                if COL_OFFSET + newcursor >= cols as usize {
                                    self.term.execute(cursor::MoveToColumn(cols-1))?;
                                } else {
                                    self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + newcursor as u16 + 1))?;
                                }
                            } 
                            subeditor::DEL::Yes => {
                                let cline = self.subed.show_curr_line();
                                let cpline = self.subed.show_curr_post_line();
                                let (cols,_) = terminal::size()?;   
                                let (cpos,_) = cursor::position()?;

                                if self.xscroll > 0 && cpos as usize == COL_OFFSET {
                                    self.setxscroll(self.xscroll-1);
                                } else {
                                    self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                                    self.term.execute(cursor::MoveLeft(1))?;
                                    self.term.execute(cursor::SavePosition)?;                    
                                    print!("{}", cpline);
                                    if cpos as usize + cpline.len() >= cols as usize {
                                        self.term.execute(cursor::MoveToColumn(cols as u16))?;    
                                        self.term.execute(style::SetForegroundColor(style::Color::White))?;
                                        print!("{}", RARROW);    
                                        self.term.execute(style::ResetColor)?;        
                                    }
                                    self.term.execute(cursor::RestorePosition)?;                
                                }
                            }
                            subeditor::DEL::No => { }
                        }
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Delete })) => {
                        match self.subed.delete() {
                            subeditor::DEL::NewLine(newcursor) => {
                                self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
                                self.term.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                                self.show_line(self.subed.curr_line_num()+1, &self.subed.curr_line())?;
                                self.term.execute(cursor::MoveToNextLine(1))?;

                                self.show_post_content()?;
                                self.term.execute(cursor::MoveToPreviousLine(1))?;
                                self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + newcursor as u16 + 1))?;
                            } 
                            subeditor::DEL::Yes => {
                                let cline = self.subed.show_curr_line();
                                let cpline = self.subed.show_curr_post_line();
                                let (cols,_) = terminal::size()?;   
                                let (cpos,_) = cursor::position()?;
                                self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;

                                self.term.execute(cursor::SavePosition)?;                
                                print!("{}", self.subed.show_curr_post_line());

                                if cpos as usize + cpline.len() >= cols as usize {
                                    self.term.execute(cursor::MoveToColumn(cols as u16))?;    
                                    self.term.execute(style::SetForegroundColor(style::Color::White))?;
                                    print!("{}", RARROW);    
                                    self.term.execute(style::ResetColor)?;        
                                }
                                self.term.execute(cursor::RestorePosition)?;

                            }
                            subeditor::DEL::No => { }
                        }
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Char(keych) })) => {
                        if keymod == KeyModifiers::CONTROL && (keych == 'q' || keych == 'Q') {
                            break;
                        } else if keymod == KeyModifiers::CONTROL && (keych == 's' || keych == 'S') {
                            self.subed.save(self.fname)?;
                            break;
                        } else {
                            self.subed.insert(keych);
                            let (cols,_) = terminal::size()?;   
                            let cols = cols as usize;  
                            let (cpos,_) = cursor::position()?;
                            let cpos = cpos as usize;
                            self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;

                            print!("{}", keych);
                            self.term.execute(cursor::SavePosition)?;

                            if COL_OFFSET + self.subed.linelen() + 1 < cols {
                                print!("{}", self.subed.show_curr_post_line());
                                self.term.execute(cursor::RestorePosition)?;        
                            } else {
                                self.term.execute(cursor::MoveToColumn(cols as u16))?;    
                                self.term.execute(style::SetForegroundColor(style::Color::White))?;
                                print!("{}", RARROW);
                                self.term.execute(style::ResetColor)?;        
                                if cpos + 1 + 1 > cols {
                                    self.setxscroll(self.xscroll+1);
                                    self.term.execute(cursor::RestorePosition)?;    
                                    self.term.execute(cursor::MoveLeft(1));
                                } else {
                                    self.term.execute(cursor::RestorePosition)?;        
                                    self.term.execute(cursor::SavePosition)?;
                                    print!("{}", &self.subed.show_curr_post_line()[..cols-2-cpos]);
                                    self.term.execute(cursor::RestorePosition)?;        
                                }
                            }

                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::F(5) })) => {
                        self.term.execute(terminal::Clear(terminal::ClearType::All))?;
                        self.xscroll = 0;
                        self.yscroll = 0;
                        self.subed.move_first();
                        self.show_content()?;
                        self.show_header()?;
                        self.term.execute(cursor::MoveTo(COL_OFFSET as u16, ROW_OFFSET as u16))?;
                    }
                    Ok(Event::Resize(_,_)) => {
                        self.show_content()?;
                        self.show_header()?;                
                    }
                    Err(_) => {
                        // error handling
                    }
                    _ => {
                        // nothing for mouse events, other Fn keys
                    }
                }
        } else {
                // Timeout expired, no event for 1s
            }
        }   
        
        Ok(())
    }
}
