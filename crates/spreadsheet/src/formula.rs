//! The formula language, with no kernel in it: a cell's name, the compiler from formula
//! text to an expression, the expression printed as an s-expression, and the reader that
//! takes the s-expression back.
//!
//! Everything here is a pure function of its input, which is what lets the endpoints in
//! `lib.rs` declare their answers cacheable: the compiled form of `=A1*2` is the same
//! today and tomorrow, so the kernel can keep it until the input it came from changes.

use std::fmt;

// ANCHOR: extent
/// The sheet's columns, left to right.
pub const COLUMNS: &str = "ABCD";

/// The sheet's rows, top to bottom: 1 to `ROWS`.
pub const ROWS: u32 = 6;
// ANCHOR_END: extent

/// The most letters a column may have in a formula (`ZZZ`), and the most digits a row may
/// have. A formula may name a cell off the sheet (its value is then `#REF`), but not one
/// whose name would not fit in a number.
const MAX_LETTERS: usize = 3;
const MAX_DIGITS: usize = 7;

/// How deeply a formula may nest, in parentheses or in minus signs. A bound refuses
/// rather than truncates: past it the formula is a syntax error, never a crashed page.
const MAX_DEPTH: usize = 64;

/// A cell's address: a column (0 for `A`) and a row (1 for the top row).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellRef {
    /// The column, from 0: `A` is 0, `Z` is 25, `AA` is 26.
    pub column: u32,
    /// The row, from 1.
    pub row: u32,
}

impl CellRef {
    /// The cell at `column` (from 0) and `row` (from 1).
    pub const fn new(column: u32, row: u32) -> Self {
        CellRef { column, row }
    }

    /// The cell spelled `text`, in the one spelling a name takes: capital letters, then a
    /// row with no leading zero (`A1`, `D6`, `AA10`). Anything else is `None`, `a1` and
    /// `A01` included, so a cell has exactly one name and one golden thread.
    ///
    /// ```
    /// use spreadsheet::CellRef;
    /// assert_eq!(CellRef::parse("B3"), Some(CellRef::new(1, 3)));
    /// assert_eq!(CellRef::parse("b3"), None);
    /// assert_eq!(CellRef::parse("B03"), None);
    /// ```
    pub fn parse(text: &str) -> Option<Self> {
        let digits_at = text.find(|c: char| c.is_ascii_digit())?;
        let (letters, digits) = text.split_at(digits_at);
        let letters_ok = !letters.is_empty()
            && letters.len() <= MAX_LETTERS
            && letters.bytes().all(|b| b.is_ascii_uppercase());
        let digits_ok = digits.len() <= MAX_DIGITS
            && !digits.starts_with('0')
            && digits.bytes().all(|b| b.is_ascii_digit());
        if !(letters_ok && digits_ok) {
            return None;
        }
        let column = letters
            .bytes()
            .fold(0u32, |n, b| n * 26 + u32::from(b - b'A') + 1)
            - 1;
        Some(CellRef::new(column, digits.parse().ok()?))
    }

    /// Whether this cell is on the sheet: columns `A`–`D`, rows 1–6.
    pub fn on_sheet(&self) -> bool {
        (self.column as usize) < COLUMNS.len() && (1..=ROWS).contains(&self.row)
    }

    /// Every cell of the sheet, row by row.
    pub fn all() -> impl Iterator<Item = CellRef> {
        (1..=ROWS).flat_map(|row| (0..COLUMNS.len() as u32).map(move |c| CellRef::new(c, row)))
    }
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut letters = Vec::new();
        let mut n = self.column + 1;
        while n > 0 {
            letters.push(b'A' + ((n - 1) % 26) as u8);
            n = (n - 1) / 26;
        }
        letters.reverse();
        write!(f, "{}{}", String::from_utf8_lossy(&letters), self.row)
    }
}

// ANCHOR: expr
/// A compiled formula.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A number, `2.5`.
    Number(f64),
    /// The value of a cell, `A1`.
    Cell(CellRef),
    /// Every cell in a rectangle, top-left to bottom-right, `(range A1 B2)`. Only ever an
    /// argument of `sum`.
    Range(CellRef, CellRef),
    /// Minus one thing, `(- A1)`.
    Negate(Box<Expr>),
    /// Two things and an operator, `(* B1 2)`.
    Binary(Op, Box<Expr>, Box<Expr>),
    /// The sum of its arguments, `(sum (range A1 A3) 10)`.
    Sum(Vec<Expr>),
    /// The current instant, `(now)`: the date and the time, read from the time chapter's
    /// `instant`.
    Now,
    /// Today's date, `(today)`.
    Today,
    /// The latest value written to a feed, `(feed acme)`: data that something other than the
    /// sheet writes.
    Feed(String),
    /// A formula that does not compile, and why: `(syntax-error "…")`. Its value is
    /// `#SYNTAX`. The compiler answers with this rather than refusing, so a formula with a
    /// mistake in it is still a representation the kernel can cache.
    SyntaxError(String),
}

/// An arithmetic operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `*`
    Multiply,
    /// `/`
    Divide,
}
// ANCHOR_END: expr

impl Op {
    fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Subtract => "-",
            Op::Multiply => "*",
            Op::Divide => "/",
        }
    }

    fn named(symbol: &str) -> Option<Op> {
        Some(match symbol {
            "+" => Op::Add,
            "-" => Op::Subtract,
            "*" => Op::Multiply,
            "/" => Op::Divide,
            _ => return None,
        })
    }
}

impl Expr {
    /// The cells and the ranges this expression names, each once, in the order they first
    /// appear.
    pub fn names(&self) -> (Vec<CellRef>, Vec<(CellRef, CellRef)>) {
        fn walk(e: &Expr, cells: &mut Vec<CellRef>, ranges: &mut Vec<(CellRef, CellRef)>) {
            match e {
                Expr::Cell(r) if !cells.contains(r) => cells.push(*r),
                Expr::Range(a, b) if !ranges.contains(&(*a, *b)) => ranges.push((*a, *b)),
                Expr::Negate(inner) => walk(inner, cells, ranges),
                Expr::Binary(_, left, right) => {
                    walk(left, cells, ranges);
                    walk(right, cells, ranges);
                }
                Expr::Sum(args) => args.iter().for_each(|a| walk(a, cells, ranges)),
                _ => {}
            }
        }
        let (mut cells, mut ranges) = (Vec::new(), Vec::new());
        walk(self, &mut cells, &mut ranges);
        (cells, ranges)
    }
}

// ANCHOR: print
/// The expression as an s-expression, in the one spelling the reader takes back.
///
/// ```
/// use spreadsheet::formula::compile;
/// assert_eq!(compile("=A1 + b2*2").to_string(), "(+ A1 (* B2 2))");
/// assert_eq!(compile("=SUM(A3:A1, 10)").to_string(), "(sum (range A1 A3) 10)");
/// assert_eq!(compile("=now()").to_string(), "(now)");
/// assert_eq!(compile("=FEED(Acme) * B1").to_string(), "(* (feed acme) B1)");
/// ```
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => f.write_str(&number_text(*n)),
            Expr::Cell(r) => write!(f, "{r}"),
            Expr::Range(a, b) => write!(f, "(range {a} {b})"),
            Expr::Negate(inner) => write!(f, "(- {inner})"),
            Expr::Binary(op, left, right) => write!(f, "({} {left} {right})", op.symbol()),
            Expr::Sum(args) => {
                f.write_str("(sum")?;
                for arg in args {
                    write!(f, " {arg}")?;
                }
                f.write_str(")")
            }
            Expr::Now => f.write_str("(now)"),
            Expr::Today => f.write_str("(today)"),
            Expr::Feed(name) => write!(f, "(feed {name})"),
            Expr::SyntaxError(why) => {
                let quoted = why.replace('\\', "\\\\").replace('"', "\\\"");
                write!(f, "(syntax-error \"{quoted}\")")
            }
        }
    }
}
// ANCHOR_END: print

/// A number as the sheet writes it: a whole number with no point (`12`), anything else
/// rounded to ten places with the trailing zeros dropped (`0.3`, not `0.30000000000000004`).
///
/// ```
/// use spreadsheet::formula::number_text;
/// assert_eq!(number_text(12.0), "12");
/// assert_eq!(number_text(0.1 + 0.2), "0.3");
/// assert_eq!(number_text(-0.0), "0");
/// assert_eq!(number_text(1.0 / 3.0), "0.3333333333");
/// ```
pub fn number_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    let fixed = format!("{n:.10}");
    let trimmed = fixed.trim_end_matches('0').trim_end_matches('.');
    match trimmed {
        "-0" | "" => "0".to_string(),
        text => text.to_string(),
    }
}

/// `text` as a number, if it is spelled as one: digits, then optionally a point and more
/// digits, with a minus in front if it is negative (`-2.5`). Nothing else is a number:
/// not `1e3`, not `.5`, not `5.`.
pub fn parse_number(text: &str) -> Option<f64> {
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (whole, fraction) = match unsigned.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (unsigned, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if digits(whole) && fraction.is_none_or(digits) {
        text.parse().ok()
    } else {
        None
    }
}

/// The most characters a feed's name may have.
const MAX_FEED_NAME: usize = 32;

/// A feed's name in its one spelling, if `text` is one: a lower-case letter, then lower-case
/// letters and digits, at most 32 of them (`acme`, `fx2`). One spelling per feed, so one
/// golden thread per feed.
///
/// ```
/// use spreadsheet::formula::feed_name;
/// assert!(feed_name("acme"));
/// assert!(!feed_name("Acme"));
/// assert!(!feed_name("2acme"));
/// assert!(!feed_name("ac-me"));
/// ```
pub fn feed_name(text: &str) -> bool {
    text.len() <= MAX_FEED_NAME
        && text.starts_with(|c: char| c.is_ascii_lowercase())
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
}

// ANCHOR: compile
/// Compile a formula, `=A1*2`, into an expression.
///
/// This never fails. A formula that does not parse compiles to [`Expr::SyntaxError`],
/// saying what was expected where, so a mistake is a value the sheet shows (`#SYNTAX`)
/// rather than a request that failed. The compiler accepts a formula however it is
/// spelled (`=a1 + 2`, `=SUM(a3:a1)`) and writes the one spelling back (`(+ A1 2)`,
/// `(sum (range A1 A3))`).
pub fn compile(formula: &str) -> Expr {
    let mut parser = Parser {
        text: formula.as_bytes(),
        at: 0,
        depth: 0,
    };
    match parser.formula() {
        Ok(expr) => expr,
        Err(why) => Expr::SyntaxError(why),
    }
}
// ANCHOR_END: compile

/// A recursive-descent parser over the formula's bytes. The grammar, lowest precedence
/// first:
///
/// ```text
/// formula := '=' sum-expr
/// sum-expr := product (('+' | '-') product)*
/// product  := unary (('*' | '/') unary)*
/// unary    := '-' unary | primary
/// primary  := number | cell | '(' sum-expr ')'
///           | 'SUM' '(' arg (',' arg)* ')' | 'NOW' '(' ')' | 'TODAY' '(' ')'
///           | 'FEED' '(' feed-name ')'
/// arg      := cell ':' cell | sum-expr
/// ```
struct Parser<'t> {
    text: &'t [u8],
    at: usize,
    depth: usize,
}

type Parsed<T> = std::result::Result<T, String>;

impl Parser<'_> {
    fn skip_spaces(&mut self) {
        while self.text.get(self.at).is_some_and(|b| *b == b' ') {
            self.at += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_spaces();
        self.text.get(self.at).copied()
    }

    fn eat(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    /// The refusal for "expected `what` here".
    fn expected(&self, what: &str) -> String {
        let rest = String::from_utf8_lossy(&self.text[self.at.min(self.text.len())..]);
        if rest.is_empty() {
            format!("expected {what}, and the formula ended")
        } else {
            let shown: String = rest.chars().take(12).collect();
            format!("expected {what} at `{shown}`")
        }
    }

    fn formula(&mut self) -> Parsed<Expr> {
        if !self.eat(b'=') {
            return Err(self.expected("= to start a formula"));
        }
        let expr = self.sum_expr()?;
        if self.peek().is_some() {
            return Err(self.expected("an operator (+ - * /) or the end"));
        }
        Ok(expr)
    }

    fn nest(&mut self) -> Parsed<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            Err(format!("the formula nests more than {MAX_DEPTH} deep"))
        } else {
            Ok(())
        }
    }

    fn sum_expr(&mut self) -> Parsed<Expr> {
        let mut left = self.product()?;
        loop {
            let op = match self.peek() {
                Some(b'+') => Op::Add,
                Some(b'-') => Op::Subtract,
                _ => return Ok(left),
            };
            self.at += 1;
            let right = self.product()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
    }

    fn product(&mut self) -> Parsed<Expr> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(b'*') => Op::Multiply,
                Some(b'/') => Op::Divide,
                _ => return Ok(left),
            };
            self.at += 1;
            let right = self.unary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
    }

    fn unary(&mut self) -> Parsed<Expr> {
        if self.eat(b'-') {
            self.nest()?;
            let inner = self.unary()?;
            self.depth -= 1;
            return Ok(Expr::Negate(Box::new(inner)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Parsed<Expr> {
        match self.peek() {
            Some(b'0'..=b'9') => self.number(),
            Some(b'(') => {
                self.at += 1;
                self.nest()?;
                let inner = self.sum_expr()?;
                self.depth -= 1;
                if !self.eat(b')') {
                    return Err(self.expected(")"));
                }
                Ok(inner)
            }
            Some(b) if b.is_ascii_alphabetic() => {
                let word = self.word();
                if self.peek() == Some(b'(') {
                    return self.call(&word);
                }
                let cell = self.cell_named(&word)?;
                if self.peek() == Some(b':') {
                    return Err(format!("a range ({cell}:…) only goes inside SUM(…)"));
                }
                Ok(Expr::Cell(cell))
            }
            _ => Err(self.expected("a number, a cell or (")),
        }
    }

    fn number(&mut self) -> Parsed<Expr> {
        let start = self.at;
        while self.text.get(self.at).is_some_and(u8::is_ascii_digit) {
            self.at += 1;
        }
        if self.text.get(self.at) == Some(&b'.') {
            self.at += 1;
            let fraction = self.at;
            while self.text.get(self.at).is_some_and(u8::is_ascii_digit) {
                self.at += 1;
            }
            if self.at == fraction {
                return Err(self.expected("a digit after the point"));
            }
        }
        let text = String::from_utf8_lossy(&self.text[start..self.at]);
        text.parse()
            .map(Expr::Number)
            .map_err(|_| format!("`{text}` is not a number"))
    }

    /// Letters and digits, as one word: `A1`, `sum`, `b12`.
    fn word(&mut self) -> String {
        let start = self.at;
        while self
            .text
            .get(self.at)
            .is_some_and(u8::is_ascii_alphanumeric)
        {
            self.at += 1;
        }
        String::from_utf8_lossy(&self.text[start..self.at]).to_string()
    }

    /// The cell `word` names, spelled any case: `b3` is `B3`.
    fn cell_named(&self, word: &str) -> Parsed<CellRef> {
        CellRef::parse(&word.to_ascii_uppercase()).ok_or_else(|| {
            format!("`{word}` is not a cell: a column (A, B, …) then a row (1, 2, …)")
        })
    }

    fn call(&mut self, name: &str) -> Parsed<Expr> {
        let upper = name.to_ascii_uppercase();
        match upper.as_str() {
            "SUM" => {}
            "NOW" | "TODAY" => {
                self.at += 1; // the `(`
                if !self.eat(b')') {
                    return Err(format!("{upper}() takes nothing between its parentheses"));
                }
                return Ok(if upper == "NOW" {
                    Expr::Now
                } else {
                    Expr::Today
                });
            }
            "FEED" => {
                self.at += 1; // the `(`
                self.skip_spaces();
                let word = self.word().to_ascii_lowercase();
                if !feed_name(&word) {
                    return Err(
                        "FEED takes a feed's name: a letter, then letters and digits (e.g. FEED(acme))"
                            .to_string(),
                    );
                }
                if !self.eat(b')') {
                    return Err(self.expected(")"));
                }
                return Ok(Expr::Feed(word));
            }
            _ => {
                return Err(format!(
                    "`{name}` is not a function the sheet knows: it knows SUM, NOW, TODAY and FEED"
                ))
            }
        }
        self.at += 1; // the `(`
        self.nest()?;
        let mut args = vec![self.arg()?];
        while self.eat(b',') {
            args.push(self.arg()?);
        }
        self.depth -= 1;
        if !self.eat(b')') {
            return Err(self.expected(", or )"));
        }
        Ok(Expr::Sum(args))
    }

    /// One argument of SUM: a range, `A1:B2`, or any expression.
    fn arg(&mut self) -> Parsed<Expr> {
        let before = self.at;
        if self.peek().is_some_and(|b| b.is_ascii_alphabetic()) {
            let word = self.word();
            if self.peek() == Some(b':') {
                let from = self.cell_named(&word)?;
                self.at += 1;
                self.skip_spaces();
                let word = self.word();
                let to = self.cell_named(&word)?;
                // One spelling for a rectangle: top-left, then bottom-right.
                let top_left = CellRef::new(from.column.min(to.column), from.row.min(to.row));
                let bottom_right = CellRef::new(from.column.max(to.column), from.row.max(to.row));
                return Ok(Expr::Range(top_left, bottom_right));
            }
            self.at = before;
        }
        self.sum_expr()
    }
}

// ANCHOR: read
/// Read an s-expression that [`Expr`]'s `Display` wrote back into the expression.
///
/// This is the other half of the compiled form being a resource: whatever reads
/// `urn:iki:tutorial:sheet:formula:{ref}` gets text, and this is how it gets the
/// expression back. It takes only what the printer writes; anything else is refused.
pub fn read(text: &str) -> std::result::Result<Expr, String> {
    let tokens = tokens(text)?;
    let mut at = 0;
    let expr = read_form(&tokens, &mut at, 0)?;
    if at != tokens.len() {
        return Err("more than one form".to_string());
    }
    Ok(expr)
}
// ANCHOR_END: read

#[derive(Debug, PartialEq)]
enum Token {
    Open,
    Close,
    Atom(String),
    Str(String),
}

fn tokens(text: &str) -> std::result::Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\n' | '\t' => {}
            '(' => out.push(Token::Open),
            ')' => out.push(Token::Close),
            '"' => {
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('\\') => s.push(chars.next().ok_or("an unfinished escape")?),
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => return Err("an unfinished string".to_string()),
                    }
                }
                out.push(Token::Str(s));
            }
            c => {
                let mut atom = c.to_string();
                while let Some(&next) = chars.peek() {
                    if matches!(next, ' ' | '\n' | '\t' | '(' | ')' | '"') {
                        break;
                    }
                    atom.push(next);
                    chars.next();
                }
                out.push(Token::Atom(atom));
            }
        }
    }
    Ok(out)
}

fn read_form(tokens: &[Token], at: &mut usize, depth: usize) -> std::result::Result<Expr, String> {
    if depth > MAX_DEPTH * 2 {
        return Err("nested too deeply".to_string());
    }
    let token = tokens.get(*at).ok_or("the text ended")?;
    *at += 1;
    match token {
        Token::Atom(atom) => parse_number(atom)
            .map(Expr::Number)
            .or_else(|| CellRef::parse(atom).map(Expr::Cell))
            .ok_or_else(|| format!("`{atom}` is neither a number nor a cell")),
        Token::Str(_) | Token::Close => Err("a form in the wrong place".to_string()),
        Token::Open => {
            let head = match tokens.get(*at) {
                Some(Token::Atom(head)) => head.clone(),
                _ => return Err("a list with no operator".to_string()),
            };
            *at += 1;
            let form = if head == "feed" {
                match tokens.get(*at) {
                    Some(Token::Atom(name)) if feed_name(name) => {
                        *at += 1;
                        Expr::Feed(name.clone())
                    }
                    _ => return Err("feed takes a feed's name".to_string()),
                }
            } else if head == "syntax-error" {
                match tokens.get(*at) {
                    Some(Token::Str(why)) => {
                        *at += 1;
                        Expr::SyntaxError(why.clone())
                    }
                    _ => return Err("syntax-error takes a string".to_string()),
                }
            } else {
                let mut args = Vec::new();
                while !matches!(tokens.get(*at), Some(Token::Close) | None) {
                    args.push(read_form(tokens, at, depth + 1)?);
                }
                form_of(&head, args)?
            };
            match tokens.get(*at) {
                Some(Token::Close) => {
                    *at += 1;
                    Ok(form)
                }
                _ => Err(format!("`({head} …` is not closed")),
            }
        }
    }
}

fn form_of(head: &str, mut args: Vec<Expr>) -> std::result::Result<Expr, String> {
    match (head, args.len()) {
        ("-", 1) => Ok(Expr::Negate(Box::new(args.remove(0)))),
        (op, 2) if Op::named(op).is_some() => {
            let right = args.remove(1);
            let left = args.remove(0);
            Ok(Expr::Binary(
                Op::named(op).expect("checked"),
                Box::new(left),
                Box::new(right),
            ))
        }
        ("range", 2) => match (&args[0], &args[1]) {
            (Expr::Cell(a), Expr::Cell(b)) => Ok(Expr::Range(*a, *b)),
            _ => Err("range takes two cells".to_string()),
        },
        ("sum", n) if n > 0 => Ok(Expr::Sum(args)),
        ("now", 0) => Ok(Expr::Now),
        ("today", 0) => Ok(Expr::Today),
        _ => Err(format!(
            "`{head}` with {} arguments is not a form",
            args.len()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_has_one_spelling_and_every_column_name_round_trips() {
        for (text, cell) in [
            ("A1", CellRef::new(0, 1)),
            ("D6", CellRef::new(3, 6)),
            ("Z9", CellRef::new(25, 9)),
            ("AA10", CellRef::new(26, 10)),
            ("ZZ1", CellRef::new(701, 1)),
            ("AAA1", CellRef::new(702, 1)),
        ] {
            assert_eq!(CellRef::parse(text), Some(cell), "{text}");
            assert_eq!(cell.to_string(), text);
        }
        for refused in [
            "a1",
            "A0",
            "A01",
            "1A",
            "A",
            "",
            "AAAA1",
            "A1B",
            "A-1",
            "A12345678",
        ] {
            assert_eq!(CellRef::parse(refused), None, "{refused}");
        }
        assert!(CellRef::parse("D6").unwrap().on_sheet());
        assert!(!CellRef::parse("E1").unwrap().on_sheet());
        assert!(!CellRef::parse("A7").unwrap().on_sheet());
        assert_eq!(CellRef::all().count(), 24);
    }

    #[test]
    fn formulas_compile_to_one_spelling() {
        for (formula, compiled) in [
            ("=5", "5"),
            ("=2.50", "2.5"),
            ("=a1", "A1"),
            ("= A1 + 2 * B2", "(+ A1 (* 2 B2))"),
            ("=(A1+2)*B2", "(* (+ A1 2) B2)"),
            ("=A1-B1-C1", "(- (- A1 B1) C1)"),
            ("=-A1", "(- A1)"),
            ("=--1", "(- (- 1))"),
            ("=A1/0", "(/ A1 0)"),
            ("=sum(a3:a1)", "(sum (range A1 A3))"),
            ("=SUM(B2:A1, 3, C1*2)", "(sum (range A1 B2) 3 (* C1 2))"),
            ("=E1", "E1"),
            ("=NOW()", "(now)"),
            ("=today( )", "(today)"),
            ("=FEED( Acme )*2", "(* (feed acme) 2)"),
            ("=SUM(FEED(a1), A1)", "(sum (feed a1) A1)"),
        ] {
            let expr = compile(formula);
            assert_eq!(expr.to_string(), compiled, "{formula}");
            assert_eq!(read(compiled), Ok(expr), "{compiled} reads back");
        }
    }

    #[test]
    fn a_formula_that_does_not_parse_compiles_to_a_syntax_error() {
        for (formula, why) in [
            ("=", "expected a number, a cell or (, and the formula ended"),
            (
                "=1+",
                "expected a number, a cell or (, and the formula ended",
            ),
            ("=(1", "expected ), and the formula ended"),
            ("=1 2", "expected an operator (+ - * /) or the end at `2`"),
            ("=A1:A3", "a range (A1:…) only goes inside SUM(…)"),
            (
                "=MAX(A1)",
                "`MAX` is not a function the sheet knows: it knows SUM, NOW, TODAY and FEED",
            ),
            (
                "=A01",
                "`A01` is not a cell: a column (A, B, …) then a row (1, 2, …)",
            ),
            (
                "=1.",
                "expected a digit after the point, and the formula ended",
            ),
            ("=SUM()", "expected a number, a cell or ( at `)`"),
            ("=NOW(1)", "NOW() takes nothing between its parentheses"),
            ("=FEED(ac-me)", "expected ) at `-me)`"),
            (
                "=FEED(2x)",
                "FEED takes a feed's name: a letter, then letters and digits (e.g. FEED(acme))",
            ),
            (
                "=FEED()",
                "FEED takes a feed's name: a letter, then letters and digits (e.g. FEED(acme))",
            ),
            ("=FEED(acme", "expected ), and the formula ended"),
            (
                "=1 € 2",
                "expected an operator (+ - * /) or the end at `€ 2`",
            ),
        ] {
            assert_eq!(
                compile(formula),
                Expr::SyntaxError(why.to_string()),
                "{formula}"
            );
        }
        let deep = format!("={}1{}", "(".repeat(100), ")".repeat(100));
        assert!(matches!(compile(&deep), Expr::SyntaxError(_)));
        let minus = format!("={}1", "-".repeat(100));
        assert!(matches!(compile(&minus), Expr::SyntaxError(_)));
    }

    #[test]
    fn a_syntax_error_reads_back_with_its_quotes() {
        let expr = Expr::SyntaxError("a \"quoted\" \\ thing".to_string());
        assert_eq!(read(&expr.to_string()), Ok(expr));
    }

    #[test]
    fn the_reader_refuses_what_the_printer_never_writes() {
        for refused in [
            "",
            "(",
            ")",
            "(+ 1)",
            "(foo 1 2)",
            "(range 1 2)",
            "x",
            "1 2",
            "(sum)",
            "(now 1)",
            "(feed)",
            "(feed Acme)",
            "(feed 1)",
        ] {
            assert!(read(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn names_are_listed_once_in_order() {
        let expr = compile("=B1 + SUM(A1:A3, B1) + C2 + SUM(A3:A1)");
        let (cells, ranges) = expr.names();
        assert_eq!(cells, vec![CellRef::new(1, 1), CellRef::new(2, 2)]);
        assert_eq!(ranges, vec![(CellRef::new(0, 1), CellRef::new(0, 3))]);
    }

    #[test]
    fn numbers_have_one_spelling_each_way() {
        for (text, n) in [("5", 5.0), ("-2.5", -2.5), ("0.25", 0.25), ("007", 7.0)] {
            assert_eq!(parse_number(text), Some(n), "{text}");
        }
        for refused in [
            "", "-", "1e3", ".5", "5.", "+1", "1,000", " 1", "NaN", "inf",
        ] {
            assert_eq!(parse_number(refused), None, "{refused}");
        }
        assert_eq!(number_text(1e20), "100000000000000000000");
        assert_eq!(number_text(-7.25), "-7.25");
        assert_eq!(number_text(1e-12), "0");
    }
}
