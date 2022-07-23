#![feature(iter_advance_by)]

use clap::{Arg, Command};
use std::io::{stderr, Write};
use std::{fs, fmt, process};
use std::iter::Peekable;
use std::path::Path;
use core::str::Chars;

const PADDING: usize =  60;

fn main() {
    let m = Command::new("trade_config_formatter")
        .arg(Arg::new("file").index(1).required(true))
        .about("A tool to format DayZ trader config files")
        .get_matches();

    let file_path: &String = m.get_one("file").unwrap();

    work(&file_path).unwrap_or_else(|err| {
        stderr().write(format!("\nError processing file: {}\n\n", err).as_bytes()).unwrap();
        process::exit(-1); 
    });
}

fn work(file_path: &str) -> Result<(), String> {
    let contents = read_file(file_path)?;
    let parsed = process_file(contents)?;

    for p in parsed.iter() {
        println!("{}", p);
    }
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

#[derive(Debug, Clone)]
struct Comment(String);

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "// {}", self.0)
    }
}

#[derive(Debug)]
struct Line {
    text: String,
    comment: Option<Comment>,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}\n", self.text, self.comment.as_ref().map(|c| format!("{}", c)).unwrap_or("".into()))
    }
}

#[derive(Debug)]
struct CSVLine {
    values: Vec<String>,
    comment: Option<Comment>
}

impl fmt::Display for CSVLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.values.len();
        for i in 0..len {
            if let Some(v) = self.values.get(i) {
                let mut str = String::from(v);
                if i != len -1 {
                    str.push(',');
                }
                write!(f, "{:0width$}", str, width = PADDING)?;

            };
        }

        if let Some(c) = self.comment.as_ref() {
            write!(f, " {}", c)?;
        }

        write!(f, "\n")?;

        Ok(())
    }
}


#[derive(Debug)]
enum CurrencyToken {
    Comment(Comment),
    Currency(CSVLine)
}

impl fmt::Display for CurrencyToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CurrencyToken::Comment(c) => write!(f, "{}", c),
            CurrencyToken::Currency(c) => write!(f, "{}", c)
        }
    }
}

#[derive(Debug)]
struct CurrencyName {
    name: Line,
    currencies: Vec<CurrencyToken>
}

impl fmt::Display for CurrencyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<CurrencyName> {}", self.name)?;
        for c in self.currencies.iter() {
            write!(f, "    {}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct CategoryItem {
    class: String,
    amount: String,
    buy_value: String,
    sell_value: String,
    comment: Option<Comment>,
}

impl TryFrom<&CSVLine> for CategoryItem {
    type Error = String;

    fn try_from(value: &CSVLine) -> Result<Self, Self::Error> {
        if value.values.len() != 4 {
            return Err(format!("Missing values to create a category item, probably a missing comma parsing {:?}", value))
        }

        Ok(CategoryItem {
            class: value.values.get(0).unwrap().clone(),
            amount: value.values.get(1).unwrap().clone(),
            buy_value: value.values.get(2).unwrap().clone(),
            sell_value: value.values.get(3).unwrap().clone(),
            comment: value.comment.clone()
        })
    }
}

impl fmt::Display for CategoryItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let comment = self.comment.as_ref().map(|c| c.to_string()).unwrap_or_default();
        write!(f, "        {},{},{},{}{}", self.class, self.amount, self.buy_value, self.sell_value, comment)
    }
}

#[derive(Debug)]
enum CategoryItemToken {
    CategoryItem(CategoryItem),
    Comment(Comment)
}

impl fmt::Display for CategoryItemToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CategoryItemToken::Comment(c) => write!(f, "\t{}", c),
            CategoryItemToken::CategoryItem(c) => write!(f, "{}", c)
        }
    }
}

#[derive(Debug)]
struct TraderCategory {
    name: Line,
    items: Vec<CategoryItemToken>,
}

impl fmt::Display for TraderCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "    <Category> {}", self.name)?;
        for c in self.items.iter() {
            write!(f, "        {}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
enum TraderCategoryToken {
    TraderCategory(TraderCategory),
    Comment(Comment)
}

impl fmt::Display for TraderCategoryToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraderCategoryToken::Comment(c) => write!(f, "\t{}", c),
            TraderCategoryToken::TraderCategory(c) => write!(f, "{}", c)
        }
    }
}

#[derive(Debug)]
struct Trader {
    name: Line,
    categories: Vec<TraderCategoryToken>
}

impl fmt::Display for Trader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Trader> {}", self.name)?;
        for c in self.categories.iter() {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct OpenFile(Line);

impl fmt::Display for OpenFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<OpenFile> {}", self.0)
    }
}

#[derive(Debug)]
struct FileEnd(Line);

impl fmt::Display for FileEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<FileEnd> {}", self.0)
    }
}

#[derive(Debug)]
enum Token {
    Comment(Comment),
    CurrencyName(CurrencyName),
    Trader(Trader),
    OpenFile(OpenFile),
    FileEnd(FileEnd)
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Comment(c) => write!(f, "{}", c),
            Token::CurrencyName(c) => write!(f, "{}", c),
            Token::Trader(t) => write!(f, "{}", t),
            Token::OpenFile(o) => write!(f, "{}", o),
            Token::FileEnd(fe) => write!(f, "{}", fe)
        }
    }
}

fn process_file(contents: String) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = contents.chars().peekable();
    while let Some(_) = chars.peek() {
        if let Some(t) = parse_token(&mut chars)? {
            tokens.push(t);
        } else {
            chars.next();
        }
    }

    Ok(tokens)

    // if let Some(Token::FileEnd(_)) = tokens.last() {
    //     Ok(tokens)
    // } else {
    //     Err("File is malformed, parsing didn't end with <FileEnd>".into())
    // }
}

fn parse_token(chars: &mut Peekable<Chars>) -> Result<Option<Token>, String> {
    consume_spaces(chars)?;
    if let Some(c) = parse_comment(chars)? {
        return Ok(Some(Token::Comment(c)));
    }

    if let Some(c) = parse_currency_name(chars)? {
        return Ok(Some(Token::CurrencyName(c)));
    }

    if let Some(t) = parse_trader(chars)? {
        return Ok(Some(Token::Trader(t)));
    }

    if let Some(o) = parse_open_file(chars)? {
        return Ok(Some(Token::OpenFile(o)))
    }

    if let Some(fe) = parse_file_end(chars)? {
        return Ok(Some(Token::FileEnd(fe)))
    }

    Ok(None)
}

fn parse_file_end(chars: &mut Peekable<Chars>) -> Result<Option<FileEnd>, String> {
    consume_spaces(chars)?;
    let c0 = chars.peek();

    if Some(&'<') != c0 {
        return Ok(None);
    }



    let mut txt: String = String::new();

    let mut internal_idx = 0;
    let mut ichars = chars.clone();
    ichars.next();
    for c in ichars {
        match c {
            '>' | '/' => break,
            '\n' | '\r' => return Err(format!("Error parsing file end, unclosed tag")),
            c => txt.push(c)
        }
        internal_idx = internal_idx + 1;
    }

    if txt != "FileEnd" {
        return Ok(None)
    }

    chars.advance_by(internal_idx + 2).map_err(|_| {
        format!("Error advancing index parsing file end")
    })?;
    
    let line = parse_line(chars)?;

    Ok(Some(FileEnd(line)))

}

fn parse_open_file(chars: &mut Peekable<Chars>) -> Result<Option<OpenFile>, String> {
    consume_spaces(chars)?;
    let c0 = chars.peek();

    if Some(&'<') != c0 {
        return Ok(None);
    }



    let mut txt: String = String::new();

    let mut internal_idx = 0;
    let mut ichars = chars.clone();
    ichars.next();
    for c in ichars {
        match c {
            '>' | '/' => break,
            '\n' | '\r' => return Err(format!("Error parsing openfile, unclosed tag")),
            c => txt.push(c)
        }
        internal_idx = internal_idx + 1;
    }

    if txt != "OpenFile" {
        return Ok(None)
    }

    chars.advance_by(internal_idx + 2).map_err(|_| {
        format!("Error advancing index parsing open file")
    })?;
    
    let line = parse_line(chars)?;

    Ok(Some(OpenFile(line)))
}

fn parse_trader_category_item_token(chars: &mut Peekable<Chars>) -> Result<Option<CategoryItemToken>, String> {
    consume_spaces(chars)?;

    if let Some(comment) = parse_comment(chars)? {
        return Ok(Some(CategoryItemToken::Comment(comment)));
    }

    if let Some(item) = parse_csv_line(chars)? {
        let item = CategoryItem::try_from(&item)?;
        return Ok(Some(CategoryItemToken::CategoryItem(item)));
    }

    Ok(None)
}

fn parse_trader_category(chars: &mut Peekable<Chars>) -> Result<Option<TraderCategory>, String> {
    consume_spaces(chars)?;
    let c0 = chars.peek();

    if Some(&'<') != c0 {
        return Ok(None);
    }



    let mut txt: String = String::new();

    let mut internal_idx = 0;
    let mut ichars = chars.clone();
    ichars.next();
    for c in ichars {
        match c {
            '>' | '/' => break,
            '\n' | '\r' => return Err(format!("Error parsing trader category name, unclosed tag")),
            c => txt.push(c)
        }
        internal_idx = internal_idx + 1;
    }

    if txt != "Category" {
        return Ok(None)
    }


    chars.advance_by(internal_idx + 2).map_err(|_| {
        format!("Error advancing index parsing trader category name")
    })?;

    let line = parse_line(chars)?;

    let mut items = Vec::new();
    while let Some(item) = parse_trader_category_item_token(chars)? {
        items.push(item);
    }

    Ok(Some(TraderCategory {
        name: line,
        items
    }))
}

fn parse_trader_category_token(chars: &mut Peekable<Chars>) -> Result<Option<TraderCategoryToken>, String> {
    consume_spaces(chars)?;

    if let Some(comment) = parse_comment(chars)? {
        return Ok(Some(TraderCategoryToken::Comment(comment)));
    }

    if let Some(category) = parse_trader_category(chars)? {
        return Ok(Some(TraderCategoryToken::TraderCategory(category)));
    }

    Ok(None)

}

fn parse_trader(chars: &mut Peekable<Chars>) -> Result<Option<Trader>, String> {
    
    consume_spaces(chars)?;

    let c0 = chars.peek();

    if Some(&'<') != c0 {
        return Ok(None);
    }

    let mut txt: String = String::new();

    let mut internal_idx = 0;
    let mut ichars = chars.clone();
    ichars.next();
    for c in ichars {
        match c {
            '>' | '/' => break,
            '\n' | '\r' => return Err(format!("Error parsing trader name, unclosed tag")),
            c => txt.push(c)
        }
        internal_idx = internal_idx + 1;
    }

    if txt != "Trader" {
        return Ok(None)
    }

    chars.advance_by(internal_idx + 2).map_err(|_| {
        format!("Error advancing index parsing trader name")
    })?;

    let line = parse_line(chars)?;



    let mut categories = Vec::new();
    while let Some(currency) = parse_trader_category_token(chars)? {
        categories.push(currency);
    }



    Ok(Some(Trader {
        name: line,
        categories
    }))


}

fn parse_comment(chars: &mut Peekable<Chars>) -> Result<Option<Comment>, String> {
    consume_spaces(chars)?;

    let c0 = chars.peek();
    
    if Some(&'/') != c0 {
        let mut further = chars.clone();
        further.next();
        let c1 = further.peek();
        if Some(&'/') != c1 {
            return Ok(None)
        }
    }

    chars.next();
    chars.next();
    consume_spaces(chars)?;

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
    consume_only_spaces(chars)?;
    let mut text: String = String::new();
    let mut comment: Option<Comment> = None;
    while let Some(c) = chars.peek() {
        match c {
            '\n' | '\r' => {
                text = text.trim().into();
                chars.next();
                break
            },
            '/' => {
                comment = parse_comment(chars)?;
                break;
            },
            c => text.push(*c)
        };
        chars.next();
    }

    Ok(Line{ text, comment })
}

fn parse_csv_line(chars: &mut Peekable<Chars>) -> Result<Option<CSVLine>, String> {
    consume_only_spaces(chars)?;
    let mut values: Vec<String> = Vec::new();
    let mut value: String = String::new();
    let mut comment: Option<Comment> = None;

    while let Some(c) = chars.peek() {
        match c {
            '<' => return Ok(None),
            '\n' | '\r' => {
                value = value.trim().into();
                if value.len() > 0 {
                    values.push(value);
                }
                chars.next();
                break;
            },
            ',' => {
                value = value.trim().into();
                if value.len() > 0 {
                    values.push(value);
                }
                value = String::new();
                chars.next();
            },
            '/' => {
                comment = parse_comment(chars)?;
            },
            c => {
                value.push(*c);
                chars.next();
            }
        };
    }

    if values.is_empty() && comment.is_none() {
        return Ok(None)
    } else {
        Ok(Some(CSVLine { values, comment }))
    }

}

fn parse_currency(chars: &mut Peekable<Chars>) -> Result<Option<CSVLine>, String> {
    consume_spaces(chars)?;

    let c0 = chars.peek();

    if Some(&'<') != c0 {
        return Ok(None);
    }

    let mut txt: String = String::new();
    let mut internal_idx = 0;
    let mut ichars = chars.clone();
    ichars.next();
    for c in ichars {
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

    chars.advance_by(internal_idx + 2).map_err(|_| {
        format!("Error advancing index parsing currency")
    })?;

    let line = parse_csv_line(chars)?;

    Ok(line)
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

    let c0 = chars.peek();

    if Some(&'<') != c0 {
        return Ok(None);
    }

    let mut txt: String = String::new();

    let mut internal_idx = 0;
    let mut ichars = chars.clone();
    ichars.next();
    for c in ichars {
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

    chars.advance_by(internal_idx + 2).map_err(|_| {
        format!("Error advancing index parsing currency name")
    })?;

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
    while let Some(c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' | '\r' => (),
            _ => break,
        }
        chars.next();
    }
    Ok(())
}

fn consume_only_spaces(chars: &mut Peekable<Chars>) -> Result<(), String> {
    while let Some(c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' => (),
            _ => break,
        }
        chars.next();
    }
    Ok(())
}