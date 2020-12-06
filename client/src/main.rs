use crossterm::style::style;
use completion::{AutoCompletor, trie::Trie};
use crossterm::{style::Color, cursor};
use crossterm::event::{self, KeyEvent};
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::{Attribute};
use crossterm::terminal::enable_raw_mode;
use crossterm::{
    style::{self},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::{time::Instant, env, io::stdout, process::exit};
use std::io::BufRead;
use std::io::Stdout;
use std::io::Write;
use std::{fs::File, io::BufReader};
use terminal::disable_raw_mode;

use completion::naive::NaiveAutoComplete;

static FILE_NAME: & 'static str = "./all_words.txt";

pub fn read_char() -> Result<char> {
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        }) = event::read()?
        {
            return Ok(c);
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.is_empty() {
        println!("Usage: client <trie|naive>");
        exit(-1);
    }
    let mut stdout = stdout();
    let auto_complete_type = &args[1];
    if !["trie", "naive"].contains(&&auto_complete_type[..]) {
        println!("Usage: client <trie|naive>; only 'trie' or 'naive' supported");
        exit(-1);
    } 
    let banner = style("Autocomplete\n")
    .with(Color::DarkGrey)
    .attribute(Attribute::Bold);
    stdout
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::MoveTo(0,0))?
        .queue(style::PrintStyledContent(banner))?
        .flush()?;
    
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(120);
    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ])
            .template("{spinner:.blue} {msg}"),
    );
    let s1 = Instant::now();
    pb.set_message(&format!("Reading suggestions from [{}]...", FILE_NAME));
    let input = input();
    let s2 = Instant::now();
    let inp = input
        .iter()
        .map(|(s, u)| (&s[..], *u))
        .collect::<Vec<(&str, u32)>>();
    pb.set_message(&format!("Initializing the auto_completor ({}) for {} suggestions...", auto_complete_type, inp.len()));
    // let start_time = Instant::now();
    let auto_completor = auto_completor_factory(auto_complete_type, inp);
    pb.finish_with_message(&format!("Initialized auto_completor ({})! [time_to_read:{} ms][time_to_initialize:{} ms]",
                                 auto_complete_type,
                                 s2.duration_since(s1).as_millis(),
                                 s2.elapsed().as_millis()
                                ));
    // println!("Initialized auto_completor({}) in {} ms", auto_complete_type, start_time.elapsed().as_millis());
    enable_raw_mode()?;
    stdout
        .queue(style::Print("Enter your input (press Esc to quit): "))?
        .queue(cursor::SavePosition)?
        .flush()?;
    let mut characters: Vec<char> = vec![];    
    loop {
        match read()? {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Char(ch) => {
                        characters.push(ch);
                        let prefix = characters.iter().collect::<String>();
                        let suggestions = suggestions(&auto_completor, &prefix[..]);
                        stdout
                            .queue(cursor::RestorePosition)?
                            .queue(style::Print(ch))?
                            .queue(cursor::SavePosition)?
                            .flush()?;

                        stdout
                            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?
                            .flush()?;
                        print_suggestions(&mut stdout, &suggestions, &prefix[..])?;
                        stdout.queue(cursor::RestorePosition)?.flush()?;
                    }
                    KeyCode::Backspace => {
                        let popped_char = characters.pop();
                        if popped_char.is_some() {
                            let prefix = characters.iter().collect::<String>();
                            let suggestions = suggestions(&auto_completor, &prefix[..]);
                            stdout
                                .queue(cursor::MoveLeft(1))?
                                .queue(cursor::SavePosition)?
                                .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?
                                .flush()?;

                            stdout
                                .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?
                                .flush()?;

                            print_suggestions(&mut stdout, &suggestions, &prefix[..])?;
                            stdout.queue(cursor::RestorePosition)?.flush()?;
                        }
                    }

                    _ => break,
                }
            }
            _ => break,
        }
    }
    disable_raw_mode()?;
    Ok(())
}

fn input() -> Vec<(String, u32)> {
    let file = File::open(FILE_NAME).unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .map(|r| (r.clone(), r.len() as u32))
        .collect::<Vec<(String, u32)>>()
}

fn suggestions(auto_completor: &Box< dyn AutoCompletor>, prefix: &str) -> Vec<String> {
    let c = auto_completor
        .suggestions(prefix)
        .iter()
        .map(|s| (*s.word).clone())
        .collect::<Vec<String>>();
    c
}

fn auto_completor_factory(auto_complete_type: &str, inp: Vec<(&str, u32)>) -> Box<dyn AutoCompletor> {
    if auto_complete_type == "trie" { 
        Box::new(Trie::new(&inp[..]))
    }  else  { 
        Box::new(NaiveAutoComplete::new(&inp[..]))
    }
}

fn print_suggestions<'a>(
    stdout: &'a mut Stdout,
    suggestions: &[String],
    prefix: &str,
) -> Result<&'a mut Stdout> {
    stdout.execute(cursor::MoveToNextLine(0))?;
    if suggestions.is_empty() {
        stdout.execute(style::Print("No suggestions"))?;
    }
    for s in suggestions {
        let suffix = s.strip_prefix(prefix).unwrap();
        stdout
            .queue(style::Print(format!(
                "{}{}{}{}{}{}",
                Attribute::Underlined,
                prefix,
                Attribute::NoUnderline,
                Attribute::Bold,
                suffix,
                Attribute::Reset
            )))?
            .queue(cursor::MoveToNextLine(0))?;
    }
    stdout.flush()?;
    Ok(stdout)
}
