#![feature(iter_advance_by)]

use clap::{Arg, Command};
use std::fs;
use std::iter::Peekable;
use std::path::Path;
use core::str::Chars;


fn main() {
    let m = Command::new("trade_config_formatter")
        .arg(Arg::new("file").index(1).required(true))
        .about("A tool to format DayZ trader config files")
        .get_matches();

    let file_path: &String = m.get_one("file").unwrap();

    work(&file_path).unwrap_or_else(|err| {
        println!("Error processing file: {}", err);
    });
}

fn work(file_path: &str) -> Result<(), String> {
    let contents = read_file(file_path)?;
    let parsed = process_file(contents)?;

    println!("Output:\n\n{:?}", parsed);
    Ok(())
}

fn read_file(file_path: &str) -> Result<String, String> {

    let p = Path::new(file_path);
    if !p.exists() || !p.is_file() {
        return Err(format!("The path provided is not valid"))
    }
    fs::read_to_string(p).map_err(|err| {
        format!("Error reading file: {:?}", err)
    })

}

#[derive(Debug)]
struct Comment(String);

#[derive(Debug)]
struct Line {
    text: String,
    comment: Option<Comment>,
}

#[derive(Debug)]
struct CSVLine {
    values: Vec<String>,
    comment: Option<Comment>
}

#[derive(Debug)]
enum CurrencyToken {
    Comment(Comment),
    Currency(CSVLine)
}

#[derive(Debug)]
struct CurrencyName {
    name: Line,
    currencies: Vec<CurrencyToken>
}

#[derive(Debug)]
enum CategoryItemToken {
    Comment(Comment),
    Item(CSVLine)
}

#[derive(Debug)]
struct TraderCategory {
    name: Line,
    items: Vec<CategoryItemToken>,
}

enum TraderCategoryToken {
    TraderCategory(TraderCategory),
    Comment(Comment)
}

#[derive(Debug)]
struct Trader {
    name: Line,
    categories: Vec<TraderCategory>
}

#[derive(Debug)]
enum Token {
    Comment(Comment),
    CurrencyName(CurrencyName),
    Trader(Trader)
}

fn process_file(contents: String) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = contents.chars().peekable();
    for c in chars.clone() {
        let res: Option<Token> = match c {
            '/' => parse_comment(&mut chars)?.map(|c| Token::Comment(c)),
            '<' => {
                parse_currency_name(&mut chars)?.map(|c| Token::CurrencyName(c))
            }
            _ => {
                None
            }
        };

        match res {
            Some(token) => tokens.push(token),
            _ => ()
        };
    }
    Ok(tokens)
}

fn parse_comment(chars: &mut Peekable<Chars>) -> Result<Option<Comment>, String> {
    consume_spaces(chars)?;

    let c0 = chars.next();
    let c1 = chars.next();
    
    if Some('/') != c0 || Some('/') == c1 {
        return Ok(None)
    }

    let mut msg: String = String::new();

    for s in chars {
        match s {
            '\n' | '\r' => break,
            s => msg.push(s)
        }
    }

    Ok(Some(Comment(msg)))

}

fn parse_line(chars: &mut Peekable<Chars>) -> Result<Line, String> {
    let mut text: String = String::new();
    let mut comment: Option<Comment> = None;
    for c in chars.clone() {
        match c {
            ' ' => (),
            '\n' | '\r' => break,
            '/' if chars.next() == Some('/') => {
                comment = parse_comment(chars)?;
            },
            c => text.push(c)
        };
    }
    Ok(Line{ text, comment })
}

fn parse_csv_line(chars: &mut Peekable<Chars>) -> Result<CSVLine, String> {

    let mut values: Vec<String> = Vec::new();
    let mut value: String = String::new();
    let mut comment: Option<Comment> = None;
    for c in chars.clone() {
        match c {
            ' ' => (),
            '\n' | '\r' => break,
            ',' => {
                values.push(value);
                value = String::new();
            },
            '/' if chars.next() == Some('/') => {
                comment = parse_comment(chars)?;
            },
            c => value.push(c)
        };
    }

    Ok(CSVLine { values, comment })
}

fn parse_currency(chars: &mut Peekable<Chars>) -> Result<Option<CSVLine>, String> {
    consume_spaces(chars)?;

    let c0 = chars.next();

    if Some('<') != c0 {
        return Ok(None);
    }

    let mut txt: String = String::new();
    let mut internal_idx = 0;
    for c in chars.clone() {
        match c {
            '>' | '/' => break,
            '\n' | '\r' => return Err(format!("Error parsing curency name, unexpected new line")),
            c => txt.push(c)
        }
        internal_idx = internal_idx + 1;
    }

    if txt != "Currency" {
        return Ok(None)
    }

    let line = parse_csv_line(chars)?;

    Ok(Some(line))
}

fn parse_currency_token(chars: &mut Peekable<Chars>) -> Result<Option<CurrencyToken>, String> {


    if let Some(comment)  = parse_comment(chars)? {
        return Ok(Some(CurrencyToken::Comment(comment)));
    }

    if let Some(currency) = parse_currency(chars)? {
        return Ok(Some(CurrencyToken::Currency(currency)));
    }

    Ok(None)

}

fn parse_currency_name(chars: &mut Peekable<Chars>) -> Result<Option<CurrencyName>, String> {
    consume_spaces(chars)?;

    let c0 = chars.next();

    if Some('<') != c0 {
        return Ok(None);
    }

    let mut txt: String = String::new();

    let mut internal_idx = 0;
    for c in chars.clone() {
        match c {
            '>' | '/' => break,
            '\n' | '\r' => return Err(format!("Error parsing curency name, unclosed tag")),
            c => txt.push(c)
        }
        internal_idx = internal_idx + 1;
    }

    if txt != "CurrencyName" {
        return Ok(None)
    }

    let line = parse_line(chars)?;

    let mut currencies = Vec::new();
    while let Some(currency) = parse_currency_token(chars)? {
        currencies.push(currency);
    }

    Ok(Some(CurrencyName {
        name: line,
        currencies
    }))

}

fn consume_spaces(chars: &mut Peekable<Chars>) -> Result<(), String> {
    for c in chars {
        match c {
            ' ' | '\n' | '\r' => (),
            _ => break,
        }
    }
    Ok(())
}
