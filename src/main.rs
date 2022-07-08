use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    queue, style,
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
};
use std::{
    io::{self, stdout, Write},
    process,
    time::Duration,
};

use rs_snake::{Snake, Food};

pub const DEFAULT_PARTS: [char; 5] = ['⮝', '⮟', '⮜', '➤', '*'];

fn main() {
    if let Err(err) = with_blank_screen(splash) {
        eprintln!("unable to print screen: {}", err);
        process::exit(1);
    }
}

type Print = fn() -> io::Result<Screen>;
enum Screen {
    Next(Print),
    Exit,
}

fn splash() -> io::Result<Screen> {
    let (col, row) = terminal::size()?;
    queue!(
        stdout(),
        cursor::MoveTo(col / 2 - 9, row / 2 - 1),
        style::Print("Snakes are Snakeing!"),
        cursor::MoveDown(1),
        cursor::MoveLeft(9),
        cursor::SavePosition,
        style::Print("Press enter to start...."),
        cursor::RestorePosition,
        cursor::MoveDown(1),
        style::Print("q or ESC to exit.")
    )?;
    stdout().flush()?;
    loop {
        match event::read()? {
            Event::Key(key) if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') => {
                break Ok(Screen::Exit)
            }
            Event::Key(key) if key.code == KeyCode::Enter => break Ok(Screen::Next(game_loop)),
            _ => continue,
        }
    }
}

fn game_loop() -> io::Result<Screen> {
    let term_size = terminal::size()?;
    let mut snake = Snake::new((0,0), 10);

    let mut food = Food::somewhere_within(term_size);

    loop {
        
        let snk_fmtr = snake.formatter(term_size, DEFAULT_PARTS);
        queue!(
            stdout(),
            terminal::Clear(ClearType::All),
            style::Print(format!("{}", snk_fmtr)),
            style::Print(format!("{}", food)),
        )?;
        stdout().flush()?;        

        snake.snaking(if event::poll(Duration::from_millis(250))? {
            use rs_snake::Direction::*;

            match event::read()? {
                Event::Key(key) if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') => {
                    break Ok(Screen::Exit);
                },
                Event::Key(key) if key.code == KeyCode::Up => Some(Up),
                Event::Key(key) if key.code == KeyCode::Down => Some(Down),
                Event::Key(key) if key.code == KeyCode::Left => Some(Left),
                Event::Key(key) if key.code == KeyCode::Right => Some(Right),
                _ => None,
            }
        } else {
            None
        });

        if snake.is_collide() {
            break Ok(Screen::Next(you_die));
        }
        if food.is_eaten_by(&snake) {
            snake.grow();
            food = Food::somewhere_within(term_size)
        }

    }
}

fn you_die() -> io::Result<Screen> {
    let (col, row) = terminal::size()?;
    queue!(
        stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(col / 2 - 3, row / 2),
        style::Print("You DIE!")
    )?;
    stdout().flush()?;
    loop {
        match event::read()? {
            Event::Key(key) if key.code == KeyCode::Enter => break Ok(Screen::Exit),
            _ => continue,
        }
    }
}

fn with_blank_screen(start: Print) -> io::Result<()> {
    enter_blank_screen()?;
    let mut actual_sceen = start;
    loop {
        match actual_sceen() {
            Err(err) => {
                exit_blank_screen()?;
                break Err(err);
            }
            Ok(state) => match state {
                Screen::Next(next_screen) => {
                    actual_sceen = next_screen;
                    continue;
                }
                Screen::Exit => {
                    exit_blank_screen()?;
                    break Ok(());
                }
            },
        }
    }
}

fn enter_blank_screen() -> io::Result<()> {
    queue!(
        stdout(),
        terminal::EnterAlternateScreen,
        terminal::Clear(ClearType::All),
        terminal::SetTitle("Snake"),
        cursor::Hide,
    )?;
    enable_raw_mode()?;
    stdout().flush()
}

fn exit_blank_screen() -> io::Result<()> {
    disable_raw_mode()?;
    queue!(stdout(), cursor::Show, terminal::LeaveAlternateScreen)?;
    stdout().flush()
}
