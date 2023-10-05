use super::ast::*;
use super::lexer::*;
use super::SharedStr;
use langbox::*;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        token: TextSpan,
        expected: &'static str,
    },
    InvalidOperands {
        op1: TextSpan,
        op2: TextSpan,
    },
    InvalidRegister {
        register: TextSpan,
    },
    TokensRemaining {
        span: TextSpan,
    },
    NoMatch {
        span: TextSpan,
    },
}

macro_rules! expect {
    ($expected:literal) => {
        |input| {
            let token_span = if let Some(token) = input.peek() {
                token.span
            } else {
                input.empty_span()
            };

            ParseError::UnexpectedToken {
                token: token_span,
                expected: $expected,
            }
        }
    };
}

trait Jam1Parser<T> = langbox::Parser<Jam1Token, T, ParseError>;

fn punctuation(list: &'static [PunctuationKind]) -> impl Jam1Parser<Punctuation> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let &Jam1Token::Punctuation(kind) = &token.kind {
                if list.contains(&kind) {
                    return ParseResult::Match {
                        value: Punctuation::new(kind, token.span),
                        span: token.span,
                        remaining: input.advance(),
                    };
                }
            }
        }

        ParseResult::NoMatch
    })
}

fn directive(kind: DirectiveKind) -> impl Jam1Parser<Directive> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let &Jam1Token::Directive(actual_kind) = &token.kind {
                if actual_kind == kind {
                    return ParseResult::Match {
                        value: Directive::new(kind, token.span),
                        span: token.span,
                        remaining: input.advance(),
                    };
                }
            }
        }

        ParseResult::NoMatch
    })
}

fn register() -> impl Jam1Parser<Register> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let &Jam1Token::Register(kind) = &token.kind {
                return ParseResult::Match {
                    value: Register::new(kind, token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            }
        }

        ParseResult::NoMatch
    })
}

fn io_register() -> impl Jam1Parser<IoRegister> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let &Jam1Token::IoRegister(kind) = &token.kind {
                return ParseResult::Match {
                    value: IoRegister::new(kind, token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            }
        }

        ParseResult::NoMatch
    })
}

fn mnemonic<const N: usize>(list: [MnemonicKind; N]) -> impl Jam1Parser<Mnemonic> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let &Jam1Token::Mnemonic(kind) = &token.kind {
                if list.contains(&kind) {
                    return ParseResult::Match {
                        value: Mnemonic::new(kind, token.span),
                        span: token.span,
                        remaining: input.advance(),
                    };
                }
            }
        }

        ParseResult::NoMatch
    })
}

fn identifier() -> impl Jam1Parser<Identifier> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let Jam1Token::Identifier(name) = &token.kind {
                return ParseResult::Match {
                    value: Identifier::new(SharedStr::clone(name), token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            }
        }

        ParseResult::NoMatch
    })
}

fn integer_literal() -> impl Jam1Parser<IntegerLiteral> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let &Jam1Token::IntegerLiteral(value) = &token.kind {
                return ParseResult::Match {
                    value: IntegerLiteral::new(Some(value), token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            } else if let Jam1Token::InvalidIntegerLiteral(_) = &token.kind {
                return ParseResult::Match {
                    value: IntegerLiteral::new(None, token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            }
        }

        ParseResult::NoMatch
    })
}

fn string_literal() -> impl Jam1Parser<StringLiteral> {
    parse_fn!(|input| {
        if let Some(token) = input.peek() {
            if let Jam1Token::StringLiteral(value) = &token.kind {
                return ParseResult::Match {
                    value: StringLiteral::new(SharedStr::clone(value), token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            } else if let Jam1Token::InvalidStringLiteral(_) = &token.kind {
                return ParseResult::Match {
                    value: StringLiteral::new("".into(), token.span),
                    span: token.span,
                    remaining: input.advance(),
                };
            }
        }

        ParseResult::NoMatch
    })
}

fn group_expression() -> impl Jam1Parser<GroupExpression> {
    let seq = sequence!(
        punctuation(&[PunctuationKind::OpeningParenthesis]),
        parser!({expression()}!![expect!("expression")]),
        parser!({punctuation(&[PunctuationKind::ClosingParenthesis])}!![expect!("expression or `)`")]),
    );

    parser!(seq->[|(open_paren, inner, close_paren)| GroupExpression::new(open_paren, inner, close_paren)])
}

fn leaf_expression() -> impl Jam1Parser<Expression> {
    choice!(
        parser!(({integer_literal()}->[Box::new])->[Expression::Literal]),
        parser!(({identifier()}->[Box::new])->[Expression::Identifier]),
        parser!(({group_expression()}->[Box::new])->[Expression::Group]),
    )
}

fn build_unary_expression_tree((ops, mut expr): (Vec<Punctuation>, Expression)) -> Expression {
    for op in ops.into_iter().rev() {
        let op_kind = op.kind();
        let unary_expr = Box::new(UnaryExpression::new(op, expr));

        expr = match op_kind {
            PunctuationKind::PlusSign => Expression::Identity(unary_expr),
            PunctuationKind::MinusSign => Expression::Negation(unary_expr),
            PunctuationKind::ExclamationMark => Expression::BitwiseNot(unary_expr),
            _ => unreachable!(),
        };
    }

    expr
}

fn unary_expression() -> impl Jam1Parser<Expression> {
    let op = punctuation(&[
        PunctuationKind::PlusSign,
        PunctuationKind::MinusSign,
        PunctuationKind::ExclamationMark,
    ]);

    parser!(
        (+op <.> {leaf_expression()}!![expect!("integer literal or `(`")])
            ->[build_unary_expression_tree]
        <|> {leaf_expression()}
    )
}

fn build_binary_expression_tree(
    (mut expr, tail): (Expression, Vec<(Punctuation, Expression)>),
) -> Expression {
    for (op, rhs) in tail {
        let op_kind = op.kind();
        let binary_expr = Box::new(BinaryExpression::new(expr, op, rhs));

        expr = match op_kind {
            PunctuationKind::PlusSign => Expression::Addition(binary_expr),
            PunctuationKind::MinusSign => Expression::Subtraction(binary_expr),
            PunctuationKind::Asterisk => Expression::Multiplication(binary_expr),
            PunctuationKind::Slash => Expression::Division(binary_expr),
            PunctuationKind::PercentSign => Expression::Remainder(binary_expr),
            PunctuationKind::Ampersand => Expression::BitwiseAnd(binary_expr),
            PunctuationKind::VerticalBar => Expression::BitwiseOr(binary_expr),
            PunctuationKind::Accent => Expression::BitwiseXor(binary_expr),
            PunctuationKind::DoubleLessThanSign => Expression::LeftShift(binary_expr),
            PunctuationKind::TrippleGreaterThanSign => {
                Expression::ArithmeticRightShift(binary_expr)
            }
            PunctuationKind::DoubleGreaterThanSign => Expression::LogicalRightShift(binary_expr),
            _ => unreachable!(),
        };
    }

    expr
}

macro_rules! binary_expression {
    ($term:expr, [$($punct:ident),+ $(,)?] $(,)?) => {{
        let op = punctuation(&[$(PunctuationKind::$punct),+]);
        let tail = parser!(op <.> {$term}!![expect!("expression")]);
        parser!(({$term} <.> *tail)->[build_binary_expression_tree])
    }};
}

fn expression() -> impl Jam1Parser<Expression> {
    let mul_expr = binary_expression!(unary_expression(), [Asterisk, Slash, PercentSign]);
    let add_expr = binary_expression!(mul_expr, [PlusSign, MinusSign]);
    let shift_expr = binary_expression!(
        add_expr,
        [
            DoubleLessThanSign,
            TrippleGreaterThanSign,
            DoubleGreaterThanSign,
        ],
    );
    let and_expr = binary_expression!(shift_expr, [Ampersand]);
    let xor_expr = binary_expression!(and_expr, [Accent]);
    let or_expr = binary_expression!(xor_expr, [VerticalBar]);

    or_expr
}

fn label() -> impl Jam1Parser<Label> {
    let label_value = choice!(
        parser!({punctuation(&[PunctuationKind::Colon])}->[|colon| LabelValue::Address { colon }]),
        parser!(
            ({punctuation(&[PunctuationKind::EqualSign])} <.> {expression()}!![expect!("expression")])
            ->[|(assign_op, value)| LabelValue::Expression { assign_op, value }]
        ),
    );

    parser!(
        ({identifier()} <.> label_value!![expect!("`:` or `=`")])
        ->[|(name, value)| Label::new(name, value)]
    )
}

fn offset_directive() -> impl Jam1Parser<OffsetDirective> {
    parser!(
        ({directive(DirectiveKind::Offset)} <.> {integer_literal()}!![expect!("integer literal")])
        ->[|(directive, value)| OffsetDirective::new(directive, value)]
    )
}

fn align_directive() -> impl Jam1Parser<AlignDirective> {
    parser!(
        ({directive(DirectiveKind::Align)} <.> {integer_literal()}!![expect!("integer literal")])
        ->[|(directive, value)| AlignDirective::new(directive, value)]
    )
}

fn origin_directive() -> impl Jam1Parser<OriginDirective> {
    parser!(
        ({directive(DirectiveKind::Origin)} <.> {integer_literal()}!![expect!("integer literal")])
        ->[|(directive, value)| OriginDirective::new(directive, value)]
    )
}

fn section_directive() -> impl Jam1Parser<SectionDirective> {
    parser!(
        (
            {directive(DirectiveKind::Section)}
            <.> {string_literal()}!![expect!("string literal")]
            <.> ?{integer_literal()}
        )->[|((directive, name), base)| SectionDirective::new(directive, name, base)]
    )
}

fn include_directive() -> impl Jam1Parser<IncludeDirective> {
    parser!(
        ({directive(DirectiveKind::Include)} <.> {string_literal()}!![expect!("string literal")])
        ->[|(directive, path)| IncludeDirective::new(directive, path)]
    )
}

fn mov_instruction() -> impl Jam1Parser<MovInstruction> {
    let dst = parser!(
        {register()}->[MovDestination::Register]
        <|> (
                {punctuation(&[PunctuationKind::OpeningBracket])}
                <.> {register()}!![expect!("register")]
                <.> {punctuation(&[PunctuationKind::ClosingBracket])}!![expect!("`]`")]
            )->[|((open_bracket, address_source), close_bracket)|
                    MovDestination::Memory { open_bracket, address_source, close_bracket }]
    );

    let src = parser!(
        {expression()}->[MovSource::Value]
        <|> {register()}->[MovSource::Register]
        <|> (
                {punctuation(&[PunctuationKind::OpeningBracket])}
                <.> {register()}!![expect!("register")]
                <.> {punctuation(&[PunctuationKind::ClosingBracket])}!![expect!("`]`")]
            )->[|((open_bracket, address_source), close_bracket)|
                    MovSource::Memory { open_bracket, address_source, close_bracket }]
    );

    let raw = sequence!(
        mnemonic([MnemonicKind::Mov]),
        parser!(dst!![expect!("register or memory location")]),
        parser!({punctuation(&[PunctuationKind::Comma])}!![expect!("`,`")]),
        parser!(src!![expect!("expression, register or memory location")]),
    );

    parse_fn!(|input| {
        match raw.run(input) {
            ParseResult::Match {
                value: (mnemonic, dst, comma, src),
                span,
                remaining,
            } => {
                let dst_span = dst.span();
                let src_span = src.span();

                if let Some(inst) = MovInstruction::new(mnemonic, dst, comma, src) {
                    ParseResult::Match {
                        value: inst,
                        span,
                        remaining,
                    }
                } else {
                    ParseResult::Err(ParseError::InvalidOperands {
                        op1: dst_span,
                        op2: src_span,
                    })
                }
            }
            ParseResult::NoMatch => ParseResult::NoMatch,
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    })
}

macro_rules! def_single_reg_inst {
    ($name:ident, $mnemonic:ident, $inst_ty:ty) => {
        fn $name() -> impl Jam1Parser<$inst_ty> {
            let raw = sequence!(
                mnemonic([MnemonicKind::$mnemonic]),
                parser!({register()}!![expect!("register")]),
            );

            parse_fn!(|input| {
                match raw.run(input) {
                    ParseResult::Match {
                        value: (mnemonic, reg),
                        span,
                        remaining,
                    } => {
                        let reg_span = reg.span();

                        if let Some(inst) = <$inst_ty>::new(mnemonic, reg) {
                            ParseResult::Match {
                                value: inst,
                                span,
                                remaining,
                            }
                        } else {
                            ParseResult::Err(ParseError::InvalidRegister { register: reg_span })
                        }
                    }
                    ParseResult::NoMatch => ParseResult::NoMatch,
                    ParseResult::Err(err) => ParseResult::Err(err),
                }
            })
        }
    };
}

def_single_reg_inst!(inc_instruction, Inc, IncInstruction);
def_single_reg_inst!(incc_instruction, Incc, InccInstruction);
def_single_reg_inst!(dec_instruction, Dec, DecInstruction);
def_single_reg_inst!(push_instruction, Push, PushInstruction);
def_single_reg_inst!(pop_instruction, Pop, PopInstruction);
def_single_reg_inst!(shl_instruction, Shl, ShlInstruction);
def_single_reg_inst!(shr_instruction, Shr, ShrInstruction);
def_single_reg_inst!(not_instruction, Not, NotInstruction);
def_single_reg_inst!(test_instruction, Test, TestInstruction);

macro_rules! def_dual_reg_inst {
    ($name:ident, $mnemonic:ident, $inst_ty:ty) => {
        fn $name() -> impl Jam1Parser<$inst_ty> {
            let raw = sequence!(
                mnemonic([MnemonicKind::$mnemonic]),
                parser!({register()}!![expect!("register")]),
                parser!({punctuation(&[PunctuationKind::Comma])}!![expect!("`,`")]),
                parser!({register()}!![expect!("register")]),
            );

            parse_fn!(|input| {
                match raw.run(input) {
                    ParseResult::Match {
                        value: (mnemonic, dst, comma, src),
                        span,
                        remaining,
                    } => {
                        let dst_span = dst.span();
                        let src_span = src.span();

                        if let Some(inst) = <$inst_ty>::new(mnemonic, dst, comma, src) {
                            ParseResult::Match {
                                value: inst,
                                span,
                                remaining,
                            }
                        } else {
                            ParseResult::Err(ParseError::InvalidOperands {
                                op1: dst_span,
                                op2: src_span,
                            })
                        }
                    }
                    ParseResult::NoMatch => ParseResult::NoMatch,
                    ParseResult::Err(err) => ParseResult::Err(err),
                }
            })
        }
    };
}

def_dual_reg_inst!(add_instruction, Add, AddInstruction);
def_dual_reg_inst!(addc_instruction, Addc, AddcInstruction);
def_dual_reg_inst!(sub_instruction, Sub, SubInstruction);
def_dual_reg_inst!(subb_instruction, Subb, SubbInstruction);
def_dual_reg_inst!(and_instruction, And, AndInstruction);
def_dual_reg_inst!(or_instruction, Or, OrInstruction);
def_dual_reg_inst!(xor_instruction, Xor, XorInstruction);
def_dual_reg_inst!(cmp_instruction, Cmp, CmpInstruction);
def_dual_reg_inst!(addac_instruction, Addac, AddacInstruction);
def_dual_reg_inst!(subae_instruction, Subae, SubaeInstruction);

fn jump_target() -> impl Jam1Parser<JumpTarget> {
    choice!(
        parser!({expression()}->[JumpTarget::Value]),
        parser!({register()}->[JumpTarget::Register]),
    )
}

macro_rules! def_jump_inst {
    ($name:ident, $mnemonic:ident, $inst_ty:ty) => {
        fn $name() -> impl Jam1Parser<$inst_ty> {
            let raw = sequence!(
                mnemonic([MnemonicKind::$mnemonic]),
                parser!({jump_target()}!![expect!("register or expression")]),
            );

            parse_fn!(|input| {
                match raw.run(input) {
                    ParseResult::Match {
                        value: (mnemonic, target),
                        span,
                        remaining,
                    } => {
                        let target_span = target.span();

                        if let Some(inst) = <$inst_ty>::new(mnemonic, target) {
                            ParseResult::Match {
                                value: inst,
                                span,
                                remaining,
                            }
                        } else {
                            ParseResult::Err(ParseError::InvalidRegister {
                                register: target_span,
                            })
                        }
                    }
                    ParseResult::NoMatch => ParseResult::NoMatch,
                    ParseResult::Err(err) => ParseResult::Err(err),
                }
            })
        }
    };
}

def_jump_inst!(call_instruction, Call, CallInstruction);
def_jump_inst!(callbd_instruction, CallBd, CallBdInstruction);
def_jump_inst!(jmp_instruction, Jmp, JmpInstruction);

fn in_instruction() -> impl Jam1Parser<InInstruction> {
    let raw = sequence!(
        mnemonic([MnemonicKind::In]),
        parser!({register()}!![expect!("register")]),
        parser!({punctuation(&[PunctuationKind::Comma])}!![expect!("`,`")]),
        parser!({io_register()}!![expect!("IO register")]),
    );

    parse_fn!(|input| {
        match raw.run(input) {
            ParseResult::Match {
                value: (mnemonic, dst, comma, src),
                span,
                remaining,
            } => {
                let dst_span = dst.span();
                let src_span = src.span();

                if let Some(inst) = InInstruction::new(mnemonic, dst, comma, src) {
                    ParseResult::Match {
                        value: inst,
                        span,
                        remaining,
                    }
                } else {
                    ParseResult::Err(ParseError::InvalidOperands {
                        op1: dst_span,
                        op2: src_span,
                    })
                }
            }
            ParseResult::NoMatch => ParseResult::NoMatch,
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    })
}

fn out_instruction() -> impl Jam1Parser<OutInstruction> {
    let raw = sequence!(
        mnemonic([MnemonicKind::In]),
        parser!({io_register()}!![expect!("IO register")]),
        parser!({punctuation(&[PunctuationKind::Comma])}!![expect!("`,`")]),
        parser!({register()}!![expect!("register")]),
    );

    parse_fn!(|input| {
        match raw.run(input) {
            ParseResult::Match {
                value: (mnemonic, dst, comma, src),
                span,
                remaining,
            } => {
                let dst_span = dst.span();
                let src_span = src.span();

                if let Some(inst) = OutInstruction::new(mnemonic, dst, comma, src) {
                    ParseResult::Match {
                        value: inst,
                        span,
                        remaining,
                    }
                } else {
                    ParseResult::Err(ParseError::InvalidOperands {
                        op1: dst_span,
                        op2: src_span,
                    })
                }
            }
            ParseResult::NoMatch => ParseResult::NoMatch,
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    })
}

fn branch_instruction() -> impl Jam1Parser<BranchInstruction> {
    let raw = sequence!(
        mnemonic([
            MnemonicKind::Jo,
            MnemonicKind::Jno,
            MnemonicKind::Js,
            MnemonicKind::Jns,
            MnemonicKind::Jz,
            MnemonicKind::Jnz,
            MnemonicKind::Je,
            MnemonicKind::Jne,
            MnemonicKind::Jc,
            MnemonicKind::Jnc,
            MnemonicKind::Jnae,
            MnemonicKind::Jb,
            MnemonicKind::Jae,
            MnemonicKind::Jnb,
            MnemonicKind::Jbe,
            MnemonicKind::Jna,
            MnemonicKind::Ja,
            MnemonicKind::Jnbe,
            MnemonicKind::Jl,
            MnemonicKind::Jnge,
            MnemonicKind::Jge,
            MnemonicKind::Jnl,
            MnemonicKind::Jle,
            MnemonicKind::Jng,
            MnemonicKind::Jg,
            MnemonicKind::Jnle,
            MnemonicKind::Jlc,
            MnemonicKind::Jnlc,
        ]),
        parser!({jump_target()}!![expect!("register or expression")]),
    );

    parse_fn!(|input| {
        match raw.run(input) {
            ParseResult::Match {
                value: (mnemonic, target),
                span,
                remaining,
            } => {
                let target_span = target.span();

                if let Some(inst) = BranchInstruction::new(mnemonic, target) {
                    ParseResult::Match {
                        value: inst,
                        span,
                        remaining,
                    }
                } else {
                    ParseResult::Err(ParseError::InvalidRegister {
                        register: target_span,
                    })
                }
            }
            ParseResult::NoMatch => ParseResult::NoMatch,
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    })
}

fn instruction() -> impl Jam1Parser<Instruction> {
    let nop_instruction = mnemonic([MnemonicKind::Nop, MnemonicKind::Cnop]);
    let break_instruction = mnemonic([MnemonicKind::Break]);
    let lodsb_instruction = mnemonic([MnemonicKind::Lodsb]);
    let stosb_instruction = mnemonic([MnemonicKind::Stosb]);
    let ret_instruction = mnemonic([MnemonicKind::Ret]);
    let retbd_instruction = mnemonic([MnemonicKind::RetBd]);
    let clc_instruction = mnemonic([MnemonicKind::Clc]);

    choice!(
        parser!(nop_instruction->[Instruction::Nop]),
        parser!(break_instruction->[Instruction::Break]),
        parser!(lodsb_instruction->[Instruction::Lodsb]),
        parser!(stosb_instruction->[Instruction::Stosb]),
        parser!(ret_instruction->[Instruction::Ret]),
        parser!(retbd_instruction->[Instruction::RetBd]),
        parser!(clc_instruction->[Instruction::Clc]),
        parser!({mov_instruction()}->[Instruction::Mov]),
        parser!({inc_instruction()}->[Instruction::Inc]),
        parser!({incc_instruction()}->[Instruction::Incc]),
        parser!({dec_instruction()}->[Instruction::Dec]),
        parser!({push_instruction()}->[Instruction::Push]),
        parser!({pop_instruction()}->[Instruction::Pop]),
        parser!({shl_instruction()}->[Instruction::Shl]),
        parser!({shr_instruction()}->[Instruction::Shr]),
        parser!({not_instruction()}->[Instruction::Not]),
        parser!({test_instruction()}->[Instruction::Test]),
        parser!({add_instruction()}->[Instruction::Add]),
        parser!({addc_instruction()}->[Instruction::Addc]),
        parser!({sub_instruction()}->[Instruction::Sub]),
        parser!({subb_instruction()}->[Instruction::Subb]),
        parser!({and_instruction()}->[Instruction::And]),
        parser!({or_instruction()}->[Instruction::Or]),
        parser!({xor_instruction()}->[Instruction::Xor]),
        parser!({cmp_instruction()}->[Instruction::Cmp]),
        parser!({addac_instruction()}->[Instruction::Addac]),
        parser!({subae_instruction()}->[Instruction::Subae]),
        parser!({call_instruction()}->[Instruction::Call]),
        parser!({callbd_instruction()}->[Instruction::CallBd]),
        parser!({jmp_instruction()}->[Instruction::Jmp]),
        parser!({branch_instruction()}->[Instruction::Branch]),
        parser!({in_instruction()}->[Instruction::In]),
        parser!({out_instruction()}->[Instruction::Out]),
    )
}

fn statement() -> impl Jam1Parser<Statement> {
    choice!(
        parser!(({label()}->[Box::new])->[Statement::Label]),
        parser!(({offset_directive()}->[Box::new])->[Statement::OffsetDirective]),
        parser!(({align_directive()}->[Box::new])->[Statement::AlignDirective]),
        parser!(({origin_directive()}->[Box::new])->[Statement::OriginDirective]),
        parser!(({section_directive()}->[Box::new])->[Statement::SectionDirective]),
        parser!(({include_directive()}->[Box::new])->[Statement::IncludeDirective]),
        parser!(({instruction()}->[Box::new])->[Statement::Instruction]),
    )
}

pub fn parse(input: TokenStream<Jam1Token>) -> Result<Statement, ParseError> {
    assert!(!input.remaining().is_empty());

    match statement().run(input) {
        ParseResult::Match {
            value, remaining, ..
        } => {
            if remaining.remaining().is_empty() {
                Ok(value)
            } else {
                let first = remaining.remaining().first().unwrap();
                let last = remaining.remaining().last().unwrap();
                Err(ParseError::TokensRemaining {
                    span: first.span.join(&last.span),
                })
            }
        }
        ParseResult::NoMatch => {
            let first = input.remaining().first().unwrap();
            let last = input.remaining().last().unwrap();
            Err(ParseError::NoMatch {
                span: first.span.join(&last.span),
            })
        }
        ParseResult::Err(err) => Err(err),
    }
}
