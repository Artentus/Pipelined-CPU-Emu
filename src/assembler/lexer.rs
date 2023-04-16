use super::SharedStr;
use cow_utils::CowUtils;
use langbox::*;
use std::fmt;
use std::num::ParseIntError;
use std::str::CharIndices;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PunctuationKind {
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `=`
    EqualSign,
    /// `+`
    PlusSign,
    /// `-`
    MinusSign,
    /// `*`
    Asterisk,
    /// `/`
    Slash,
    /// `%`
    PercentSign,
    /// `!`
    ExclamationMark,
    /// `&`
    Ampersand,
    /// `|`
    VerticalBar,
    /// `^`
    Accent,
    /// `<<`
    DoubleLessThanSign,
    /// `>>>`
    TrippleGreaterThanSign,
    /// `>>`
    DoubleGreaterThanSign,
    /// `(`
    OpeningParenthesis,
    /// `)`
    ClosingParenthesis,
    /// `[`
    OpeningBracket,
    /// `]`
    ClosingBracket,
}

impl fmt::Display for PunctuationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::EqualSign => write!(f, "="),
            Self::PlusSign => write!(f, "+"),
            Self::MinusSign => write!(f, "-"),
            Self::Asterisk => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::PercentSign => write!(f, "%"),
            Self::ExclamationMark => write!(f, "!"),
            Self::Ampersand => write!(f, "&"),
            Self::VerticalBar => write!(f, "|"),
            Self::Accent => write!(f, "^"),
            Self::DoubleLessThanSign => write!(f, "<<"),
            Self::TrippleGreaterThanSign => write!(f, ">>>"),
            Self::DoubleGreaterThanSign => write!(f, ">>"),
            Self::OpeningParenthesis => write!(f, "("),
            Self::ClosingParenthesis => write!(f, ")"),
            Self::OpeningBracket => write!(f, "["),
            Self::ClosingBracket => write!(f, "]"),
        }
    }
}

#[rustfmt::skip]
const PUNCTUATION_MAP: &[(&str, PunctuationKind)] = &[
    (","  , PunctuationKind::Comma                 ),
    (":"  , PunctuationKind::Colon                 ),
    ("="  , PunctuationKind::EqualSign             ),
    ("+"  , PunctuationKind::PlusSign              ),
    ("-"  , PunctuationKind::MinusSign             ),
    ("*"  , PunctuationKind::Asterisk              ),
    ("/"  , PunctuationKind::Slash                 ),
    ("%"  , PunctuationKind::PercentSign           ),
    ("!"  , PunctuationKind::ExclamationMark       ),
    ("&"  , PunctuationKind::Ampersand             ),
    ("|"  , PunctuationKind::VerticalBar           ),
    ("^"  , PunctuationKind::Accent                ),
    ("<<" , PunctuationKind::DoubleLessThanSign    ),
    (">>>", PunctuationKind::TrippleGreaterThanSign),
    (">>" , PunctuationKind::DoubleGreaterThanSign ),
    ("("  , PunctuationKind::OpeningParenthesis    ),
    (")"  , PunctuationKind::ClosingParenthesis    ),
    ("["  , PunctuationKind::OpeningBracket        ),
    ("]"  , PunctuationKind::ClosingBracket        ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveKind {
    Offset,
    Align,
    Origin,
    Section,
    Include,
}

impl fmt::Display for DirectiveKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offset => write!(f, ".offset"),
            Self::Align => write!(f, ".align"),
            Self::Origin => write!(f, ".origin"),
            Self::Section => write!(f, ".section"),
            Self::Include => write!(f, ".include"),
        }
    }
}

#[rustfmt::skip]
const DIRECTIVE_MAP: &[(&str, DirectiveKind)] = &[
    ("offset" , DirectiveKind::Offset ),
    ("align"  , DirectiveKind::Align  ),
    ("origin" , DirectiveKind::Origin ),
    ("section", DirectiveKind::Section),
    ("include", DirectiveKind::Include),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterKind {
    A,
    B,
    C,
    D,
    TL,
    TH,
    SI,
    DI,
    TX,
    AB,
    CD,
    RA,
    SP,
}

impl fmt::Display for RegisterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => write!(f, "a"),
            Self::B => write!(f, "b"),
            Self::C => write!(f, "v"),
            Self::D => write!(f, "d"),
            Self::TL => write!(f, "tl"),
            Self::TH => write!(f, "th"),
            Self::SI => write!(f, "si"),
            Self::DI => write!(f, "di"),
            Self::TX => write!(f, "tx"),
            Self::AB => write!(f, "ab"),
            Self::CD => write!(f, "cd"),
            Self::RA => write!(f, "ra"),
            Self::SP => write!(f, "sp"),
        }
    }
}

#[rustfmt::skip]
const REGISTER_MAP: &[(&str, RegisterKind)] = &[
    ("a" , RegisterKind::A ),
    ("b" , RegisterKind::B ),
    ("c" , RegisterKind::C ),
    ("d" , RegisterKind::D ),
    ("tl", RegisterKind::TL),
    ("th", RegisterKind::TH),
    ("si", RegisterKind::SI),
    ("di", RegisterKind::DI),
    ("tx", RegisterKind::TX),
    ("ab", RegisterKind::AB),
    ("cd", RegisterKind::CD),
    ("ra", RegisterKind::RA),
    ("sp", RegisterKind::SP),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoRegisterKind {
    UartData,
    UartControl,
    AudioData,
    ControllerData,
    VgaStatus,
    Gpio,
}

impl fmt::Display for IoRegisterKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UartData => write!(f, "uart_data"),
            Self::UartControl => write!(f, "uart_ctrl"),
            Self::AudioData => write!(f, "audio_data"),
            Self::ControllerData => write!(f, "cntrl_data"),
            Self::VgaStatus => write!(f, "vga"),
            Self::Gpio => write!(f, "gpio"),
        }
    }
}

#[rustfmt::skip]
const IO_REGISTER_MAP: &[(&str, IoRegisterKind)] = &[
    ("uart_data" , IoRegisterKind::UartData      ),
    ("uart_ctrl" , IoRegisterKind::UartControl   ),
    ("audio_data", IoRegisterKind::AudioData     ),
    ("cntrl_data", IoRegisterKind::ControllerData),
    ("vga"       , IoRegisterKind::VgaStatus     ),
    ("gpio"      , IoRegisterKind::Gpio          ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MnemonicKind {
    Nop,
    Cnop,
    Mov,
    Inc,
    Incc,
    Dec,
    In,
    Out,
    Break,
    Lodsb,
    Stosb,
    Call,
    Ret,
    CallBd,
    RetBd,
    Jmp,
    Jo,
    Jno,
    Js,
    Jns,
    Jz,
    Jnz,
    Je,
    Jne,
    Jc,
    Jnc,
    Jnae,
    Jb,
    Jae,
    Jnb,
    Jbe,
    Jna,
    Ja,
    Jnbe,
    Jl,
    Jnge,
    Jge,
    Jnl,
    Jle,
    Jng,
    Jg,
    Jnle,
    Jlc,
    Jnlc,
    Push,
    Pop,
    Clc,
    Shl,
    Shr,
    Add,
    Addc,
    Addac,
    Sub,
    Subb,
    Subae,
    And,
    Or,
    Xor,
    Not,
    Cmp,
    Test,
}

impl fmt::Display for MnemonicKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nop => write!(f, "nop"),
            Self::Cnop => write!(f, "cnop"),
            Self::Mov => write!(f, "mov"),
            Self::Inc => write!(f, "inc"),
            Self::Incc => write!(f, "incc"),
            Self::Dec => write!(f, "dec"),
            Self::In => write!(f, "in"),
            Self::Out => write!(f, "out"),
            Self::Break => write!(f, "break"),
            Self::Lodsb => write!(f, "lodsb"),
            Self::Stosb => write!(f, "stosb"),
            Self::Call => write!(f, "call"),
            Self::Ret => write!(f, "ret"),
            Self::CallBd => write!(f, "callbd"),
            Self::RetBd => write!(f, "retbd"),
            Self::Jmp => write!(f, "jmp"),
            Self::Jo => write!(f, "jo"),
            Self::Jno => write!(f, "jno"),
            Self::Js => write!(f, "js"),
            Self::Jns => write!(f, "jns"),
            Self::Jz => write!(f, "jz"),
            Self::Jnz => write!(f, "jnz"),
            Self::Je => write!(f, "je"),
            Self::Jne => write!(f, "jne"),
            Self::Jc => write!(f, "jc"),
            Self::Jnc => write!(f, "jnc"),
            Self::Jnae => write!(f, "jnae"),
            Self::Jb => write!(f, "jb"),
            Self::Jae => write!(f, "jae"),
            Self::Jnb => write!(f, "jnb"),
            Self::Jbe => write!(f, "jbe"),
            Self::Jna => write!(f, "jna"),
            Self::Ja => write!(f, "ja"),
            Self::Jnbe => write!(f, "jnbe"),
            Self::Jl => write!(f, "jl"),
            Self::Jnge => write!(f, "jnge"),
            Self::Jge => write!(f, "jge"),
            Self::Jnl => write!(f, "jnl"),
            Self::Jle => write!(f, "jle"),
            Self::Jng => write!(f, "jng"),
            Self::Jg => write!(f, "jg"),
            Self::Jnle => write!(f, "jnle"),
            Self::Jlc => write!(f, "jlc"),
            Self::Jnlc => write!(f, "jnlc"),
            Self::Push => write!(f, "push"),
            Self::Pop => write!(f, "pop"),
            Self::Clc => write!(f, "clc"),
            Self::Shl => write!(f, "shl"),
            Self::Shr => write!(f, "shr"),
            Self::Add => write!(f, "add"),
            Self::Addc => write!(f, "addc"),
            Self::Addac => write!(f, "addac"),
            Self::Sub => write!(f, "sub"),
            Self::Subb => write!(f, "subb"),
            Self::Subae => write!(f, "subae"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Xor => write!(f, "xor"),
            Self::Not => write!(f, "not"),
            Self::Cmp => write!(f, "cmp"),
            Self::Test => write!(f, "test"),
        }
    }
}

#[rustfmt::skip]
const MNEMONIC_MAP: &[(&str, MnemonicKind)] = &[
    ("nop"   , MnemonicKind::Nop   ),
    ("cnop"  , MnemonicKind::Cnop  ),
    ("mov"   , MnemonicKind::Mov   ),
    ("inc"   , MnemonicKind::Inc   ),
    ("incc"  , MnemonicKind::Incc  ),
    ("dec"   , MnemonicKind::Dec   ),
    ("in"    , MnemonicKind::In    ),
    ("out"   , MnemonicKind::Out   ),
    ("break" , MnemonicKind::Break ),
    ("lodsb" , MnemonicKind::Lodsb ),
    ("stosb" , MnemonicKind::Stosb ),
    ("call"  , MnemonicKind::Call  ),
    ("ret"   , MnemonicKind::Ret   ),
    ("callbd", MnemonicKind::CallBd),
    ("retbd" , MnemonicKind::RetBd ),
    ("jmp"   , MnemonicKind::Jmp   ),
    ("jo"    , MnemonicKind::Jo    ),
    ("jno"   , MnemonicKind::Jno   ),
    ("js"    , MnemonicKind::Js    ),
    ("jns"   , MnemonicKind::Jns   ),
    ("jz"    , MnemonicKind::Jz    ),
    ("jnz"   , MnemonicKind::Jnz   ),
    ("je"    , MnemonicKind::Je    ),
    ("jne"   , MnemonicKind::Jne   ),
    ("jc"    , MnemonicKind::Jc    ),
    ("jnc"   , MnemonicKind::Jnc   ),
    ("jnae"  , MnemonicKind::Jnae  ),
    ("jb"    , MnemonicKind::Jb    ),
    ("jae"   , MnemonicKind::Jae   ),
    ("jnb"   , MnemonicKind::Jnb   ),
    ("jbe"   , MnemonicKind::Jbe   ),
    ("jna"   , MnemonicKind::Jna   ),
    ("ja"    , MnemonicKind::Ja    ),
    ("jnbe"  , MnemonicKind::Jnbe  ),
    ("jl"    , MnemonicKind::Jl    ),
    ("jnge"  , MnemonicKind::Jnge  ),
    ("jge"   , MnemonicKind::Jge   ),
    ("jnl"   , MnemonicKind::Jnl   ),
    ("jle"   , MnemonicKind::Jle   ),
    ("jng"   , MnemonicKind::Jng   ),
    ("jg"    , MnemonicKind::Jg    ),
    ("jnle"  , MnemonicKind::Jnle  ),
    ("jlc"   , MnemonicKind::Jlc   ),
    ("jnlc"  , MnemonicKind::Jnlc  ),
    ("push"  , MnemonicKind::Push  ),
    ("pop"   , MnemonicKind::Pop   ),
    ("clc"   , MnemonicKind::Clc   ),
    ("shl"   , MnemonicKind::Shl   ),
    ("shr"   , MnemonicKind::Shr   ),
    ("add"   , MnemonicKind::Add   ),
    ("addc"  , MnemonicKind::Addc  ),
    ("addac" , MnemonicKind::Addac ),
    ("sub"   , MnemonicKind::Sub   ),
    ("subb"  , MnemonicKind::Subb  ),
    ("subae" , MnemonicKind::Subae ),
    ("and"   , MnemonicKind::And   ),
    ("or"    , MnemonicKind::Or    ),
    ("xor"   , MnemonicKind::Xor   ),
    ("not"   , MnemonicKind::Not   ),
    ("cmp"   , MnemonicKind::Cmp   ),
    ("test"  , MnemonicKind::Test  ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseStringError {
    MissingClosingQuote,
    InvalidEscapeSequence(std::ops::Range<usize>),
}

#[derive(Debug, Clone)]
pub enum Jam1Token {
    NewLine,
    Comment,
    Punctuation(PunctuationKind),
    Directive(DirectiveKind),
    Register(RegisterKind),
    IoRegister(IoRegisterKind),
    Mnemonic(MnemonicKind),
    Identifier(SharedStr),
    IntegerLiteral(i64),
    StringLiteral(SharedStr),
    InvalidDirective(SharedStr),
    InvalidIntegerLiteral(ParseIntError),
    InvalidStringLiteral(Box<[ParseStringError]>),
    InvalidChar(char),
}

fn read_comment_token(text: &str) -> Option<ReadTokenResult<Jam1Token>> {
    if let Some(text) = text.strip_prefix("//") {
        let end = text.find('\n').unwrap_or(text.len());
        Some(ReadTokenResult {
            token: Jam1Token::Comment,
            consumed_bytes: end + "//".len(),
        })
    } else if let Some(text) = text.strip_prefix(";") {
        let end = text.find('\n').unwrap_or(text.len());
        Some(ReadTokenResult {
            token: Jam1Token::Comment,
            consumed_bytes: end + ";".len(),
        })
    } else {
        None
    }
}

fn read_punctuation_token(text: &str) -> Option<ReadTokenResult<Jam1Token>> {
    for &(pattern, punctuation) in PUNCTUATION_MAP {
        if text.starts_with(pattern) {
            return Some(ReadTokenResult {
                token: Jam1Token::Punctuation(punctuation),
                consumed_bytes: pattern.len(),
            });
        }
    }

    None
}

fn read_directive_token(text: &str) -> Option<ReadTokenResult<Jam1Token>> {
    let mut chars = text.chars();
    let first_char = chars.next().expect("text was empty");
    if first_char == '.' {
        let mut consumed = '.'.len_utf8();
        for char in chars {
            if matches!(char, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_') {
                consumed += char.len_utf8();
            } else {
                break;
            }
        }

        let identifier = &text['.'.len_utf8()..consumed];
        for &(pattern, directive) in DIRECTIVE_MAP {
            if identifier.eq(pattern) {
                return Some(ReadTokenResult {
                    token: Jam1Token::Directive(directive),
                    consumed_bytes: consumed,
                });
            }
        }

        Some(ReadTokenResult {
            token: Jam1Token::InvalidDirective(identifier.into()),
            consumed_bytes: consumed,
        })
    } else {
        None
    }
}

fn read_identifier_token(text: &str) -> Option<ReadTokenResult<Jam1Token>> {
    let mut chars = text.chars();
    let first_char = chars.next().expect("text was empty");
    if matches!(first_char, 'A'..='Z' | 'a'..='z' | '_') {
        let mut consumed = first_char.len_utf8();
        for char in chars {
            if matches!(char, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_') {
                consumed += char.len_utf8();
            } else {
                break;
            }
        }

        let identifier = &text[..consumed];
        let lowercase = identifier.cow_to_ascii_lowercase();

        for &(pattern, register) in REGISTER_MAP {
            if pattern.eq(lowercase.as_ref()) {
                return Some(ReadTokenResult {
                    token: Jam1Token::Register(register),
                    consumed_bytes: consumed,
                });
            }
        }

        for &(pattern, io_register) in IO_REGISTER_MAP {
            if pattern.eq(lowercase.as_ref()) {
                return Some(ReadTokenResult {
                    token: Jam1Token::IoRegister(io_register),
                    consumed_bytes: consumed,
                });
            }
        }

        for &(pattern, mnemonic) in MNEMONIC_MAP {
            if pattern.eq(lowercase.as_ref()) {
                return Some(ReadTokenResult {
                    token: Jam1Token::Mnemonic(mnemonic),
                    consumed_bytes: consumed,
                });
            }
        }

        Some(ReadTokenResult {
            token: Jam1Token::Identifier(identifier.into()),
            consumed_bytes: consumed,
        })
    } else {
        None
    }
}

fn read_integer_literal_token(text: &str) -> Option<ReadTokenResult<Jam1Token>> {
    let mut chars = text.chars();
    let first_char = chars.next().expect("text was empty");
    if matches!(first_char, '0'..='9') {
        let mut consumed = first_char.len_utf8();
        for char in chars {
            if matches!(char, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_') {
                consumed += char.len_utf8();
            } else {
                break;
            }
        }

        let raw_literal = &text[..consumed];
        let (raw_literal, radix) = {
            if let Some(raw_literal) = raw_literal.strip_prefix("0x") {
                (raw_literal, 16)
            } else if let Some(raw_literal) = raw_literal.strip_prefix("0X") {
                (raw_literal, 16)
            } else if let Some(raw_literal) = raw_literal.strip_prefix("0o") {
                (raw_literal, 8)
            } else if let Some(raw_literal) = raw_literal.strip_prefix("0O") {
                (raw_literal, 8)
            } else if let Some(raw_literal) = raw_literal.strip_prefix("0b") {
                (raw_literal, 2)
            } else if let Some(raw_literal) = raw_literal.strip_prefix("0B") {
                (raw_literal, 2)
            } else {
                (raw_literal, 10)
            }
        };

        let raw_literal = raw_literal.cow_replace('_', "");
        match i64::from_str_radix(raw_literal.as_ref(), radix) {
            Ok(literal) => Some(ReadTokenResult {
                token: Jam1Token::IntegerLiteral(literal),
                consumed_bytes: consumed,
            }),
            Err(err) => Some(ReadTokenResult {
                token: Jam1Token::InvalidIntegerLiteral(err),
                consumed_bytes: consumed,
            }),
        }
    } else {
        None
    }
}

fn process_escape_sequence(
    chars: &mut CharIndices,
    literal: &mut String,
) -> Result<(), ParseStringError> {
    let (index, escape_char) = chars.next().ok_or(ParseStringError::MissingClosingQuote)?;

    match escape_char {
        'r' => literal.push('\r'),
        'n' => literal.push('\n'),
        't' => literal.push('\t'),
        '0' => literal.push('\0'),
        '"' => literal.push('"'),
        '\\' => literal.push('\\'),
        'x' => {
            let [(_, d1), (_, d2)] = chars
                .next_chunk::<2>()
                .map_err(|_| ParseStringError::MissingClosingQuote)?;

            let mut buffer = [0u8; 8];
            let mut buffer_len = 0;
            buffer_len += d1.encode_utf8(&mut buffer[buffer_len..]).len();
            buffer_len += d2.encode_utf8(&mut buffer[buffer_len..]).len();

            let raw_val = &buffer[..buffer_len];
            let raw_val = unsafe { std::str::from_utf8_unchecked(raw_val) };

            match u8::from_str_radix(raw_val, 16) {
                Ok(val) => {
                    let char = char::from_u32(val as u32).expect("invalid char code");
                    literal.push(char);
                }
                Err(_) => {
                    let range = (index - '\\'.len_utf8())..(index + 'x'.len_utf8() + buffer_len);
                    return Err(ParseStringError::InvalidEscapeSequence(range));
                }
            }
        }
        'u' => {
            let [(_, d1), (_, d2), (_, d3), (_, d4)] = chars
                .next_chunk::<4>()
                .map_err(|_| ParseStringError::MissingClosingQuote)?;

            let mut buffer = [0u8; 16];
            let mut buffer_len = 0;
            buffer_len += d1.encode_utf8(&mut buffer[buffer_len..]).len();
            buffer_len += d2.encode_utf8(&mut buffer[buffer_len..]).len();
            buffer_len += d3.encode_utf8(&mut buffer[buffer_len..]).len();
            buffer_len += d4.encode_utf8(&mut buffer[buffer_len..]).len();

            let raw_val = &buffer[..buffer_len];
            let raw_val = unsafe { std::str::from_utf8_unchecked(raw_val) };

            match u16::from_str_radix(raw_val, 16) {
                Ok(val) => {
                    let char = char::from_u32(val as u32).expect("invalid char code");
                    literal.push(char);
                }
                Err(_) => {
                    let range = (index - '\\'.len_utf8())..(index + 'u'.len_utf8() + buffer_len);
                    return Err(ParseStringError::InvalidEscapeSequence(range));
                }
            }
        }
        '\n' => {
            return Err(ParseStringError::MissingClosingQuote);
        }
        invalid_char => {
            let range = (index - '\\'.len_utf8())..(index + invalid_char.len_utf8());
            return Err(ParseStringError::InvalidEscapeSequence(range));
        }
    }

    Ok(())
}

fn read_string_literal_token(text: &str) -> Option<ReadTokenResult<Jam1Token>> {
    let mut chars = text.char_indices();
    let (_, first_char) = chars.next().expect("text was empty");

    if first_char == '"' {
        let mut literal = String::new();
        let mut errors = Vec::new();

        while let Some((index, char)) = chars.next() {
            match char {
                '\\' => {
                    if let Err(err) = process_escape_sequence(&mut chars, &mut literal) {
                        errors.push(err.clone());

                        if err == ParseStringError::MissingClosingQuote {
                            return Some(ReadTokenResult {
                                token: Jam1Token::InvalidStringLiteral(errors.into_boxed_slice()),
                                consumed_bytes: index + '\\'.len_utf8(),
                            });
                        }
                    }
                }
                '"' => {
                    let consumed = index + '"'.len_utf8();

                    if errors.is_empty() {
                        return Some(ReadTokenResult {
                            token: Jam1Token::StringLiteral(literal.into()),
                            consumed_bytes: consumed,
                        });
                    } else {
                        return Some(ReadTokenResult {
                            token: Jam1Token::InvalidStringLiteral(errors.into_boxed_slice()),
                            consumed_bytes: consumed,
                        });
                    }
                }
                '\n' => {
                    errors.push(ParseStringError::MissingClosingQuote);
                    return Some(ReadTokenResult {
                        token: Jam1Token::InvalidStringLiteral(errors.into_boxed_slice()),
                        consumed_bytes: index,
                    });
                }
                char => {
                    literal.push(char);
                }
            }
        }

        errors.push(ParseStringError::MissingClosingQuote);
        Some(ReadTokenResult {
            token: Jam1Token::InvalidStringLiteral(errors.into_boxed_slice()),
            consumed_bytes: text.len(),
        })
    } else {
        None
    }
}

pub struct Jam1TokenReader;
impl TokenReader for Jam1TokenReader {
    type Token = Jam1Token;

    fn read_token(text: &str) -> ReadTokenResult<Self::Token> {
        if text.starts_with('\n') {
            return ReadTokenResult {
                token: Jam1Token::NewLine,
                consumed_bytes: '\n'.len_utf8(),
            };
        }

        if let Some(result) = read_comment_token(text) {
            return result;
        }

        if let Some(result) = read_punctuation_token(text) {
            return result;
        }

        if let Some(result) = read_directive_token(text) {
            return result;
        }

        if let Some(result) = read_identifier_token(text) {
            return result;
        }

        if let Some(result) = read_integer_literal_token(text) {
            return result;
        }

        if let Some(result) = read_string_literal_token(text) {
            return result;
        }

        let next_char = text.chars().next().expect("text was empty");
        ReadTokenResult {
            token: Jam1Token::InvalidChar(next_char),
            consumed_bytes: next_char.len_utf8(),
        }
    }
}

pub type Jam1Lexer<'a> = Lexer<'a, Jam1TokenReader, whitespace_mode::RemoveKeepNewLine>;
