#![allow(dead_code)]

use super::lexer::*;
use super::AssemblerError;
use super::SharedStr;
use langbox::TextSpan;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub trait Spanned {
    fn span(&self) -> TextSpan;
}

#[derive(Clone)]
pub struct Punctuation {
    kind: PunctuationKind,
    span: TextSpan,
}

impl Punctuation {
    #[inline]
    pub fn new(kind: PunctuationKind, span: TextSpan) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn kind(&self) -> PunctuationKind {
        self.kind
    }
}

impl Debug for Punctuation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for Punctuation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.kind, f)
    }
}

impl Spanned for Punctuation {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct Directive {
    kind: DirectiveKind,
    span: TextSpan,
}

impl Directive {
    #[inline]
    pub fn new(kind: DirectiveKind, span: TextSpan) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn kind(&self) -> DirectiveKind {
        self.kind
    }
}

impl Debug for Directive {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for Directive {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.kind, f)
    }
}

impl Spanned for Directive {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct Register {
    kind: RegisterKind,
    span: TextSpan,
}

impl Register {
    #[inline]
    pub fn new(kind: RegisterKind, span: TextSpan) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn kind(&self) -> RegisterKind {
        self.kind
    }
}

impl Debug for Register {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for Register {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.kind, f)
    }
}

impl Spanned for Register {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct IoRegister {
    kind: IoRegisterKind,
    span: TextSpan,
}

impl IoRegister {
    #[inline]
    pub fn new(kind: IoRegisterKind, span: TextSpan) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn kind(&self) -> IoRegisterKind {
        self.kind
    }
}

impl Debug for IoRegister {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for IoRegister {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.kind, f)
    }
}

impl Spanned for IoRegister {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct Mnemonic {
    kind: MnemonicKind,
    span: TextSpan,
}

impl Mnemonic {
    #[inline]
    pub fn new(kind: MnemonicKind, span: TextSpan) -> Self {
        Self { kind, span }
    }

    #[inline]
    pub fn kind(&self) -> MnemonicKind {
        self.kind
    }
}

impl Debug for Mnemonic {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.kind, f)
    }
}

impl Display for Mnemonic {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.kind, f)
    }
}

impl Spanned for Mnemonic {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct Identifier {
    name: SharedStr,
    span: TextSpan,
}

impl Identifier {
    #[inline]
    pub fn new(name: SharedStr, span: TextSpan) -> Self {
        Self { name, span }
    }

    #[inline]
    pub fn name(&self) -> SharedStr {
        SharedStr::clone(&self.name)
    }
}

impl Debug for Identifier {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.name, f)
    }
}

impl Display for Identifier {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.name, f)
    }
}

impl Spanned for Identifier {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct IntegerLiteral {
    value: Option<i64>,
    span: TextSpan,
}

impl IntegerLiteral {
    #[inline]
    pub fn new(value: Option<i64>, span: TextSpan) -> Self {
        Self { value, span }
    }

    #[inline]
    pub fn value(&self) -> Option<i64> {
        self.value
    }
}

impl Debug for IntegerLiteral {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(value) = self.value {
            Debug::fmt(&value, f)
        } else {
            write!(f, "<invalid>")
        }
    }
}

impl Display for IntegerLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if let Some(value) = self.value {
            Display::fmt(&value, f)
        } else {
            write!(f, "<invalid>")
        }
    }
}

impl Spanned for IntegerLiteral {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Clone)]
pub struct StringLiteral {
    value: SharedStr,
    span: TextSpan,
}

impl StringLiteral {
    #[inline]
    pub fn new(value: SharedStr, span: TextSpan) -> Self {
        Self { value, span }
    }

    #[inline]
    pub fn value(&self) -> SharedStr {
        SharedStr::clone(&self.value)
    }
}

impl Debug for StringLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "\"{}\"", self.value)
    }
}

impl Display for StringLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "\"{}\"", self.value)
    }
}

impl Spanned for StringLiteral {
    #[inline]
    fn span(&self) -> TextSpan {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct GroupExpression {
    open_paren: Punctuation,
    inner: Expression,
    close_paren: Punctuation,
}

impl GroupExpression {
    #[inline]
    pub fn new(open_paren: Punctuation, inner: Expression, close_paren: Punctuation) -> Self {
        Self {
            open_paren,
            inner,
            close_paren,
        }
    }

    #[inline]
    pub fn inner(&self) -> &Expression {
        &self.inner
    }
}

impl Display for GroupExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}{}{}", self.open_paren, self.inner, self.close_paren)
    }
}

impl Spanned for GroupExpression {
    fn span(&self) -> TextSpan {
        self.open_paren.span().join(&self.close_paren.span())
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    op: Punctuation,
    inner: Expression,
}

impl UnaryExpression {
    #[inline]
    pub fn new(op: Punctuation, inner: Expression) -> Self {
        Self { op, inner }
    }

    #[inline]
    pub fn inner(&self) -> &Expression {
        &self.inner
    }
}

impl Display for UnaryExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}{}", self.op, self.inner)
    }
}

impl Spanned for UnaryExpression {
    fn span(&self) -> TextSpan {
        self.op.span().join(&self.inner.span())
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    lhs: Expression,
    op: Punctuation,
    rhs: Expression,
}

impl BinaryExpression {
    #[inline]
    pub fn new(lhs: Expression, op: Punctuation, rhs: Expression) -> Self {
        Self { lhs, op, rhs }
    }

    #[inline]
    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    #[inline]
    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}

impl Display for BinaryExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {} {}", self.lhs, self.op, self.rhs)
    }
}

impl Spanned for BinaryExpression {
    fn span(&self) -> TextSpan {
        self.lhs.span().join(&self.rhs.span())
    }
}

#[derive(Clone)]
pub enum Expression {
    Literal(Box<IntegerLiteral>),
    Identifier(Box<Identifier>),
    Group(Box<GroupExpression>),
    Identity(Box<UnaryExpression>),
    Negation(Box<UnaryExpression>),
    BitwiseNot(Box<UnaryExpression>),
    Addition(Box<BinaryExpression>),
    Subtraction(Box<BinaryExpression>),
    Multiplication(Box<BinaryExpression>),
    Division(Box<BinaryExpression>),
    Remainder(Box<BinaryExpression>),
    LeftShift(Box<BinaryExpression>),
    ArithmeticRightShift(Box<BinaryExpression>),
    LogicalRightShift(Box<BinaryExpression>),
    BitwiseAnd(Box<BinaryExpression>),
    BitwiseOr(Box<BinaryExpression>),
    BitwiseXor(Box<BinaryExpression>),
}

impl Debug for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Literal(expr) => Debug::fmt(expr, f),
            Self::Identifier(expr) => Debug::fmt(expr, f),
            Self::Group(expr) => Debug::fmt(expr, f),
            Self::Identity(expr) | Self::Negation(expr) | Self::BitwiseNot(expr) => {
                Debug::fmt(expr, f)
            }
            Self::Addition(expr)
            | Self::Subtraction(expr)
            | Self::Multiplication(expr)
            | Self::Division(expr)
            | Self::Remainder(expr)
            | Self::LeftShift(expr)
            | Self::ArithmeticRightShift(expr)
            | Self::LogicalRightShift(expr)
            | Self::BitwiseAnd(expr)
            | Self::BitwiseOr(expr)
            | Self::BitwiseXor(expr) => Debug::fmt(expr, f),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Literal(expr) => Display::fmt(expr, f),
            Self::Identifier(expr) => Display::fmt(expr, f),
            Self::Group(expr) => Display::fmt(expr, f),
            Self::Identity(expr) | Self::Negation(expr) | Self::BitwiseNot(expr) => {
                Display::fmt(expr, f)
            }
            Self::Addition(expr)
            | Self::Subtraction(expr)
            | Self::Multiplication(expr)
            | Self::Division(expr)
            | Self::Remainder(expr)
            | Self::LeftShift(expr)
            | Self::ArithmeticRightShift(expr)
            | Self::LogicalRightShift(expr)
            | Self::BitwiseAnd(expr)
            | Self::BitwiseOr(expr)
            | Self::BitwiseXor(expr) => Display::fmt(expr, f),
        }
    }
}

impl Spanned for Expression {
    fn span(&self) -> TextSpan {
        match self {
            Self::Literal(expr) => expr.span(),
            Self::Identifier(expr) => expr.span(),
            Self::Group(expr) => expr.span(),
            Self::Identity(expr) | Self::Negation(expr) | Self::BitwiseNot(expr) => expr.span(),
            Self::Addition(expr)
            | Self::Subtraction(expr)
            | Self::Multiplication(expr)
            | Self::Division(expr)
            | Self::Remainder(expr)
            | Self::LeftShift(expr)
            | Self::ArithmeticRightShift(expr)
            | Self::LogicalRightShift(expr)
            | Self::BitwiseAnd(expr)
            | Self::BitwiseOr(expr)
            | Self::BitwiseXor(expr) => expr.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LabelValue {
    Address {
        colon: Punctuation,
    },
    Expression {
        assign_op: Punctuation,
        value: Expression,
    },
}

impl Spanned for LabelValue {
    fn span(&self) -> TextSpan {
        match self {
            Self::Address { colon } => colon.span(),
            Self::Expression { assign_op, value } => assign_op.span().join(&value.span()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Label {
    name: Identifier,
    value: LabelValue,
}

impl Label {
    #[inline]
    pub fn new(name: Identifier, value: LabelValue) -> Self {
        Self { name, value }
    }

    #[inline]
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    #[inline]
    pub fn value(&self) -> &LabelValue {
        &self.value
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.value {
            LabelValue::Address { colon } => write!(f, "{}{colon}", self.name),
            LabelValue::Expression { assign_op, value } => {
                write!(f, "{} {assign_op} {value}", self.name)
            }
        }
    }
}

impl Spanned for Label {
    fn span(&self) -> TextSpan {
        self.name.span().join(&self.value.span())
    }
}

#[derive(Clone, Debug)]
pub struct OffsetDirective {
    directive: Directive,
    value: IntegerLiteral,
}

impl OffsetDirective {
    #[inline]
    pub fn new(directive: Directive, value: IntegerLiteral) -> Self {
        Self { directive, value }
    }

    #[inline]
    pub fn directive(&self) -> &Directive {
        &self.directive
    }

    #[inline]
    pub fn value(&self) -> &IntegerLiteral {
        &self.value
    }
}

impl Display for OffsetDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.directive, self.value)
    }
}

impl Spanned for OffsetDirective {
    fn span(&self) -> TextSpan {
        self.directive.span().join(&self.value.span())
    }
}

#[derive(Clone, Debug)]
pub struct AlignDirective {
    directive: Directive,
    value: IntegerLiteral,
}

impl AlignDirective {
    #[inline]
    pub fn new(directive: Directive, value: IntegerLiteral) -> Self {
        Self { directive, value }
    }

    #[inline]
    pub fn directive(&self) -> &Directive {
        &self.directive
    }

    #[inline]
    pub fn value(&self) -> &IntegerLiteral {
        &self.value
    }
}

impl Display for AlignDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.directive, self.value)
    }
}

impl Spanned for AlignDirective {
    fn span(&self) -> TextSpan {
        self.directive.span().join(&self.value.span())
    }
}

#[derive(Clone, Debug)]
pub struct OriginDirective {
    directive: Directive,
    value: IntegerLiteral,
}

impl OriginDirective {
    #[inline]
    pub fn new(directive: Directive, value: IntegerLiteral) -> Self {
        Self { directive, value }
    }

    #[inline]
    pub fn directive(&self) -> &Directive {
        &self.directive
    }

    #[inline]
    pub fn value(&self) -> &IntegerLiteral {
        &self.value
    }
}

impl Display for OriginDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.directive, self.value)
    }
}

impl Spanned for OriginDirective {
    fn span(&self) -> TextSpan {
        self.directive.span().join(&self.value.span())
    }
}

#[derive(Clone, Debug)]
pub struct SectionDirective {
    directive: Directive,
    name: StringLiteral,
    base: Option<IntegerLiteral>,
}

impl SectionDirective {
    #[inline]
    pub fn new(directive: Directive, name: StringLiteral, base: Option<IntegerLiteral>) -> Self {
        Self {
            directive,
            name,
            base,
        }
    }

    #[inline]
    pub fn directive(&self) -> &Directive {
        &self.directive
    }

    #[inline]
    pub fn name(&self) -> &StringLiteral {
        &self.name
    }

    #[inline]
    pub fn base(&self) -> Option<&IntegerLiteral> {
        self.base.as_ref()
    }
}

impl Display for SectionDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.directive, self.name)
    }
}

impl Spanned for SectionDirective {
    fn span(&self) -> TextSpan {
        self.directive.span().join(&self.name.span())
    }
}

#[derive(Clone, Debug)]
pub struct IncludeDirective {
    directive: Directive,
    path: StringLiteral,
}

impl IncludeDirective {
    #[inline]
    pub fn new(directive: Directive, path: StringLiteral) -> Self {
        Self { directive, path }
    }

    #[inline]
    pub fn directive(&self) -> &Directive {
        &self.directive
    }

    #[inline]
    pub fn path(&self) -> &StringLiteral {
        &self.path
    }
}

impl Display for IncludeDirective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.directive, self.path)
    }
}

impl Spanned for IncludeDirective {
    fn span(&self) -> TextSpan {
        self.directive.span().join(&self.path.span())
    }
}

#[derive(Clone, Debug)]
pub enum MovDestination {
    Register(Register),
    Memory {
        open_bracket: Punctuation,
        address_source: Register,
        close_bracket: Punctuation,
    },
}

impl Display for MovDestination {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Register(reg) => Display::fmt(reg, f),
            Self::Memory {
                open_bracket,
                address_source,
                close_bracket,
            } => write!(f, "{open_bracket}{address_source}{close_bracket}"),
        }
    }
}

impl Spanned for MovDestination {
    fn span(&self) -> TextSpan {
        match self {
            Self::Register(reg) => reg.span(),
            Self::Memory {
                open_bracket,
                close_bracket,
                ..
            } => open_bracket.span().join(&close_bracket.span()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MovSource {
    Value(Expression),
    Register(Register),
    Memory {
        open_bracket: Punctuation,
        address_source: Register,
        close_bracket: Punctuation,
    },
}

impl Display for MovSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Value(value) => Display::fmt(value, f),
            Self::Register(reg) => Display::fmt(reg, f),
            Self::Memory {
                open_bracket,
                address_source,
                close_bracket,
            } => write!(f, "{open_bracket}{address_source}{close_bracket}"),
        }
    }
}

impl Spanned for MovSource {
    fn span(&self) -> TextSpan {
        match self {
            Self::Value(value) => value.span(),
            Self::Register(reg) => reg.span(),
            Self::Memory {
                open_bracket,
                close_bracket,
                ..
            } => open_bracket.span().join(&close_bracket.span()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MovInstruction {
    mnemonic: Mnemonic,
    destination: MovDestination,
    comma: Punctuation,
    source: MovSource,
    emit_size: u16,
}

impl MovInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: MovDestination,
        comma: Punctuation,
        source: MovSource,
    ) -> Option<Self> {
        let emit_size = match (&destination, &source) {
            (MovDestination::Register(destination), MovSource::Value(_)) => {
                match destination.kind {
                    RegisterKind::A
                    | RegisterKind::B
                    | RegisterKind::C
                    | RegisterKind::D
                    | RegisterKind::TL
                    | RegisterKind::TH => 2,
                    RegisterKind::TX | RegisterKind::AB | RegisterKind::CD => 4,
                    RegisterKind::SI | RegisterKind::DI => 5,
                    _ => return None,
                }
            }
            (MovDestination::Register(destination), MovSource::Register(source)) => {
                match (destination.kind, source.kind) {
                    (RegisterKind::A, RegisterKind::B)
                    | (RegisterKind::A, RegisterKind::C)
                    | (RegisterKind::A, RegisterKind::D)
                    | (RegisterKind::B, RegisterKind::A)
                    | (RegisterKind::B, RegisterKind::C)
                    | (RegisterKind::B, RegisterKind::D)
                    | (RegisterKind::C, RegisterKind::A)
                    | (RegisterKind::C, RegisterKind::B)
                    | (RegisterKind::C, RegisterKind::D)
                    | (RegisterKind::D, RegisterKind::A)
                    | (RegisterKind::D, RegisterKind::B)
                    | (RegisterKind::D, RegisterKind::C)
                    | (RegisterKind::A, RegisterKind::TL)
                    | (RegisterKind::A, RegisterKind::TH)
                    | (RegisterKind::B, RegisterKind::TL)
                    | (RegisterKind::B, RegisterKind::TH)
                    | (RegisterKind::C, RegisterKind::TL)
                    | (RegisterKind::C, RegisterKind::TH)
                    | (RegisterKind::D, RegisterKind::TL)
                    | (RegisterKind::D, RegisterKind::TH)
                    | (RegisterKind::TL, RegisterKind::A)
                    | (RegisterKind::TL, RegisterKind::B)
                    | (RegisterKind::TL, RegisterKind::C)
                    | (RegisterKind::TL, RegisterKind::D)
                    | (RegisterKind::TH, RegisterKind::A)
                    | (RegisterKind::TH, RegisterKind::B)
                    | (RegisterKind::TH, RegisterKind::C)
                    | (RegisterKind::TH, RegisterKind::D)
                    | (RegisterKind::RA, RegisterKind::TX)
                    | (RegisterKind::TX, RegisterKind::RA)
                    | (RegisterKind::SP, RegisterKind::TX)
                    | (RegisterKind::TX, RegisterKind::SP)
                    | (RegisterKind::SI, RegisterKind::TX)
                    | (RegisterKind::TX, RegisterKind::SI)
                    | (RegisterKind::DI, RegisterKind::TX)
                    | (RegisterKind::TX, RegisterKind::DI)
                    | (RegisterKind::DI, RegisterKind::SI)
                    | (RegisterKind::SI, RegisterKind::DI)
                    | (RegisterKind::SI, RegisterKind::SP)
                    | (RegisterKind::DI, RegisterKind::SP) => 1,
                    _ => return None,
                }
            }
            (
                MovDestination::Register(destination),
                MovSource::Memory {
                    address_source: source,
                    ..
                },
            ) => match (destination.kind, source.kind) {
                (RegisterKind::A, RegisterKind::SI)
                | (RegisterKind::B, RegisterKind::SI)
                | (RegisterKind::C, RegisterKind::SI)
                | (RegisterKind::D, RegisterKind::SI)
                | (RegisterKind::A, RegisterKind::DI)
                | (RegisterKind::B, RegisterKind::DI)
                | (RegisterKind::C, RegisterKind::DI)
                | (RegisterKind::D, RegisterKind::DI)
                | (RegisterKind::A, RegisterKind::TX)
                | (RegisterKind::B, RegisterKind::TX)
                | (RegisterKind::C, RegisterKind::TX)
                | (RegisterKind::D, RegisterKind::TX) => 1,
                _ => return None,
            },
            (
                MovDestination::Memory {
                    address_source: destination,
                    ..
                },
                MovSource::Register(source),
            ) => match (destination.kind, source.kind) {
                (RegisterKind::SI, RegisterKind::A)
                | (RegisterKind::SI, RegisterKind::B)
                | (RegisterKind::SI, RegisterKind::C)
                | (RegisterKind::SI, RegisterKind::D)
                | (RegisterKind::DI, RegisterKind::A)
                | (RegisterKind::DI, RegisterKind::B)
                | (RegisterKind::DI, RegisterKind::C)
                | (RegisterKind::DI, RegisterKind::D)
                | (RegisterKind::TX, RegisterKind::A)
                | (RegisterKind::TX, RegisterKind::B)
                | (RegisterKind::TX, RegisterKind::C)
                | (RegisterKind::TX, RegisterKind::D) => 1,
                _ => return None,
            },
            _ => return None,
        };

        Some(Self {
            mnemonic,
            destination,
            comma,
            source,
            emit_size,
        })
    }

    pub fn encode(
        &self,
        mut writer: impl std::io::Write,
        label_set: &HashMap<SharedStr, TextSpan>,
        label_values: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> std::io::Result<()> {
        match (&self.destination, &self.source) {
            (MovDestination::Register(destination), MovSource::Value(source)) => {
                let source = source.eval_or_zero(label_set, label_values, errors);
                let low = source as u8;
                let high = (source >> 8) as u8;

                match destination.kind {
                    RegisterKind::A => writer.write_all(&[0x01, low]),
                    RegisterKind::B => writer.write_all(&[0x02, low]),
                    RegisterKind::C => writer.write_all(&[0x03, low]),
                    RegisterKind::D => writer.write_all(&[0x04, low]),
                    RegisterKind::TL => writer.write_all(&[0x05, low]),
                    RegisterKind::TH => writer.write_all(&[0x06, low]),
                    RegisterKind::TX => writer.write_all(&[0x05, low, 0x06, high]),
                    RegisterKind::AB => writer.write_all(&[0x01, low, 0x02, high]),
                    RegisterKind::CD => writer.write_all(&[0x03, low, 0x04, high]),
                    RegisterKind::SI => writer.write_all(&[0x05, low, 0x06, high, 0x27]),
                    RegisterKind::DI => writer.write_all(&[0x05, low, 0x06, high, 0x29]),
                    _ => unreachable!("invalid MOV operands"),
                }
            }
            (MovDestination::Register(destination), MovSource::Register(source)) => {
                match (destination.kind, source.kind) {
                    (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0x07]),
                    (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0x08]),
                    (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0x09]),
                    (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0x0A]),
                    (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0x0B]),
                    (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0x0C]),
                    (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0x0D]),
                    (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0x0E]),
                    (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0x0F]),
                    (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0x10]),
                    (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0x11]),
                    (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0x12]),
                    (RegisterKind::TL, RegisterKind::A) => writer.write_all(&[0x13]),
                    (RegisterKind::TL, RegisterKind::B) => writer.write_all(&[0x14]),
                    (RegisterKind::TL, RegisterKind::C) => writer.write_all(&[0x15]),
                    (RegisterKind::TL, RegisterKind::D) => writer.write_all(&[0x16]),
                    (RegisterKind::TH, RegisterKind::A) => writer.write_all(&[0x17]),
                    (RegisterKind::TH, RegisterKind::B) => writer.write_all(&[0x18]),
                    (RegisterKind::TH, RegisterKind::C) => writer.write_all(&[0x19]),
                    (RegisterKind::TH, RegisterKind::D) => writer.write_all(&[0x1A]),
                    (RegisterKind::A, RegisterKind::TL) => writer.write_all(&[0x1B]),
                    (RegisterKind::B, RegisterKind::TL) => writer.write_all(&[0x1C]),
                    (RegisterKind::C, RegisterKind::TL) => writer.write_all(&[0x1D]),
                    (RegisterKind::D, RegisterKind::TL) => writer.write_all(&[0x1E]),
                    (RegisterKind::A, RegisterKind::TH) => writer.write_all(&[0x1F]),
                    (RegisterKind::B, RegisterKind::TH) => writer.write_all(&[0x20]),
                    (RegisterKind::C, RegisterKind::TH) => writer.write_all(&[0x21]),
                    (RegisterKind::D, RegisterKind::TH) => writer.write_all(&[0x22]),
                    (RegisterKind::RA, RegisterKind::TX) => writer.write_all(&[0x23]),
                    (RegisterKind::TX, RegisterKind::RA) => writer.write_all(&[0x24]),
                    (RegisterKind::SP, RegisterKind::TX) => writer.write_all(&[0x25]),
                    (RegisterKind::TX, RegisterKind::SP) => writer.write_all(&[0x26]),
                    (RegisterKind::SI, RegisterKind::TX) => writer.write_all(&[0x27]),
                    (RegisterKind::TX, RegisterKind::SI) => writer.write_all(&[0x28]),
                    (RegisterKind::DI, RegisterKind::TX) => writer.write_all(&[0x29]),
                    (RegisterKind::TX, RegisterKind::DI) => writer.write_all(&[0x2A]),
                    (RegisterKind::DI, RegisterKind::SI) => writer.write_all(&[0x2B]),
                    (RegisterKind::SI, RegisterKind::DI) => writer.write_all(&[0x2C]),
                    (RegisterKind::SI, RegisterKind::SP) => writer.write_all(&[0x2D]),
                    (RegisterKind::DI, RegisterKind::SP) => writer.write_all(&[0x2E]),
                    _ => unreachable!("invalid MOV operands"),
                }
            }
            (
                MovDestination::Register(destination),
                MovSource::Memory {
                    address_source: source,
                    ..
                },
            ) => match (destination.kind, source.kind) {
                (RegisterKind::A, RegisterKind::SI) => writer.write_all(&[0x40]),
                (RegisterKind::B, RegisterKind::SI) => writer.write_all(&[0x41]),
                (RegisterKind::C, RegisterKind::SI) => writer.write_all(&[0x42]),
                (RegisterKind::D, RegisterKind::SI) => writer.write_all(&[0x43]),
                (RegisterKind::A, RegisterKind::DI) => writer.write_all(&[0x44]),
                (RegisterKind::B, RegisterKind::DI) => writer.write_all(&[0x45]),
                (RegisterKind::C, RegisterKind::DI) => writer.write_all(&[0x46]),
                (RegisterKind::D, RegisterKind::DI) => writer.write_all(&[0x47]),
                (RegisterKind::A, RegisterKind::TX) => writer.write_all(&[0x48]),
                (RegisterKind::B, RegisterKind::TX) => writer.write_all(&[0x49]),
                (RegisterKind::C, RegisterKind::TX) => writer.write_all(&[0x4A]),
                (RegisterKind::D, RegisterKind::TX) => writer.write_all(&[0x4B]),
                _ => unreachable!("invalid MOV operands"),
            },
            (
                MovDestination::Memory {
                    address_source: destination,
                    ..
                },
                MovSource::Register(source),
            ) => match (destination.kind, source.kind) {
                (RegisterKind::SI, RegisterKind::A) => writer.write_all(&[0x4C]),
                (RegisterKind::SI, RegisterKind::B) => writer.write_all(&[0x4D]),
                (RegisterKind::SI, RegisterKind::C) => writer.write_all(&[0x4E]),
                (RegisterKind::SI, RegisterKind::D) => writer.write_all(&[0x4F]),
                (RegisterKind::DI, RegisterKind::A) => writer.write_all(&[0x50]),
                (RegisterKind::DI, RegisterKind::B) => writer.write_all(&[0x51]),
                (RegisterKind::DI, RegisterKind::C) => writer.write_all(&[0x52]),
                (RegisterKind::DI, RegisterKind::D) => writer.write_all(&[0x53]),
                (RegisterKind::TX, RegisterKind::A) => writer.write_all(&[0x54]),
                (RegisterKind::TX, RegisterKind::B) => writer.write_all(&[0x55]),
                (RegisterKind::TX, RegisterKind::C) => writer.write_all(&[0x56]),
                (RegisterKind::TX, RegisterKind::D) => writer.write_all(&[0x57]),
                _ => unreachable!("invalid MOV operands"),
            },
            _ => unreachable!("invalid MOV operands"),
        }
    }
}

impl Display for MovInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for MovInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct IncInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl IncInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A
            | RegisterKind::B
            | RegisterKind::C
            | RegisterKind::D
            | RegisterKind::SI
            | RegisterKind::DI => Some(Self { mnemonic, register }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0xA0]),
            RegisterKind::B => writer.write_all(&[0xA1]),
            RegisterKind::C => writer.write_all(&[0xA2]),
            RegisterKind::D => writer.write_all(&[0xA3]),
            RegisterKind::SI => writer.write_all(&[0x35]),
            RegisterKind::DI => writer.write_all(&[0x36]),
            _ => unreachable!("invalid INC operand"),
        }
    }
}

impl Display for IncInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for IncInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct InccInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl InccInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A
            | RegisterKind::B
            | RegisterKind::C
            | RegisterKind::D
            | RegisterKind::SI => Some(Self { mnemonic, register }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0xA4]),
            RegisterKind::B => writer.write_all(&[0xA5]),
            RegisterKind::C => writer.write_all(&[0xA6]),
            RegisterKind::D => writer.write_all(&[0xA7]),
            RegisterKind::SI => writer.write_all(&[0x34]),
            _ => unreachable!("invalid INCC operand"),
        }
    }
}

impl Display for InccInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for InccInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct DecInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl DecInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A
            | RegisterKind::B
            | RegisterKind::C
            | RegisterKind::D
            | RegisterKind::SI
            | RegisterKind::DI => Some(Self { mnemonic, register }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0xC0]),
            RegisterKind::B => writer.write_all(&[0xC1]),
            RegisterKind::C => writer.write_all(&[0xC2]),
            RegisterKind::D => writer.write_all(&[0xC3]),
            RegisterKind::SI => writer.write_all(&[0x32]),
            RegisterKind::DI => writer.write_all(&[0x33]),
            _ => unreachable!("invalid DEC operand"),
        }
    }
}

impl Display for DecInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for DecInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct PushInstruction {
    mnemonic: Mnemonic,
    register: Register,
    emit_size: u16,
}

impl PushInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        let emit_size = match register.kind {
            RegisterKind::A
            | RegisterKind::B
            | RegisterKind::C
            | RegisterKind::D
            | RegisterKind::TL
            | RegisterKind::TH => 1,
            RegisterKind::TX => 2,
            RegisterKind::RA | RegisterKind::SP | RegisterKind::SI | RegisterKind::DI => 3,
            _ => return None,
        };

        Some(Self {
            mnemonic,
            register,
            emit_size,
        })
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0x72]),
            RegisterKind::B => writer.write_all(&[0x73]),
            RegisterKind::C => writer.write_all(&[0x74]),
            RegisterKind::D => writer.write_all(&[0x75]),
            RegisterKind::TL => writer.write_all(&[0x76]),
            RegisterKind::TH => writer.write_all(&[0x77]),
            RegisterKind::TX => writer.write_all(&[0x76, 0x77]),
            RegisterKind::RA => writer.write_all(&[0x24, 0x76, 0x77]),
            RegisterKind::SP => writer.write_all(&[0x26, 0x76, 0x77]),
            RegisterKind::SI => writer.write_all(&[0x28, 0x76, 0x77]),
            RegisterKind::DI => writer.write_all(&[0x2A, 0x76, 0x77]),
            _ => unreachable!("invalid PUSH operand"),
        }
    }
}

impl Display for PushInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for PushInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct PopInstruction {
    mnemonic: Mnemonic,
    register: Register,
    emit_size: u16,
}

impl PopInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        let emit_size = match register.kind {
            RegisterKind::A
            | RegisterKind::B
            | RegisterKind::C
            | RegisterKind::D
            | RegisterKind::TL
            | RegisterKind::TH => 1,
            RegisterKind::TX => 2,
            RegisterKind::RA | RegisterKind::SP | RegisterKind::SI | RegisterKind::DI => 4,
            _ => return None,
        };

        Some(Self {
            mnemonic,
            register,
            emit_size,
        })
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0x78]),
            RegisterKind::B => writer.write_all(&[0x79]),
            RegisterKind::C => writer.write_all(&[0x7A]),
            RegisterKind::D => writer.write_all(&[0x7B]),
            RegisterKind::TL => writer.write_all(&[0x7C]),
            RegisterKind::TH => writer.write_all(&[0x7D]),
            RegisterKind::TX => writer.write_all(&[0x7D, 0x7C]),
            RegisterKind::RA => writer.write_all(&[0x7D, 0x7C, 0x00, 0x23]),
            RegisterKind::SP => writer.write_all(&[0x7D, 0x7C, 0x00, 0x25]),
            RegisterKind::SI => writer.write_all(&[0x7D, 0x7C, 0x00, 0x27]),
            RegisterKind::DI => writer.write_all(&[0x7D, 0x7C, 0x00, 0x29]),
            _ => unreachable!("invalid POP operand"),
        }
    }
}

impl Display for PopInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for PopInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct ShlInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl ShlInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A | RegisterKind::B | RegisterKind::C | RegisterKind::D => {
                Some(Self { mnemonic, register })
            }
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0x80]),
            RegisterKind::B => writer.write_all(&[0x81]),
            RegisterKind::C => writer.write_all(&[0x82]),
            RegisterKind::D => writer.write_all(&[0x83]),
            _ => unreachable!("invalid SHL operand"),
        }
    }
}

impl Display for ShlInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for ShlInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct ShrInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl ShrInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A | RegisterKind::B | RegisterKind::C | RegisterKind::D => {
                Some(Self { mnemonic, register })
            }
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0x84]),
            RegisterKind::B => writer.write_all(&[0x85]),
            RegisterKind::C => writer.write_all(&[0x86]),
            RegisterKind::D => writer.write_all(&[0x87]),
            _ => unreachable!("invalid SHR operand"),
        }
    }
}

impl Display for ShrInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for ShrInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct NotInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl NotInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A | RegisterKind::B | RegisterKind::C | RegisterKind::D => {
                Some(Self { mnemonic, register })
            }
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0xEC]),
            RegisterKind::B => writer.write_all(&[0xED]),
            RegisterKind::C => writer.write_all(&[0xEE]),
            RegisterKind::D => writer.write_all(&[0xEF]),
            _ => unreachable!("invalid NOT operand"),
        }
    }
}

impl Display for NotInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for NotInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct TestInstruction {
    mnemonic: Mnemonic,
    register: Register,
}

impl TestInstruction {
    pub fn new(mnemonic: Mnemonic, register: Register) -> Option<Self> {
        match register.kind {
            RegisterKind::A | RegisterKind::B | RegisterKind::C | RegisterKind::D => {
                Some(Self { mnemonic, register })
            }
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.register.kind {
            RegisterKind::A => writer.write_all(&[0xFC]),
            RegisterKind::B => writer.write_all(&[0xFD]),
            RegisterKind::C => writer.write_all(&[0xFE]),
            RegisterKind::D => writer.write_all(&[0xFF]),
            _ => unreachable!("invalid TEST operand"),
        }
    }
}

impl Display for TestInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.register)
    }
}

impl Spanned for TestInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.register.span())
    }
}

#[derive(Clone, Debug)]
pub struct AddInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl AddInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::B)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0x88]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0x89]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0x8A]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0x8B]),
            (RegisterKind::B, RegisterKind::B) => writer.write_all(&[0x59]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0x8C]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0x8D]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0x8E]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0x8F]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0x90]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0x91]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0x92]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0x93]),
            _ => unreachable!("invalid ADD operand"),
        }
    }
}

impl Display for AddInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for AddInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct AddcInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl AddcInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::B)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0x94]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0x95]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0x96]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0x97]),
            (RegisterKind::B, RegisterKind::B) => writer.write_all(&[0x58]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0x98]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0x99]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0x9A]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0x9B]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0x9C]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0x9D]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0x9E]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0x9F]),
            _ => unreachable!("invalid ADDC operand"),
        }
    }
}

impl Display for AddcInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for AddcInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct SubInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl SubInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0xA8]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0xA9]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0xAA]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0xAB]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0xAC]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0xAD]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0xAE]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0xAF]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0xB0]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0xB1]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0xB2]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0xB3]),
            _ => unreachable!("invalid SUB operand"),
        }
    }
}

impl Display for SubInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for SubInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct SubbInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl SubbInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0xB4]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0xB5]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0xB6]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0xB7]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0xB8]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0xB9]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0xBA]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0xBB]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0xBC]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0xBD]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0xBE]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0xBF]),
            _ => unreachable!("invalid SUBB operand"),
        }
    }
}

impl Display for SubbInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for SubbInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct AndInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl AndInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0xC4]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0xC5]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0xC6]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0xC7]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0xC8]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0xC9]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0xCA]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0xCB]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0xCC]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0xCD]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0xCE]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0xCF]),
            _ => unreachable!("invalid AND operand"),
        }
    }
}

impl Display for AndInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for AndInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct OrInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl OrInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0xD0]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0xD1]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0xD2]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0xD3]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0xD4]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0xD5]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0xD6]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0xD7]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0xD8]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0xD9]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0xDA]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0xDB]),
            _ => unreachable!("invalid OR operand"),
        }
    }
}

impl Display for OrInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for OrInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct XorInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl XorInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::C)
            | (RegisterKind::D, RegisterKind::D) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0xDC]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0xDD]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0xDE]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0xDF]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0xE0]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0xE1]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0xE2]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0xE3]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0xE4]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0xE5]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0xE6]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0xE7]),
            (RegisterKind::A, RegisterKind::A) => writer.write_all(&[0xE8]),
            (RegisterKind::B, RegisterKind::B) => writer.write_all(&[0xE9]),
            (RegisterKind::C, RegisterKind::C) => writer.write_all(&[0xEA]),
            (RegisterKind::D, RegisterKind::D) => writer.write_all(&[0xEB]),
            _ => unreachable!("invalid XOR operand"),
        }
    }
}

impl Display for XorInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for XorInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct CmpInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl CmpInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::A, RegisterKind::B)
            | (RegisterKind::A, RegisterKind::C)
            | (RegisterKind::A, RegisterKind::D)
            | (RegisterKind::B, RegisterKind::A)
            | (RegisterKind::B, RegisterKind::C)
            | (RegisterKind::B, RegisterKind::D)
            | (RegisterKind::C, RegisterKind::A)
            | (RegisterKind::C, RegisterKind::B)
            | (RegisterKind::C, RegisterKind::D)
            | (RegisterKind::D, RegisterKind::A)
            | (RegisterKind::D, RegisterKind::B)
            | (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::A, RegisterKind::B) => writer.write_all(&[0xF0]),
            (RegisterKind::A, RegisterKind::C) => writer.write_all(&[0xF1]),
            (RegisterKind::A, RegisterKind::D) => writer.write_all(&[0xF2]),
            (RegisterKind::B, RegisterKind::A) => writer.write_all(&[0xF3]),
            (RegisterKind::B, RegisterKind::C) => writer.write_all(&[0xF4]),
            (RegisterKind::B, RegisterKind::D) => writer.write_all(&[0xF5]),
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0xF6]),
            (RegisterKind::C, RegisterKind::B) => writer.write_all(&[0xF7]),
            (RegisterKind::C, RegisterKind::D) => writer.write_all(&[0xF8]),
            (RegisterKind::D, RegisterKind::A) => writer.write_all(&[0xF9]),
            (RegisterKind::D, RegisterKind::B) => writer.write_all(&[0xFA]),
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0xFB]),
            _ => unreachable!("invalid CMP operand"),
        }
    }
}

impl Display for CmpInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for CmpInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct AddacInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl AddacInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::C, RegisterKind::A) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::C, RegisterKind::A) => writer.write_all(&[0x5A]),
            _ => unreachable!("invalid ADDAC operand"),
        }
    }
}

impl Display for AddacInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for AddacInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct SubaeInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: Register,
}

impl SubaeInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match (destination.kind, source.kind) {
            (RegisterKind::D, RegisterKind::C) => Some(Self {
                mnemonic,
                destination,
                comma,
                source,
            }),
            _ => None,
        }
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match (self.destination.kind, self.source.kind) {
            (RegisterKind::D, RegisterKind::C) => writer.write_all(&[0x2F]),
            _ => unreachable!("invalid ADDAC operand"),
        }
    }
}

impl Display for SubaeInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for SubaeInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub enum JumpTarget {
    Value(Expression),
    Register(Register),
}

impl Display for JumpTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Value(value) => Display::fmt(value, f),
            Self::Register(reg) => Display::fmt(reg, f),
        }
    }
}

impl Spanned for JumpTarget {
    fn span(&self) -> TextSpan {
        match self {
            Self::Value(value) => value.span(),
            Self::Register(reg) => reg.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallInstruction {
    mnemonic: Mnemonic,
    target: JumpTarget,
    emit_size: u16,
}

impl CallInstruction {
    pub fn new(mnemonic: Mnemonic, target: JumpTarget) -> Option<Self> {
        let emit_size = match &target {
            JumpTarget::Value(_) => 7,
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX | RegisterKind::DI => 3,
                _ => return None,
            },
        };

        Some(Self {
            mnemonic,
            target,
            emit_size,
        })
    }

    pub fn encode(
        &self,
        mut writer: impl std::io::Write,
        label_set: &HashMap<SharedStr, TextSpan>,
        label_values: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> std::io::Result<()> {
        match &self.target {
            JumpTarget::Value(target) => {
                let target = target.eval_or_zero(label_set, label_values, errors);
                let low = target as u8;
                let high = (target >> 8) as u8;

                writer.write_all(&[0x05, low, 0x06, high, 0x5C, 0x00, 0x00])
            }
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX => writer.write_all(&[0x5C, 0x00, 0x00]),
                RegisterKind::DI => writer.write_all(&[0x5D, 0x00, 0x00]),
                _ => unreachable!("invalid CALL operand"),
            },
        }
    }
}

impl Display for CallInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.target)
    }
}

impl Spanned for CallInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.target.span())
    }
}

#[derive(Clone, Debug)]
pub struct CallBdInstruction {
    mnemonic: Mnemonic,
    target: JumpTarget,
    emit_size: u16,
}

impl CallBdInstruction {
    pub fn new(mnemonic: Mnemonic, target: JumpTarget) -> Option<Self> {
        let emit_size = match &target {
            JumpTarget::Value(_) => 5,
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX | RegisterKind::DI => 1,
                _ => return None,
            },
        };

        Some(Self {
            mnemonic,
            target,
            emit_size,
        })
    }

    pub fn encode(
        &self,
        mut writer: impl std::io::Write,
        label_set: &HashMap<SharedStr, TextSpan>,
        label_values: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> std::io::Result<()> {
        match &self.target {
            JumpTarget::Value(target) => {
                let target = target.eval_or_zero(label_set, label_values, errors);
                let low = target as u8;
                let high = (target >> 8) as u8;

                writer.write_all(&[0x05, low, 0x06, high, 0x5C])
            }
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX => writer.write_all(&[0x5C]),
                RegisterKind::DI => writer.write_all(&[0x5D]),
                _ => unreachable!("invalid CALLBD operand"),
            },
        }
    }
}

impl Display for CallBdInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.target)
    }
}

impl Spanned for CallBdInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.target.span())
    }
}

#[derive(Clone, Debug)]
pub struct JmpInstruction {
    mnemonic: Mnemonic,
    target: JumpTarget,
    emit_size: u16,
}

impl JmpInstruction {
    pub fn new(mnemonic: Mnemonic, target: JumpTarget) -> Option<Self> {
        let emit_size = match &target {
            JumpTarget::Value(_) => 6,
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX | RegisterKind::DI => 2,
                _ => return None,
            },
        };

        Some(Self {
            mnemonic,
            target,
            emit_size,
        })
    }

    pub fn encode(
        &self,
        mut writer: impl std::io::Write,
        label_set: &HashMap<SharedStr, TextSpan>,
        label_values: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> std::io::Result<()> {
        match &self.target {
            JumpTarget::Value(target) => {
                let target = target.eval_or_zero(label_set, label_values, errors);
                let low = target as u8;
                let high = (target >> 8) as u8;

                writer.write_all(&[0x05, low, 0x06, high, 0x5F, 0x60])
            }
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX => writer.write_all(&[0x5F, 0x60]),
                RegisterKind::DI => writer.write_all(&[0x5F, 0x71]),
                _ => unreachable!("invalid JMP operand"),
            },
        }
    }
}

impl Display for JmpInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.target)
    }
}

impl Spanned for JmpInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.target.span())
    }
}

#[derive(Clone, Debug)]
pub struct BranchInstruction {
    mnemonic: Mnemonic,
    target: JumpTarget,
    emit_size: u16,
}

impl BranchInstruction {
    pub fn new(mnemonic: Mnemonic, target: JumpTarget) -> Option<Self> {
        let emit_size = match &target {
            JumpTarget::Value(_) => 6,
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX => 2,
                _ => return None,
            },
        };

        Some(Self {
            mnemonic,
            target,
            emit_size,
        })
    }

    pub fn encode(
        &self,
        mut writer: impl std::io::Write,
        label_set: &HashMap<SharedStr, TextSpan>,
        label_values: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> std::io::Result<()> {
        let opcode = match self.mnemonic.kind {
            MnemonicKind::Jo => 0x61,
            MnemonicKind::Jno => 0x62,
            MnemonicKind::Js => 0x63,
            MnemonicKind::Jns => 0x64,
            MnemonicKind::Jz => 0x65,
            MnemonicKind::Jnz => 0x66,
            MnemonicKind::Je => 0x65,
            MnemonicKind::Jne => 0x66,
            MnemonicKind::Jc => 0x67,
            MnemonicKind::Jnc => 0x68,
            MnemonicKind::Jnae => 0x68,
            MnemonicKind::Jb => 0x68,
            MnemonicKind::Jae => 0x67,
            MnemonicKind::Jnb => 0x67,
            MnemonicKind::Jbe => 0x69,
            MnemonicKind::Jna => 0x69,
            MnemonicKind::Ja => 0x6A,
            MnemonicKind::Jnbe => 0x6A,
            MnemonicKind::Jl => 0x6B,
            MnemonicKind::Jnge => 0x6B,
            MnemonicKind::Jge => 0x6C,
            MnemonicKind::Jnl => 0x6C,
            MnemonicKind::Jle => 0x6D,
            MnemonicKind::Jng => 0x6D,
            MnemonicKind::Jg => 0x6E,
            MnemonicKind::Jnle => 0x6E,
            MnemonicKind::Jlc => 0x6F,
            MnemonicKind::Jnlc => 0x70,
            _ => unreachable!("invalid branch mnemonic"),
        };

        match &self.target {
            JumpTarget::Value(target) => {
                let target = target.eval_or_zero(label_set, label_values, errors);
                let low = target as u8;
                let high = (target >> 8) as u8;

                writer.write_all(&[0x05, low, 0x06, high, 0x5F, opcode])
            }
            JumpTarget::Register(target) => match target.kind {
                RegisterKind::TX => writer.write_all(&[0x5F, opcode]),
                _ => unreachable!("invalid branch operand"),
            },
        }
    }
}

impl Display for BranchInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.mnemonic, self.target)
    }
}

impl Spanned for BranchInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.target.span())
    }
}

#[derive(Clone, Debug)]
pub struct InInstruction {
    mnemonic: Mnemonic,
    destination: Register,
    comma: Punctuation,
    source: IoRegister,
}

impl InInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: Register,
        comma: Punctuation,
        source: IoRegister,
    ) -> Option<Self> {
        if destination.kind != RegisterKind::A {
            return None;
        }

        if source.kind == IoRegisterKind::AudioData {
            return None;
        }

        Some(Self {
            mnemonic,
            destination,
            comma,
            source,
        })
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.source.kind {
            IoRegisterKind::UartData => writer.write_all(&[0x3A]),
            IoRegisterKind::UartControl => writer.write_all(&[0x3B]),
            IoRegisterKind::ControllerData => writer.write_all(&[0x3D]),
            IoRegisterKind::VgaStatus => writer.write_all(&[0x31]),
            IoRegisterKind::Gpio => writer.write_all(&[0x3E]),
            _ => unreachable!("invalid IN operand"),
        }
    }
}

impl Display for InInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for InInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub struct OutInstruction {
    mnemonic: Mnemonic,
    destination: IoRegister,
    comma: Punctuation,
    source: Register,
}

impl OutInstruction {
    pub fn new(
        mnemonic: Mnemonic,
        destination: IoRegister,
        comma: Punctuation,
        source: Register,
    ) -> Option<Self> {
        match destination.kind {
            IoRegisterKind::UartControl
            | IoRegisterKind::ControllerData
            | IoRegisterKind::VgaStatus => return None,
            _ => {}
        }

        if source.kind != RegisterKind::A {
            return None;
        }

        Some(Self {
            mnemonic,
            destination,
            comma,
            source,
        })
    }

    pub fn encode(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        match self.destination.kind {
            IoRegisterKind::UartData => writer.write_all(&[0x39]),
            IoRegisterKind::AudioData => writer.write_all(&[0x3C]),
            IoRegisterKind::Gpio => writer.write_all(&[0x37]),
            _ => unreachable!("invalid OUT operand"),
        }
    }
}

impl Display for OutInstruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} {}{} {}",
            self.mnemonic, self.destination, self.comma, self.source
        )
    }
}

impl Spanned for OutInstruction {
    fn span(&self) -> TextSpan {
        self.mnemonic.span().join(&self.source.span())
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Nop(Mnemonic),
    Break(Mnemonic),
    Lodsb(Mnemonic),
    Stosb(Mnemonic),
    Ret(Mnemonic),
    RetBd(Mnemonic),
    Clc(Mnemonic),
    Mov(MovInstruction),
    Inc(IncInstruction),
    Incc(InccInstruction),
    Dec(DecInstruction),
    Push(PushInstruction),
    Pop(PopInstruction),
    Shl(ShlInstruction),
    Shr(ShrInstruction),
    Not(NotInstruction),
    Test(TestInstruction),
    Add(AddInstruction),
    Addc(AddcInstruction),
    Sub(SubInstruction),
    Subb(SubbInstruction),
    And(AndInstruction),
    Or(OrInstruction),
    Xor(XorInstruction),
    Cmp(CmpInstruction),
    Addac(AddacInstruction),
    Subae(SubaeInstruction),
    Call(CallInstruction),
    CallBd(CallBdInstruction),
    Jmp(JmpInstruction),
    Branch(BranchInstruction),
    In(InInstruction),
    Out(OutInstruction),
}

impl Instruction {
    pub fn emit_size(&self) -> u16 {
        match self {
            Self::Nop(_) => 1,
            Self::Break(_) => 4,
            Self::Lodsb(_) => 1,
            Self::Stosb(_) => 1,
            Self::Ret(_) => 3,
            Self::RetBd(_) => 1,
            Self::Clc(_) => 1,
            Self::Mov(inst) => inst.emit_size,
            Self::Inc(_) => 1,
            Self::Incc(_) => 1,
            Self::Dec(_) => 1,
            Self::Push(inst) => inst.emit_size,
            Self::Pop(inst) => inst.emit_size,
            Self::Shl(_) => 1,
            Self::Shr(_) => 1,
            Self::Not(_) => 1,
            Self::Test(_) => 1,
            Self::Add(_) => 1,
            Self::Addc(_) => 1,
            Self::Sub(_) => 1,
            Self::Subb(_) => 1,
            Self::And(_) => 1,
            Self::Or(_) => 1,
            Self::Xor(_) => 1,
            Self::Cmp(_) => 1,
            Self::Addac(_) => 1,
            Self::Subae(_) => 1,
            Self::Call(inst) => inst.emit_size,
            Self::CallBd(inst) => inst.emit_size,
            Self::Jmp(inst) => inst.emit_size,
            Self::Branch(inst) => inst.emit_size,
            Self::In(_) => 1,
            Self::Out(_) => 1,
        }
    }

    pub fn encode(
        &self,
        mut writer: impl std::io::Write,
        label_set: &HashMap<SharedStr, TextSpan>,
        label_values: &HashMap<SharedStr, Option<i64>>,
        errors: &mut Vec<AssemblerError>,
    ) -> std::io::Result<()> {
        match self {
            Self::Nop(_) => writer.write_all(&[0x00]),
            Self::Break(_) => writer.write_all(&[0x00, 0x3F, 0x00, 0x00]),
            Self::Lodsb(_) => writer.write_all(&[0x5B]),
            Self::Stosb(_) => writer.write_all(&[0x7E]),
            Self::Ret(_) => writer.write_all(&[0x5E, 0x00, 0x00]),
            Self::RetBd(_) => writer.write_all(&[0x5E]),
            Self::Clc(_) => writer.write_all(&[0x7F]),
            Self::Mov(inst) => inst.encode(writer, label_set, label_values, errors),
            Self::Inc(inst) => inst.encode(writer),
            Self::Incc(inst) => inst.encode(writer),
            Self::Dec(inst) => inst.encode(writer),
            Self::Push(inst) => inst.encode(writer),
            Self::Pop(inst) => inst.encode(writer),
            Self::Shl(inst) => inst.encode(writer),
            Self::Shr(inst) => inst.encode(writer),
            Self::Not(inst) => inst.encode(writer),
            Self::Test(inst) => inst.encode(writer),
            Self::Add(inst) => inst.encode(writer),
            Self::Addc(inst) => inst.encode(writer),
            Self::Sub(inst) => inst.encode(writer),
            Self::Subb(inst) => inst.encode(writer),
            Self::And(inst) => inst.encode(writer),
            Self::Or(inst) => inst.encode(writer),
            Self::Xor(inst) => inst.encode(writer),
            Self::Cmp(inst) => inst.encode(writer),
            Self::Addac(inst) => inst.encode(writer),
            Self::Subae(inst) => inst.encode(writer),
            Self::Call(inst) => inst.encode(writer, label_set, label_values, errors),
            Self::CallBd(inst) => inst.encode(writer, label_set, label_values, errors),
            Self::Jmp(inst) => inst.encode(writer, label_set, label_values, errors),
            Self::Branch(inst) => inst.encode(writer, label_set, label_values, errors),
            Self::In(inst) => inst.encode(writer),
            Self::Out(inst) => inst.encode(writer),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Nop(mnemonic) => Display::fmt(mnemonic, f),
            Self::Break(mnemonic) => Display::fmt(mnemonic, f),
            Self::Lodsb(mnemonic) => Display::fmt(mnemonic, f),
            Self::Stosb(mnemonic) => Display::fmt(mnemonic, f),
            Self::Ret(mnemonic) => Display::fmt(mnemonic, f),
            Self::RetBd(mnemonic) => Display::fmt(mnemonic, f),
            Self::Clc(mnemonic) => Display::fmt(mnemonic, f),
            Self::Mov(inst) => Display::fmt(inst, f),
            Self::Inc(inst) => Display::fmt(inst, f),
            Self::Incc(inst) => Display::fmt(inst, f),
            Self::Dec(inst) => Display::fmt(inst, f),
            Self::Push(inst) => Display::fmt(inst, f),
            Self::Pop(inst) => Display::fmt(inst, f),
            Self::Shl(inst) => Display::fmt(inst, f),
            Self::Shr(inst) => Display::fmt(inst, f),
            Self::Not(inst) => Display::fmt(inst, f),
            Self::Test(inst) => Display::fmt(inst, f),
            Self::Add(inst) => Display::fmt(inst, f),
            Self::Addc(inst) => Display::fmt(inst, f),
            Self::Sub(inst) => Display::fmt(inst, f),
            Self::Subb(inst) => Display::fmt(inst, f),
            Self::And(inst) => Display::fmt(inst, f),
            Self::Or(inst) => Display::fmt(inst, f),
            Self::Xor(inst) => Display::fmt(inst, f),
            Self::Cmp(inst) => Display::fmt(inst, f),
            Self::Addac(inst) => Display::fmt(inst, f),
            Self::Subae(inst) => Display::fmt(inst, f),
            Self::Call(inst) => Display::fmt(inst, f),
            Self::CallBd(inst) => Display::fmt(inst, f),
            Self::Jmp(inst) => Display::fmt(inst, f),
            Self::Branch(inst) => Display::fmt(inst, f),
            Self::In(inst) => Display::fmt(inst, f),
            Self::Out(inst) => Display::fmt(inst, f),
        }
    }
}

impl Spanned for Instruction {
    fn span(&self) -> TextSpan {
        match self {
            Self::Nop(mnemonic) => mnemonic.span(),
            Self::Break(mnemonic) => mnemonic.span(),
            Self::Lodsb(mnemonic) => mnemonic.span(),
            Self::Stosb(mnemonic) => mnemonic.span(),
            Self::Ret(mnemonic) => mnemonic.span(),
            Self::RetBd(mnemonic) => mnemonic.span(),
            Self::Clc(mnemonic) => mnemonic.span(),
            Self::Mov(inst) => inst.span(),
            Self::Inc(inst) => inst.span(),
            Self::Incc(inst) => inst.span(),
            Self::Dec(inst) => inst.span(),
            Self::Push(inst) => inst.span(),
            Self::Pop(inst) => inst.span(),
            Self::Shl(inst) => inst.span(),
            Self::Shr(inst) => inst.span(),
            Self::Not(inst) => inst.span(),
            Self::Test(inst) => inst.span(),
            Self::Add(inst) => inst.span(),
            Self::Addc(inst) => inst.span(),
            Self::Sub(inst) => inst.span(),
            Self::Subb(inst) => inst.span(),
            Self::And(inst) => inst.span(),
            Self::Or(inst) => inst.span(),
            Self::Xor(inst) => inst.span(),
            Self::Cmp(inst) => inst.span(),
            Self::Addac(inst) => inst.span(),
            Self::Subae(inst) => inst.span(),
            Self::Call(inst) => inst.span(),
            Self::CallBd(inst) => inst.span(),
            Self::Jmp(inst) => inst.span(),
            Self::Branch(inst) => inst.span(),
            Self::In(inst) => inst.span(),
            Self::Out(inst) => inst.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    Label(Box<Label>),
    OffsetDirective(Box<OffsetDirective>),
    AlignDirective(Box<AlignDirective>),
    OriginDirective(Box<OriginDirective>),
    SectionDirective(Box<SectionDirective>),
    IncludeDirective(Box<IncludeDirective>),
    Instruction(Box<Instruction>),
}

impl Statement {
    pub fn emit_size(&self) -> u16 {
        match self {
            Self::Instruction(instruction) => instruction.emit_size(),
            _ => 0,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Label(label) => Display::fmt(label, f),
            Self::OffsetDirective(directive) => Display::fmt(directive, f),
            Self::AlignDirective(directive) => Display::fmt(directive, f),
            Self::OriginDirective(directive) => Display::fmt(directive, f),
            Self::SectionDirective(directive) => Display::fmt(directive, f),
            Self::IncludeDirective(directive) => Display::fmt(directive, f),
            Self::Instruction(inst) => Display::fmt(inst, f),
        }
    }
}

impl Spanned for Statement {
    fn span(&self) -> TextSpan {
        match self {
            Self::Label(label) => label.span(),
            Self::OffsetDirective(directive) => directive.span(),
            Self::AlignDirective(directive) => directive.span(),
            Self::OriginDirective(directive) => directive.span(),
            Self::SectionDirective(directive) => directive.span(),
            Self::IncludeDirective(directive) => directive.span(),
            Self::Instruction(inst) => inst.span(),
        }
    }
}
