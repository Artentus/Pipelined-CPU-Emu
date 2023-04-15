mod ast;
mod eval;
mod lexer;
mod parser;

use ast::*;
use eval::*;
use indexmap::IndexMap;
use langbox::*;
use lexer::*;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::ops::Range;
use std::rc::Rc;

type SharedStr = Rc<str>;

#[derive(Debug)]
pub enum AssemblerError {
    UnclosedBlockComment {
        comment: TextSpan,
    },
    InvalidDirective {
        directive: TextSpan,
    },
    InvalidIntegerLiteral {
        literal: TextSpan,
        error: ParseIntError,
    },
    UnclosedStringLiteral {
        literal: TextSpan,
    },
    InvalidEscapeSequence {
        literal: TextSpan,
        range: Range<usize>,
    },
    InvalidChars {
        span: TextSpan,
    },
    DuplicateSectionBase {
        value: TextSpan,
        previous: TextSpan,
    },
    DuplicateLabel {
        previous: TextSpan,
        duplicate: TextSpan,
    },
    SectionTooLarge {
        section: SharedStr,
    },
    InvalidValue {
        value: TextSpan,
        directive: TextSpan,
    },
    InvalidOriginDirective {
        directive: TextSpan,
    },
    UndefinedSection {
        statement: TextSpan,
    },
    OverlappingSections {
        first: SharedStr,
        second: SharedStr,
    },
    DivideByZero {
        expr: TextSpan,
    },
    UndefinedSymbol {
        ident: TextSpan,
    },
    CyclicExpression {
        expr: TextSpan,
    },
    IncludeError {
        directive: TextSpan,
        error: std::io::Error,
    },
    IncludeUnsupported {
        directive: TextSpan,
    },
    ParseError(parser::ParseError),
}

fn format_code_hint<W: std::fmt::Write>(
    mut writer: W,
    file_server: &FileServer,
    span: TextSpan,
    hint_color: &str,
    hint_range: Option<Range<usize>>,
) {
    const BOLD: &str = "\x1B\x5B1m";
    const REGULAR: &str = "\x1B\x5B22m";
    const CYAN: &str = "\x1B\x5B36m";
    const WHITE: &str = "\x1B\x5B39m";

    let (start_line, start_column) = span.start_pos().line_column(file_server);
    let (end_line, end_column) = span.end_pos().line_column(file_server);

    let file = file_server.get_file(span.file_id()).unwrap();
    let line = file.text().lines().nth(start_line as usize).unwrap();
    let line_number = format!("{}", start_line + 1);

    let (start_column, end_column) = if end_line == start_line {
        assert!(end_column >= start_column);
        (start_column as usize, end_column as usize)
    } else {
        (start_column as usize, line.chars().count())
    };

    let hint_range = hint_range.unwrap_or_else(|| 0..(end_column - start_column));

    write!(
        writer,
        "{BOLD}{CYAN}{:width$} |{WHITE}\r\n",
        "",
        width = line_number.len()
    )
    .unwrap();
    write!(writer, "{CYAN}{line_number} |{WHITE}{REGULAR}  {line}\r\n").unwrap();
    write!(
        writer,
        "{BOLD}{CYAN}{:width$} |{WHITE}  ",
        "",
        width = line_number.len()
    )
    .unwrap();
    write!(
        writer,
        "{:width$}",
        "",
        width = start_column + hint_range.start
    )
    .unwrap();
    write!(
        writer,
        "{hint_color}{:^>width$}{WHITE}\r\n",
        "",
        width = (hint_range.end - hint_range.start).max(1)
    )
    .unwrap();
    write!(
        writer,
        "{CYAN}{:width$} |{WHITE}{REGULAR}\r\n",
        "",
        width = line_number.len()
    )
    .unwrap();
}

impl AssemblerError {
    pub fn format(&self, file_server: &FileServer) -> String {
        use std::fmt::Write;

        const BOLD: &str = "\x1B\x5B1m";
        const REGULAR: &str = "\x1B\x5B22m";
        const RED: &str = "\x1B\x5B31m";
        const BLUE: &str = "\x1B\x5B34m";
        const WHITE: &str = "\x1B\x5B39m";

        let mut output = String::new();

        match self {
            &Self::UnclosedBlockComment { comment } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: block comment is not closed{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, comment, RED, None);
            }
            &Self::InvalidDirective { directive } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: unknown directive{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, directive, RED, None);
            }
            &Self::InvalidIntegerLiteral { literal, .. } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: literal contains invalid characters{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, literal, RED, None);
            }
            &Self::UnclosedStringLiteral { literal } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: literal is missing closing quotes{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, literal, RED, None);
            }
            Self::InvalidEscapeSequence { literal, range } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: unknown escape sequence{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, *literal, RED, Some(range.clone()));
            }
            &Self::InvalidChars { span } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: invalid characters in input{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, span, RED, None);
            }
            &Self::DuplicateSectionBase { value, previous } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: section base address is defined twice{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, value, RED, None);
                write!(output, "Previous definition:\r\n").unwrap();
                format_code_hint(&mut output, file_server, previous, BLUE, None);
            }
            &Self::DuplicateLabel {
                previous,
                duplicate,
            } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: symbol is defined twice{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, duplicate, RED, None);
                write!(output, "Previous definition:\r\n").unwrap();
                format_code_hint(&mut output, file_server, previous, BLUE, None);
            }
            Self::SectionTooLarge { section } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: section `{section}` is too large{REGULAR}\r\n"
                )
                .unwrap();
            }
            &Self::InvalidValue { value, .. } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: value is not valid for this directive{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, value, RED, None);
            }
            &Self::InvalidOriginDirective { directive } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: origin has already been defined{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, directive, RED, None);
            }
            &Self::UndefinedSection { statement } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: statement is only valid inside a section{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, statement, RED, None);
            }
            Self::OverlappingSections { first, second } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: sections `{first}` and `{second}` are overlapping{REGULAR}\r\n"
                )
                .unwrap();
            }
            &Self::DivideByZero { expr } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: divide by zero error while evaluating expression{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, expr, RED, None);
            }
            &Self::UndefinedSymbol { ident } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: symbol is not defined{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, ident, RED, None);
            }
            &Self::CyclicExpression { expr } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: expression cannot be evaluated due to cyclic dependencies{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, expr, RED, None);
            }
            Self::IncludeError { directive, error } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: failed to include file{REGULAR}\r\n"
                )
                .unwrap();
                write!(output, "{error}\r\n").unwrap();
                format_code_hint(&mut output, file_server, *directive, RED, None);
            }
            &Self::IncludeUnsupported { directive } => {
                write!(
                    output,
                    "{BOLD}{RED}Error{WHITE}: including files is not supported in this environment{REGULAR}\r\n"
                )
                .unwrap();
                format_code_hint(&mut output, file_server, directive, RED, None);
            }
            Self::ParseError(err) => match err {
                &parser::ParseError::UnexpectedToken { token, expected } => {
                    write!(output, "{BOLD}{RED}Error{WHITE}: expected {expected}\r\n").unwrap();
                    format_code_hint(&mut output, file_server, token, RED, None);
                }
                parser::ParseError::InvalidOperands { op1, op2 } => {
                    write!(
                        output,
                        "{BOLD}{RED}Error{WHITE}: instruction does not support this combination of operands{REGULAR}\r\n"
                    )
                    .unwrap();
                    format_code_hint(&mut output, file_server, op1.join(op2), RED, None);
                }
                &parser::ParseError::InvalidRegister { register } => {
                    write!(
                        output,
                        "{BOLD}{RED}Error{WHITE}: register is not supported by this instruction{REGULAR}\r\n"
                    )
                    .unwrap();
                    format_code_hint(&mut output, file_server, register, RED, None);
                }
                &parser::ParseError::TokensRemaining { span } => {
                    write!(output, "{BOLD}{RED}Error{WHITE}: unexpected tokens after complete statement{REGULAR}\r\n").unwrap();
                    format_code_hint(&mut output, file_server, span, RED, None);
                }
                &parser::ParseError::NoMatch { span } => {
                    write!(
                        output,
                        "{BOLD}{RED}Error{WHITE}: unknown statement{REGULAR}\r\n"
                    )
                    .unwrap();
                    format_code_hint(&mut output, file_server, span, RED, None);
                }
            },
        }

        output
    }
}

fn emit_lexer_errors(tokens: &[Token<Jam1Token>], errors: &mut Vec<AssemblerError>) -> bool {
    let mut tokens = tokens.into_iter().peekable();
    let mut can_parse = true;

    while let Some(token) = tokens.next() {
        match &token.kind {
            Jam1Token::InvalidDirective(_) => {
                errors.push(AssemblerError::InvalidDirective {
                    directive: token.span,
                });
                can_parse = false;
            }
            Jam1Token::InvalidIntegerLiteral(int_error) => {
                errors.push(AssemblerError::InvalidIntegerLiteral {
                    literal: token.span,
                    error: int_error.clone(),
                });
            }
            Jam1Token::InvalidStringLiteral(string_errors) => {
                for string_error in string_errors.as_ref() {
                    match string_error {
                        ParseStringError::MissingClosingQuote => {
                            errors.push(AssemblerError::UnclosedStringLiteral {
                                literal: token.span,
                            });
                        }
                        ParseStringError::InvalidEscapeSequence(range) => {
                            errors.push(AssemblerError::InvalidEscapeSequence {
                                literal: token.span,
                                range: range.clone(),
                            });
                        }
                    }
                }
            }
            Jam1Token::InvalidChar(_) => {
                let start = token.span;
                let mut end = start;

                while let Some(token) = tokens.peek() {
                    if let Jam1Token::InvalidChar(_) = token.kind {
                        end = token.span;
                        tokens.next();
                    } else {
                        break;
                    }
                }

                errors.push(AssemblerError::InvalidChars {
                    span: start.join(&end),
                });
                can_parse = false;
            }
            _ => {}
        }
    }

    can_parse
}

struct RawSection {
    base: Option<(u16, TextSpan)>,
    statements: Vec<Statement>,
}

impl Default for RawSection {
    #[inline]
    fn default() -> Self {
        Self {
            base: None,
            statements: Vec::new(),
        }
    }
}

fn process_file(
    file_server: &mut FileServer,
    file: FileId,
    errors: &mut Vec<AssemblerError>,
    sections: &mut IndexMap<SharedStr, RawSection>,
    label_set: &mut HashMap<SharedStr, TextSpan>,
    current_section: &mut Option<SharedStr>,
    default_base: &mut Option<u16>,
    allow_include: bool,
) {
    let mut statements = Vec::new();

    // Tokenize and parse
    let mut lexer = Jam1Lexer::new(file, &file_server);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next() {
        match &token.kind {
            Jam1Token::NewLine => {
                if !tokens.is_empty() {
                    if emit_lexer_errors(&tokens, errors) {
                        match parser::parse(TokenStream::new(&tokens)) {
                            Ok(statement) => {
                                statements.push(statement);
                            }
                            Err(err) => {
                                errors.push(AssemblerError::ParseError(err));
                            }
                        }
                    }

                    tokens.clear();
                }
            }
            Jam1Token::InvalidBlockComment => {
                errors.push(AssemblerError::UnclosedBlockComment {
                    comment: token.span,
                });
            }
            Jam1Token::Comment => {}
            _ => {
                tokens.push(token);
            }
        }
    }

    if !tokens.is_empty() {
        if emit_lexer_errors(&tokens, errors) {
            match parser::parse(TokenStream::new(&tokens)) {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(err) => {
                    errors.push(AssemblerError::ParseError(err));
                }
            }
        }
    }

    // Place statements into sections
    for statement in statements {
        match &statement {
            Statement::SectionDirective(directive) => {
                let current_section = current_section.insert(directive.name().value());
                *default_base = Some(0);

                if let Some(base) = directive.base().and_then(|base| base.value()) {
                    let section = sections
                        .entry(SharedStr::clone(current_section))
                        .or_default();

                    if let Some((_, previous)) = section.base {
                        errors.push(AssemblerError::DuplicateSectionBase {
                            value: directive.base().unwrap().span(),
                            previous,
                        });
                    } else {
                        section.base = Some((base as u16, directive.base().unwrap().span()));
                    }
                }
            }
            Statement::IncludeDirective(directive) => {
                if allow_include {
                    let rel_path = directive.path().value();
                    let file_path = file_server.get_file(file).unwrap().path();
                    let include_path = file_path
                        .parent()
                        .map(|parent| parent.join(rel_path.as_ref()))
                        .unwrap_or(rel_path.as_ref().into());

                    match file_server.register_file(&include_path) {
                        Ok(include_file) => {
                            process_file(
                                file_server,
                                include_file,
                                errors,
                                sections,
                                label_set,
                                current_section,
                                default_base,
                                allow_include,
                            );
                        }
                        Err(error) => {
                            errors.push(AssemblerError::IncludeError {
                                directive: directive.span(),
                                error,
                            });
                        }
                    }
                } else {
                    errors.push(AssemblerError::IncludeUnsupported {
                        directive: directive.span(),
                    });
                }
            }
            Statement::OriginDirective(directive) => {
                if default_base.is_none() {
                    match u16::try_from(directive.value().value().unwrap_or(0)) {
                        Ok(value) => {
                            *default_base = Some(value);
                        }
                        Err(_) => {
                            errors.push(AssemblerError::InvalidValue {
                                value: directive.value().span(),
                                directive: directive.span(),
                            });
                        }
                    }
                } else {
                    errors.push(AssemblerError::InvalidOriginDirective {
                        directive: directive.span(),
                    });
                }
            }
            _ => {
                if let Statement::Label(label) = &statement {
                    if let Some(previous) =
                        label_set.insert(label.name().name(), label.name().span())
                    {
                        errors.push(AssemblerError::DuplicateLabel {
                            previous,
                            duplicate: label.name().span(),
                        });
                    }
                }

                if let Some(current_section) = current_section {
                    sections
                        .entry(SharedStr::clone(current_section))
                        .or_default()
                        .statements
                        .push(statement);
                } else {
                    errors.push(AssemblerError::UndefinedSection {
                        statement: statement.span(),
                    });
                    break;
                }
            }
        }
    }
}

struct Section {
    name: SharedStr,
    base: u16,
    size: u16,
    statements: Vec<Statement>,
}

fn process_sections(
    sections: IndexMap<SharedStr, RawSection>,
    mut default_base: u16,
    errors: &mut Vec<AssemblerError>,
) -> Vec<Section> {
    // Find section sizes and base addresses
    let sections: Vec<_> = sections
        .into_iter()
        .map(|(name, section)| {
            let (base, update_default_base) = if let Some((base, _)) = section.base {
                (base, false)
            } else {
                (default_base, true)
            };

            let mut size = 0u16;
            let mut current_address = 0u16;
            for statement in &section.statements {
                match statement {
                    Statement::OffsetDirective(directive) => {
                        match u16::try_from(directive.value().value().unwrap_or(0)) {
                            Ok(value) => current_address = value,
                            Err(_) => {
                                errors.push(AssemblerError::InvalidValue {
                                    value: directive.value().span(),
                                    directive: directive.span(),
                                });
                            }
                        }
                    }
                    Statement::AlignDirective(directive) => {
                        match u16::try_from(directive.value().value().unwrap_or(1)) {
                            Ok(0) => {
                                errors.push(AssemblerError::InvalidValue {
                                    value: directive.value().span(),
                                    directive: directive.span(),
                                });
                            }
                            Ok(align) => {
                                current_address = current_address.div_ceil(align) * align;
                            }
                            Err(_) => {
                                errors.push(AssemblerError::InvalidValue {
                                    value: directive.value().span(),
                                    directive: directive.span(),
                                });
                            }
                        }
                    }
                    Statement::OriginDirective(_) => unreachable!(),
                    Statement::SectionDirective(_) => unreachable!(),
                    Statement::IncludeDirective(_) => unreachable!(),
                    Statement::Label(_) => {}
                    Statement::Instruction(_) => {}
                }

                match current_address.checked_add(statement.emit_size()) {
                    Some(new_address) => current_address = new_address,
                    None => {
                        errors.push(AssemblerError::SectionTooLarge {
                            section: SharedStr::clone(&name),
                        });
                        break;
                    }
                }

                size = size.max(current_address);
            }

            match base.checked_add(size) {
                Some(new_base) => {
                    if update_default_base {
                        default_base = new_base;
                    }
                }
                None => {
                    errors.push(AssemblerError::SectionTooLarge {
                        section: SharedStr::clone(&name),
                    });
                }
            }

            Section {
                name,
                base,
                size,
                statements: section.statements,
            }
        })
        .collect();

    // Check for overlapping sections
    for (i, first) in sections.iter().enumerate() {
        for second in sections.iter().skip(i + 1) {
            if (second.base >= first.base) && (second.base <= (first.base + first.size))
                || (((second.base + second.size) >= first.base)
                    && ((second.base + second.size) <= (first.base + first.size)))
            {
                errors.push(AssemblerError::OverlappingSections {
                    first: SharedStr::clone(&first.name),
                    second: SharedStr::clone(&second.name),
                });
            }
        }
    }

    sections
}

fn evaluate_labels(
    sections: &[Section],
    label_set: &HashMap<SharedStr, TextSpan>,
    errors: &mut Vec<AssemblerError>,
) -> HashMap<SharedStr, Option<i64>> {
    // Evaluate positional labels
    let mut label_values = HashMap::new();
    let mut label_expressions = Vec::new();
    for section in sections {
        let mut current_address = section.base;

        for statement in &section.statements {
            match statement {
                Statement::Label(label) => match label.value() {
                    LabelValue::Address { .. } => {
                        label_values.insert(label.name().name(), Some(current_address as i64));
                    }
                    LabelValue::Expression { value, .. } => {
                        label_expressions.push((label.name().name(), value));
                    }
                },
                Statement::OffsetDirective(directive) => {
                    current_address =
                        section.base + (directive.value().value().unwrap_or(0) as u16);
                }
                Statement::AlignDirective(directive) => {
                    let align = directive.value().value().unwrap_or(0) as u16;
                    if align > 0 {
                        current_address = current_address.div_ceil(align) * align;
                    }
                }
                Statement::OriginDirective(_) => unreachable!(),
                Statement::SectionDirective(_) => unreachable!(),
                Statement::IncludeDirective(_) => unreachable!(),
                Statement::Instruction(_) => {}
            }

            current_address += statement.emit_size();
        }
    }

    // Evaluate expression labels
    let mut last_evaluated_count = label_values.len();
    loop {
        for (label_name, label_expr) in &label_expressions {
            if !label_values.contains_key(label_name.as_ref()) {
                match label_expr.try_eval(&label_set, &label_values) {
                    Ok(value) => {
                        label_values.insert(SharedStr::clone(label_name), Some(value));
                    }
                    Err(EvalError::InvalidLiteralValue(_))
                    | Err(EvalError::ErrorInReferenceEval) => {
                        label_values.insert(SharedStr::clone(label_name), None);
                    }
                    Err(EvalError::DivideByZero(expr)) => {
                        errors.push(AssemblerError::DivideByZero { expr: expr.span() });
                        label_values.insert(SharedStr::clone(label_name), None);
                    }
                    Err(EvalError::UndefinedSymbol(ident)) => {
                        errors.push(AssemblerError::UndefinedSymbol {
                            ident: ident.span(),
                        });
                        label_values.insert(SharedStr::clone(label_name), None);
                    }
                    Err(EvalError::MissingReferenceValue) => {}
                }
            }
        }

        if last_evaluated_count < label_values.len() {
            last_evaluated_count = label_values.len();
        } else {
            break;
        }
    }

    // Check for label expressions that cannot be evaluated (cyclic references)
    for (label_name, label_expr) in &label_expressions {
        if !label_values.contains_key(label_name.as_ref()) {
            errors.push(AssemblerError::CyclicExpression {
                expr: label_expr.span(),
            });
        }
    }

    label_values
}

pub fn assemble(
    file_server: &mut FileServer,
    file: FileId,
    allow_include: bool,
) -> Result<(u16, Vec<u8>), Vec<AssemblerError>> {
    let mut errors = Vec::new();
    let mut sections = IndexMap::<SharedStr, RawSection>::new();

    let mut label_set = HashMap::new();
    let mut current_section = None;
    let mut default_base = None;
    process_file(
        file_server,
        file,
        &mut errors,
        &mut sections,
        &mut label_set,
        &mut current_section,
        &mut default_base,
        allow_include,
    );

    let mut sections = process_sections(sections, default_base.unwrap_or(0), &mut errors);
    let label_values = evaluate_labels(&sections, &label_set, &mut errors);

    if errors.is_empty() {
        if sections.is_empty() {
            Ok((0, Vec::new()))
        } else {
            sections.sort_by_key(|section| section.base);

            let first_section = sections.first().unwrap();
            let last_section = sections.last().unwrap();

            let start_address = first_section.base;
            let end_address = last_section.base + last_section.size;

            let mut data = vec![0u8; (end_address - start_address) as usize];
            let mut writer = std::io::Cursor::new(&mut data);

            for section in sections {
                writer.set_position((section.base - start_address) as u64);

                for statement in section.statements {
                    match statement {
                        Statement::Label(_) => {}
                        Statement::OffsetDirective(directive) => {
                            let offset = directive.value().value().unwrap() as u16;
                            writer.set_position((section.base - start_address + offset) as u64);
                        }
                        Statement::AlignDirective(directive) => {
                            let align = directive.value().value().unwrap() as u64;
                            if align > 0 {
                                writer.set_position(writer.position().div_ceil(align) * align);
                            }
                        }
                        Statement::OriginDirective(_) => unreachable!(),
                        Statement::SectionDirective(_) => unreachable!(),
                        Statement::IncludeDirective(_) => unreachable!(),
                        Statement::Instruction(instruction) => {
                            instruction
                                .encode(&mut writer, &label_set, &label_values, &mut errors)
                                .expect("writing to an in-memory buffer");
                        }
                    }
                }
            }

            if errors.is_empty() {
                Ok((start_address, data))
            } else {
                Err(errors)
            }
        }
    } else {
        Err(errors)
    }
}

pub fn assemble_code(code: &str, allow_include: bool) -> Result<(u16, Vec<u8>), String> {
    let code = code.replace('\t', "    ");

    let mut file_server = FileServer::new();
    let file = file_server.register_file_memory("<code>", code).unwrap();

    assemble(&mut file_server, file, allow_include).map_err(|errors| {
        let mut output = String::new();

        for (i, error) in errors.into_iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }

            output.push_str(&error.format(&file_server));
        }

        output
    })
}
