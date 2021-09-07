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
    pub fname: &'a str
}

static FNAME_WIDTH: usize = 20; // even and more than 3
static COMMAND_WIDTH: usize = 3;
static ROW_OFFSET: usize = 2;
static COL_OFFSET: usize = 4 ; // even

impl Editor<'_> {

    pub fn init(&mut self) -> Result<()> {
        self.term.execute(terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;    

        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.term.execute(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;    

        Ok(())
    }

    pub fn disp_name(&self) -> Result<String> {
        let filename =  Path::new(self.fname).file_name()
                        .ok_or(io::Error::new(io::ErrorKind::PermissionDenied, "Could not get file name"))?
                        .to_str()
                        .ok_or(io::Error::new(io::ErrorKind::PermissionDenied, "Could not get file name"))?;
        let fnamelen = filename.len();
        let start = fnamelen + 2 - FNAME_WIDTH/2;

        if fnamelen + 1 < FNAME_WIDTH { 
            Ok(format!("{}{}", filename, if self.subed.is_changed() {"*"} else { " " }) )
        } else { Ok(format!( "{}...{}{}", 
                        &filename[0..FNAME_WIDTH/2], 
                        &filename[start..fnamelen], 
                        if self.subed.is_changed() {"*"} else { " " })) }
    }

    pub fn show_header(&mut self) -> Result<()> {
        self.term.execute(cursor::SavePosition)?;
        self.term.execute(style::SetForegroundColor(style::Color::White))?;

        self.term.execute(cursor::MoveTo(0,0))?;
        
        let (cols,_) = terminal::size()?;
        let title_width: usize = cols as usize - COMMAND_WIDTH - FNAME_WIDTH - 8;

        self.term.execute(cursor::MoveTo(0,0))?;
        print!(" {:^fwidth$} | {:^twidth$} | {:^cmwidth$} ", 
                self.disp_name()?, " Med v0.1 ", "N",
                twidth=title_width, fwidth=FNAME_WIDTH, cmwidth=COMMAND_WIDTH
        );
        self.term.execute(cursor::MoveToNextLine(1))?;
        print!("{}", vec!['¯'; cols as usize].iter().collect::<String>());

        self.term.execute(style::ResetColor)?;
        self.term.execute(cursor::RestorePosition)?; 
        
        Ok(())
    }

    pub fn show_content(&mut self) -> Result<()> {
        self.term.execute(cursor::SavePosition)?;        
        self.term.execute(cursor::MoveTo(0, ROW_OFFSET as u16))?;
        for (i,line) in self.subed.get_lines().iter().enumerate() {
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            print!("{:^lwidth$} ", i+1, lwidth=COL_OFFSET-1);
            self.term.execute(style::ResetColor)?;
            print!("{}",line.show());
            self.term.execute(cursor::MoveToNextLine(1))?;
        }
        self.term.execute(cursor::RestorePosition)?; 
        
        Ok(())
    }

    pub fn show_post_content(&mut self) -> Result<()> {
        self.term.execute(cursor::SavePosition)?;        
        let cnum = self.subed.curr_line_num();
        for (i,line) in self.subed.get_post_lines().iter().rev().enumerate() {
            self.term.execute(style::SetForegroundColor(style::Color::White))?;
            print!("{:^lwidth$} ", i + 2 + cnum, lwidth=COL_OFFSET-1);
            self.term.execute(style::ResetColor)?;
            print!("{}",line.show());
            self.term.execute(cursor::MoveToNextLine(1))?;
        }
        self.term.execute(cursor::RestorePosition)?;
        
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
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Esc })) => {
                        break;
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::End })) => {
                        self.subed.save(self.fname)?;
                        break;
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Left })) => {
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_start();
                            self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + 1))?;
                        } else if self.subed.move_left() {
                            self.term.execute(cursor::MoveLeft(1))?;
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Right })) => {
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_end();
                            self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + self.subed.linelen() as u16 + 1))?;
                        } else if self.subed.move_right() {
                            self.term.execute(cursor::MoveRight(1))?;
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Up })) => {
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_first();
                            self.term.execute(cursor::MoveTo(COL_OFFSET as u16, ROW_OFFSET as u16))?;
                        } else if self.subed.move_up() {
                            self.term.execute(cursor::MoveToPreviousLine(1))?;
                            self.term.execute(cursor::MoveToColumn((COL_OFFSET + self.subed.cursor() + 1) as u16))?;                            
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Down })) => {
                        if keymod == KeyModifiers::CONTROL {
                            self.subed.move_last();
                            self.term.execute(cursor::MoveTo(COL_OFFSET as u16, ROW_OFFSET as u16 + self.subed.num_lines() as u16 - 1))?;
                        } else if self.subed.move_down() {
                            self.term.execute(cursor::MoveToNextLine(1))?;
                            self.term.execute(cursor::MoveToColumn((COL_OFFSET + self.subed.cursor() + 1) as u16))?;                            
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Enter })) => {
                        let prevline = self.subed.insert_newline();
                        self.term.execute(terminal::Clear(terminal::ClearType::CurrentLine))?;
                        self.term.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                        
                        self.term.execute(cursor::MoveToColumn(0))?;
                        self.term.execute(style::SetForegroundColor(style::Color::White))?;
                        print!("{:^lwidth$} ", self.subed.curr_line_num(), lwidth=COL_OFFSET-1);
                        self.term.execute(style::ResetColor)?;
                        print!("{}", prevline);
                        self.term.execute(cursor::MoveToNextLine(1))?;

                        self.term.execute(style::SetForegroundColor(style::Color::White))?;
                        print!("{:^lwidth$} ", self.subed.curr_line_num()+1, lwidth=COL_OFFSET-1);
                        self.term.execute(style::ResetColor)?;
                        print!("{}", self.subed.curr_line());

                        self.term.execute(cursor::MoveToNextLine(1))?;
                        self.show_post_content()?;
                        self.term.execute(cursor::MoveToPreviousLine(1))?;
                        self.term.execute(cursor::MoveToColumn((COL_OFFSET + self.subed.cursor() + 1) as u16))?;                            
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Backspace })) => {
                        if self.subed.linelen() == 0 {
                            if self.subed.delete_empty_line() { 
                                self.term.execute(cursor::MoveToPreviousLine(1))?; 
                                self.term.execute(style::SetForegroundColor(style::Color::White))?;
                                print!("{:^lwidth$} ", self.subed.curr_line_num()+1, lwidth=COL_OFFSET-1);
                                self.term.execute(style::ResetColor)?;        
                            }
                            self.term.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                            print!("{}", self.subed.curr_line());
                            self.term.execute(cursor::MoveToNextLine(1))?;
                            self.show_post_content()?;
                            self.term.execute(cursor::MoveToPreviousLine(1))?;
                            self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + 1))?;
                        } else if self.subed.backspace() {
                            self.term.execute(cursor::MoveLeft(1))?;
                            self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                            self.term.execute(cursor::SavePosition)?;                
                            print!("{}", self.subed.show_curr_post_line());
                            self.term.execute(cursor::RestorePosition)?;                
                        }
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Delete })) => {
                        if self.subed.linelen() == 0 {
                            if self.subed.delete_empty_line() { 
                                self.term.execute(cursor::MoveToPreviousLine(1))?; 
                                self.term.execute(style::SetForegroundColor(style::Color::White))?;
                                print!("{:^lwidth$} ", self.subed.curr_line_num()+1, lwidth=COL_OFFSET-1);
                                self.term.execute(style::ResetColor)?;        
                            }
                            self.term.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;
                            print!("{}", self.subed.curr_line());
                            self.term.execute(cursor::MoveToNextLine(1))?;
                            self.show_post_content()?;
                            self.term.execute(cursor::MoveToPreviousLine(1))?;
                            self.term.execute(cursor::MoveToColumn(COL_OFFSET as u16 + 1))?;
                        } else if self.subed.delete() {
                            self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                            self.term.execute(cursor::SavePosition)?;                
                            print!("{}", self.subed.show_curr_post_line());
                            self.term.execute(cursor::RestorePosition)?;                
                        }
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Char(keych) })) => {
                        self.subed.insert(keych);
                        self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine))?;
                        print!("{}", keych);
                        self.term.execute(cursor::SavePosition)?;                
                        print!("{}", self.subed.show_curr_post_line());
                        self.term.execute(cursor::RestorePosition)?;                
                }
                    Ok(Event::Resize(_,_)) => {
                        self.show_content()?;
                    }
                    Err(_) => {
                        // error handling
                    }
                    _ => {
                        // nothing for mouse events, F keys
                    }
                }
        } else {
                // Timeout expired, no event for 1s
            }
        }   
        
        Ok(())
    }
}
