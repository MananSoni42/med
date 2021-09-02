use std::io::{BufRead, Write};
use std::time::Duration;
use crossterm::{
    cursor::{self,position},
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers },
    execute, queue, 
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
        self.term.execute(terminal::Clear(terminal::ClearType::All));
    }

    pub fn exit(&mut self) {
        self.term.execute(terminal::LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();    
    }

    pub fn show(&mut self) {

        let (cols,rows) = terminal::size().unwrap();

        let SEP: StyledContent<&str> = "|".white();             
        let CURSOR_WIDTH: usize = 10;
        let COMMAND_WIDTH: usize = 10;
        let TITLE_WIDTH: usize = cols as usize - CURSOR_WIDTH - COMMAND_WIDTH - 10;

        self.term.execute(cursor::MoveTo(0,0));
        self.term.execute(style::Print(
            format!("{} {:^cwidth$} {} {:^twidth$} {} {:^cmwidth$} {}", 
                    SEP, "(0,0)", SEP, "--- Med v0.1 ---", SEP, "N", SEP,
                    twidth=TITLE_WIDTH, cwidth=CURSOR_WIDTH, cmwidth=COMMAND_WIDTH)
                )
        );
        self.term.execute(cursor::MoveToNextLine(1));
        self.term.execute(style::Print(String::from_utf8(vec![b'-'; cols as usize]).unwrap().white()));
        self.term.execute(cursor::MoveToNextLine(1));
        
        for line in self.subed.lines.iter() {
            print!("{}", line.show());
            self.term.execute(cursor::MoveToNextLine(1));
        }

        self.term.execute(cursor::MoveTo(0,2));
    }
}

fn editor(term: &mut Write) {
    loop {
        // Wait up to 1s for another event
        if poll(Duration::from_millis(1_000)).unwrap() {
            // It's guaranteed that read() wont block if `poll` returns `Ok(true)`
            match read() {
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Esc })) => {
                    break;
                }
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Left })) => {
                    term.execute(cursor::MoveLeft(1));
                }
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Right })) => {
                        term.execute(cursor::MoveRight(1));
                }
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Up })) => {
                    //term.execute(cursor::MoveToPreviousLine(1));
                    term.execute(cursor::MoveUp(1));
                }
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Down })) => {
                    //term.execute(cursor::MoveToNextLine(1));
                    term.execute(cursor::MoveDown(1));
                }
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Enter })) => {
                    term.execute(cursor::MoveToNextLine(1));
                } 
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Backspace })) => {
                    term.execute(cursor::MoveLeft(1));
                    term.execute(terminal::Clear(terminal::ClearType::UntilNewLine));
                } 
                Ok(Event::Key(KeyEvent{ modifiers: keymod, code: KeyCode::Char(keych) })) => {
                            term.execute(style::Print(keych));        
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
