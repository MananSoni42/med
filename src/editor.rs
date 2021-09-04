use std::io::{BufRead, Write};
use std::time::Duration;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers },
    style::{self,Stylize, StyledContent},
    terminal,
    Command, ExecutableCommand, Result
};

pub mod subeditor;

pub struct Editor<'a> {
    pub term: &'a mut Write,
    pub subed: subeditor::SubEditor,
}

impl Editor<'_> {

    pub fn init(&mut self) {
        self.term.execute(terminal::EnterAlternateScreen).unwrap();
        terminal::enable_raw_mode().unwrap();    
    }

    pub fn exit(&mut self) {
        self.term.execute(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();    
    }

    pub fn change_loc(&mut self) {
        self.term.execute(cursor::SavePosition);
        self.term.execute(cursor::MoveTo(0,0));
        let CURSOR_WIDTH: usize = 10;
        let SEP: StyledContent<&str> = "|".white();             
        self.term.execute(style::Print(
            format!("{} {:^cwidth$} {}", 
                    SEP, format!("({},{})", self.subed.curr_line(), self.subed.cursor()+1 as usize), 
                    SEP, cwidth=CURSOR_WIDTH)
            )
        );
        self.term.execute(cursor::RestorePosition);        
    }

    pub fn show_header(&mut self) {
        self.term.execute(cursor::SavePosition);
        self.term.execute(cursor::MoveTo(0,0));
        
        let (cols,rows) = terminal::size().unwrap();
        let SEP: StyledContent<&str> = "|".white();             
        let CURSOR_WIDTH: usize = 10;
        let COMMAND_WIDTH: usize = 10;
        let TITLE_WIDTH: usize = cols as usize - CURSOR_WIDTH - COMMAND_WIDTH - 10;

        self.term.execute(cursor::MoveTo(0,0));
        self.term.execute(style::Print(
            format!("{} {:^cwidth$} {} {:^twidth$} {} {:^cmwidth$} {}", 
                    SEP, format!("({},{})", self.subed.curr_line(), self.subed.cursor()+1 as usize), 
                    SEP, "--- Med v0.1 ---", SEP, "N", SEP,
                    twidth=TITLE_WIDTH, cwidth=CURSOR_WIDTH, cmwidth=COMMAND_WIDTH)
                )
        );
        self.term.execute(cursor::MoveToNextLine(1));
        self.term.execute(style::Print(String::from_utf8(vec![b'-'; cols as usize]).unwrap().white()));

        self.term.execute(cursor::RestorePosition);        
    }

    pub fn show_content(&mut self) {
        self.term.execute(cursor::SavePosition);        
        self.term.execute(cursor::MoveTo(0,2));
        for line in self.subed.get_lines() {
            print!("{}", line.show());
            self.term.execute(cursor::MoveToNextLine(1));
        }
        self.term.execute(cursor::RestorePosition);                
    }

    pub fn start(&mut self) {

        self.term.execute(terminal::Clear(terminal::ClearType::All));
        self.show_content();
        self.show_header();
        self.term.execute(cursor::MoveTo(0,2));

        let cursor_pos = cursor::position();
        loop {
            // Wait up to 1s for another event
            if poll(Duration::from_millis(1_000)).unwrap() {
                // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
                match read() {
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Esc })) => {
                        break;
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Left })) => {
                        if self.subed.move_left() {
                            self.change_loc();
                            self.term.execute(cursor::MoveLeft(1));
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Right })) => {
                        if self.subed.move_right() {
                            self.change_loc();
                            self.term.execute(cursor::MoveRight(1));
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Up })) => {
                        if self.subed.move_up() {
                            self.change_loc();
                            let (ccol,crow) = cursor::position().unwrap();
                            self.term.execute(cursor::MoveTo(self.subed.cursor() as u16, crow - 1 as u16));
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Down })) => {
                        if self.subed.move_down() {
                            self.change_loc();
                            let (ccol,crow) = cursor::position().unwrap();
                            self.term.execute(cursor::MoveTo(self.subed.cursor() as u16, crow + 1 as u16));
                        }
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Enter })) => {
                        self.subed.insert_newline();
                        self.subed.move_up();
                        self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
                        self.term.execute(cursor::MoveToColumn(0));
                        self.term.execute(style::Print(self.subed.show_curr_post_line()));

                        self.term.execute(cursor::MoveToNextLine(1));
                        self.subed.move_down();
                        self.term.execute(style::Print(self.subed.show_curr_post_line()));
                        self.term.execute(cursor::MoveToColumn(0));
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Backspace })) => {
                        if self.subed.backspace() {
                            self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
                            self.term.execute(cursor::MoveLeft(1));
                            self.term.execute(cursor::SavePosition);                
                            self.term.execute(style::Print(self.subed.show_curr_post_line()));
                            self.term.execute(cursor::RestorePosition);                
                        }
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Char(keych) })) => {
                        self.subed.insert(keych);
                        self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
                        self.term.execute(style::Print(keych));
                        self.term.execute(cursor::SavePosition);                
                        self.term.execute(style::Print(self.subed.show_curr_post_line()));
                        self.term.execute(cursor::RestorePosition);                
                        //self.show_content();
                }
                    Ok(Event::Resize(_,_)) => {
                        self.show_content();
                        self.show_header();
                    }
                    Err(e) => {
                        // error handling
                    }
                    _ => {
                        // nothing for mouse events
                    }
                }
        } else {
                // Timeout expired, no event for 1s
            }
        }    
    }
}
