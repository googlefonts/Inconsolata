use std::borrow::Cow;
use std::collections::HashMap;

/// An enum representing a property list.
#[derive(Clone, Debug)]
pub enum Plist {
    Dictionary(HashMap<String, Plist>),
    Array(Vec<Plist>),
    String(String),
    Integer(i64),
    Float(f64),
}

#[derive(Debug)]
pub enum Error {
    UnexpectedChar(char),
    UnclosedString,
    UnknownEscape,
    NotAString,
    ExpectedEquals,
    ExpectedComma,
    ExpectedSemicolon,
    SomethingWentWrong,
}

enum Token<'a> {
    Eof,
    OpenBrace,
    OpenParen,
    String(Cow<'a, str>),
    Atom(&'a str),
}

fn is_numeric(b: u8) -> bool {
    (b >= b'0' && b <= b'9') || b == b'.' || b == b'-'
}

fn is_alnum(b: u8) -> bool {
    is_numeric(b) || (b >= b'A' && b <= b'Z') || (b >= b'a' && b <= b'z') || b == b'_'
}

// Used for serialization; make sure UUID's get quoted
fn is_alnum_strict(b: u8) -> bool {
    is_alnum(b) && b != b'-'
}

fn is_ascii_digit(b: u8) -> bool {
    b >= b'0' && b <= b'9'
}

fn is_hex_upper(b: u8) -> bool {
    (b >= b'0' && b <= b'9') || (b >= b'A' && b <= b'F')
}

fn is_ascii_whitespace(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\r' || b == b'\n'
}

fn numeric_ok(s: &str) -> bool {
    let s = s.as_bytes();
    if s.is_empty() {
        return false;
    }
    if s.iter().all(|&b| is_hex_upper(b)) && !s.iter().all(|&b| is_ascii_digit(b)) {
        return false;
    }
    if s.len() > 1 && s[0] == b'0' {
        return !s.iter().all(|&b| is_ascii_digit(b));
    }
    true
}

fn skip_ws(s: &str, mut ix: usize) -> usize {
    while ix < s.len() && is_ascii_whitespace(s.as_bytes()[ix]) {
        ix += 1;
    }
    ix
}

fn escape_string(buf: &mut String, s: &str) {
    if !s.is_empty() && s.as_bytes().iter().all(|&b| is_alnum_strict(b)) {
        buf.push_str(s);
    } else {
        buf.push('"');
        let mut start = 0;
        let mut ix = start;
        while ix < s.len() {
            let b = s.as_bytes()[ix];
            match b {
                b'"' | b'\\' => {
                    buf.push_str(&s[start..ix]);
                    buf.push('\\');
                    start = ix;
                }
                _ => (),
            }
            ix += 1;
        }
        buf.push_str(&s[start..]);
        buf.push('"');
    }
}

impl Plist {
    pub fn parse(s: &str) -> Result<Plist, Error> {
        let (plist, _ix) = Plist::parse_rec(s, 0)?;
        // TODO: check that we're actually at eof
        Ok(plist)
    }

    #[allow(unused)]
    pub fn as_dict(&self) -> Option<&HashMap<String, Plist>> {
        match self {
            Plist::Dictionary(d) => Some(d),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn get(&self, key: &str) -> Option<&Plist> {
        match self {
            Plist::Dictionary(d) => d.get(key),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_array(&self) -> Option<&[Plist]> {
        match self {
            Plist::Array(a) => Some(a),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Plist::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Plist::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Plist::Integer(i) => Some(*i as f64),
            Plist::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Plist::String(s) => s,
            _ => panic!("expected string"),
        }
    }

    pub fn into_vec(self) -> Vec<Plist> {
        match self {
            Plist::Array(a) => a,
            _ => panic!("expected array"),
        }
    }

    pub fn into_hashmap(self) -> HashMap<String, Plist> {
        match self {
            Plist::Dictionary(d) => d,
            _ => panic!("expected dictionary"),
        }
    }

    fn parse_rec(s: &str, ix: usize) -> Result<(Plist, usize), Error> {
        let (tok, mut ix) = Token::lex(s, ix)?;
        match tok {
            Token::Atom(s) => Ok((Plist::parse_atom(s), ix)),
            Token::String(s) => Ok((Plist::String(s.into()), ix)),
            Token::OpenBrace => {
                let mut dict = HashMap::new();
                loop {
                    if let Some(ix) = Token::expect(s, ix, b'}') {
                        return Ok((Plist::Dictionary(dict), ix));
                    }
                    let (key, next) = Token::lex(s, ix)?;
                    let key_str = Token::try_into_string(key)?;
                    let next = Token::expect(s, next, b'=');
                    if next.is_none() {
                        return Err(Error::ExpectedEquals);
                    }
                    let (val, next) = Self::parse_rec(s, next.unwrap())?;
                    dict.insert(key_str, val);
                    if let Some(next) = Token::expect(s, next, b';') {
                        ix = next;
                    } else {
                        return Err(Error::ExpectedSemicolon);
                    }
                }
            }
            Token::OpenParen => {
                let mut list = Vec::new();
                if let Some(ix) = Token::expect(s, ix, b')') {
                    return Ok((Plist::Array(list), ix));
                }
                loop {
                    let (val, next) = Self::parse_rec(s, ix)?;
                    list.push(val);
                    if let Some(ix) = Token::expect(s, next, b')') {
                        return Ok((Plist::Array(list), ix));
                    }
                    if let Some(next) = Token::expect(s, next, b',') {
                        ix = next;
                    } else {
                        return Err(Error::ExpectedComma);
                    }
                }
            }
            _ => Err(Error::SomethingWentWrong),
        }
    }

    fn parse_atom(s: &str) -> Plist {
        if numeric_ok(s) {
            if let Ok(num) = s.parse() {
                return Plist::Integer(num);
            }
            if let Ok(num) = s.parse() {
                return Plist::Float(num);
            }
        }
        Plist::String(s.into())
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        self.push_to_string(&mut s);
        s
    }

    fn push_to_string(&self, s: &mut String) {
        match self {
            Plist::Array(a) => {
                s.push_str("(");
                let mut delim = "\n";
                for el in a {
                    s.push_str(delim);
                    el.push_to_string(s);
                    delim = ",\n";
                }
                s.push_str("\n)");
            }
            Plist::Dictionary(a) => {
                s.push_str("{\n");
                let mut keys: Vec<_> = a.keys().collect();
                keys.sort();
                for k in keys {
                    let el = &a[k];
                    // TODO: quote if needed?
                    escape_string(s, k);
                    s.push_str(" = ");
                    el.push_to_string(s);
                    s.push_str(";\n");
                }
                s.push_str("}");
            }
            Plist::String(st) => escape_string(s, st),
            Plist::Integer(i) => {
                s.push_str(&format!("{}", i));
            }
            Plist::Float(f) => {
                s.push_str(&format!("{}", f));
            }
        }
    }
}

impl<'a> Token<'a> {
    fn lex(s: &'a str, ix: usize) -> Result<(Token<'a>, usize), Error> {
        let start = skip_ws(s, ix);
        if start == s.len() {
            return Ok((Token::Eof, start));
        }
        let b = s.as_bytes()[start];
        match b {
            b'{' => Ok((Token::OpenBrace, start + 1)),
            b'(' => Ok((Token::OpenParen, start + 1)),
            b'"' => {
                let mut ix = start + 1;
                let mut cow_start = ix;
                let mut buf = String::new();
                while ix < s.len() {
                    let b = s.as_bytes()[ix];
                    match b {
                        b'"' => {
                            // End of string
                            let string = if buf.is_empty() {
                                s[cow_start..ix].into()
                            } else {
                                buf.push_str(&s[cow_start..ix]);
                                buf.into()
                            };
                            return Ok((Token::String(string), ix + 1));
                        }
                        b'\\' => {
                            buf.push_str(&s[cow_start..ix]);
                            ix += 1;
                            if ix == s.len() {
                                return Err(Error::UnclosedString);
                            }
                            let b = s.as_bytes()[ix];
                            match b {
                                b'"' | b'\\' => cow_start = ix,
                                b'n' => {
                                    buf.push('\n');
                                    cow_start = ix + 1;
                                }
                                b'r' => {
                                    buf.push('\r');
                                    cow_start = ix + 1;
                                }
                                _ => {
                                    if b >= b'0' && b <= b'3' && ix + 2 < s.len() {
                                        // octal escape
                                        let b1 = s.as_bytes()[ix + 1];
                                        let b2 = s.as_bytes()[ix + 2];
                                        if b1 >= b'0' && b1 <= b'7' && b2 >= b'0' && b2 <= b'7' {
                                            let oct =
                                                (b - b'0') * 64 + (b1 - b'0') * 8 + (b2 - b'0');
                                            buf.push(oct as char);
                                            ix += 2;
                                            cow_start = ix + 1;
                                        } else {
                                            return Err(Error::UnknownEscape);
                                        }
                                    } else {
                                        return Err(Error::UnknownEscape);
                                    }
                                }
                            }
                            ix += 1;
                        }
                        _ => ix += 1,
                    }
                }
                Err(Error::UnclosedString)
            }
            _ => {
                if is_alnum(b) {
                    let mut ix = start + 1;
                    while ix < s.len() {
                        if !is_alnum(s.as_bytes()[ix]) {
                            break;
                        }
                        ix += 1;
                    }
                    Ok((Token::Atom(&s[start..ix]), ix))
                } else {
                    Err(Error::UnexpectedChar(s[start..].chars().next().unwrap()))
                }
            }
        }
    }

    fn try_into_string(self) -> Result<String, Error> {
        match self {
            Token::Atom(s) => Ok(s.into()),
            Token::String(s) => Ok(s.into()),
            _ => Err(Error::NotAString),
        }
    }

    fn expect(s: &str, ix: usize, delim: u8) -> Option<usize> {
        let ix = skip_ws(s, ix);
        if ix < s.len() {
            let b = s.as_bytes()[ix];
            if b == delim {
                return Some(ix + 1);
            }
        }
        None
    }
}

impl From<String> for Plist {
    fn from(x: String) -> Plist {
        Plist::String(x)
    }
}

impl From<i64> for Plist {
    fn from(x: i64) -> Plist {
        Plist::Integer(x)
    }
}

impl From<f64> for Plist {
    fn from(x: f64) -> Plist {
        Plist::Float(x)
    }
}

impl From<Vec<Plist>> for Plist {
    fn from(x: Vec<Plist>) -> Plist {
        Plist::Array(x)
    }
}

impl From<HashMap<String, Plist>> for Plist {
    fn from(x: HashMap<String, Plist>) -> Plist {
        Plist::Dictionary(x)
    }
}
