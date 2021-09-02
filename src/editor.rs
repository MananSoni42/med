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

    pub fn show_header(&mut self, line: usize, cursor: usize) {
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
                    SEP, format!("({},{})", line, cursor), SEP, "--- Med v0.1 ---", SEP, "N", SEP,
                    twidth=TITLE_WIDTH, cwidth=CURSOR_WIDTH, cmwidth=COMMAND_WIDTH)
                )
        );
        self.term.execute(cursor::MoveToNextLine(1));
        self.term.execute(style::Print(String::from_utf8(vec![b'-'; cols as usize]).unwrap().white()));

        self.term.execute(cursor::RestorePosition);        
    }

    pub fn show(&mut self) {
        self.term.execute(terminal::Clear(terminal::ClearType::All));
        self.term.execute(cursor::MoveTo(0,2));
        self.show_header(0,0);
        for line in self.subed.lines.iter() {
            print!("{}", line.show());
            self.term.execute(cursor::MoveToNextLine(1));
        }

        self.term.execute(cursor::MoveTo(0,2));
    }

    pub fn start(&mut self) {
        self.show();
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
                        self.term.execute(cursor::MoveLeft(1));
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Right })) => {
                        self.term.execute(cursor::MoveRight(1));
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Up })) => {
                        //term.execute(cursor::MoveToPreviousLine(1));
                        self.term.execute(cursor::MoveUp(1));
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Down })) => {
                        //term.execute(cursor::MoveToNextLine(1));
                        self.term.execute(cursor::MoveDown(1));
                    }
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Enter })) => {
                        self.term.execute(cursor::MoveToNextLine(1));
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Backspace })) => {
                        self.term.execute(cursor::MoveLeft(1));
                        self.term.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
                    } 
                    Ok(Event::Key(KeyEvent{ modifiers: _keymod, code: KeyCode::Char(keych) })) => {
                        self.term.execute(style::Print(keych));        
                    }
                    Ok(Event::Resize(_,_)) => {
                        self.show();
                    }
                    Err(e) => {
                        // error handling
                    }
                    _ => {
                        // nothing for mouse and resize events
                    }
                }
        } else {
                // Timeout expired, no event for 1s
            }
        }    
    }
}

fn editor(term: &mut Write) {
}
