use std::io::{self, BufRead, Write, stdout};
use std::time::Duration;
use crossterm::{
    cursor::{self,position},
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers },
    execute, queue, 
    style::{self,Stylize, StyledContent},
    terminal,
    Command, ExecutableCommand, Result
};

static CURSOR_WIDTH: usize = 10;
static COMMAND_WIDTH: usize = 10;

fn editor_input(term: &mut Write) {
    let SEP: StyledContent<&str> = "|".white();

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

pub fn editor_test() -> Result<()> {
    let SEP: StyledContent<&str> = "|".white();
    let (cols,rows) = terminal::size().unwrap();
    let mut term = stdout();
    let TITLE_WIDTH: usize = cols as usize - CURSOR_WIDTH - COMMAND_WIDTH - 10;

    term.execute(terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    term.execute(cursor::MoveTo(0,0));
    term.execute(terminal::Clear(terminal::ClearType::All));
    term.execute(style::Print(
        format!("{} {:^cwidth$} {} {:^twidth$} {} {:^cmwidth$} {}", 
                SEP, "(2,21)", SEP, "--- Med v0.1 ---", SEP, "N", SEP,
                twidth=TITLE_WIDTH, cwidth=CURSOR_WIDTH, cmwidth=COMMAND_WIDTH)
            )
    );
    term.execute(cursor::MoveToNextLine(1));
    term.execute(style::Print(String::from_utf8(vec![b'-'; cols as usize]).unwrap().white()));
    term.execute(cursor::MoveToNextLine(1));
    editor_input(&mut term);

    terminal::disable_raw_mode()?;
    term.execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}
