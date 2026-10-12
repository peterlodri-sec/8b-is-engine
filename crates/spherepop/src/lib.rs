//! spherepop — the language cousin, as a clean-room, zero-dependency core.
//!
//! Spherepop (Flyxion / `standardgalactic`) draws nested parentheses as
//! nested bubbles: `(((1 + 2) + 3) + 4)` is a sphere holding a sphere holding
//! a sphere. You evaluate by *popping* — and only a bubble with nothing left
//! inside it may pop. Evaluation is inside-out, one admissible pop at a time.
//!
//! This crate is the engine's operational view of that idea, written from
//! the public description only (the upstream repository ships no licence, so
//! nothing is vendored). The canonical semantics — `Pop` / `Refuse` / `Bind`
//! / `Collapse` over an option space with an append-only event history —
//! live upstream in `standardgalactic/spherepop: SPEC.md` and its Rust
//! `spherepop-kernel/`. Here we keep the two laws the engine needs:
//!
//! 1. **Innermost-first is admissibility.** The admissible pops at any moment
//!    are exactly the frontier of unpopped bubbles with no unpopped children
//!    ([`Sphere::admissible`]). Anything else is *refused*, and the refusal
//!    is itself recorded — a refusal is an event, not an exception.
//! 2. **History is identity.** Every attempt appends to an append-only
//!    [`Event`] log. [`Sphere::replay`] rebuilds the exact state from source
//!    plus history, and [`Sphere::digest`] hashes that history: two runs that
//!    reach the same value by different pop orders are *different* spheres.
//!
//! `()`, `[]` and `{}` are synonyms, but a bubble must close with the kind
//! it opened with. Arithmetic is checked `i64` (`+ - * /`, usual precedence
//! inside a bubble); division by zero and overflow are refusals.
//!
//! ```
//! use spherepop::Sphere;
//!
//! let mut s = Sphere::parse("(((1 + 2) + 3) + 4)").unwrap();
//! assert_eq!(s.admissible(), vec![2]); // only the innermost may pop
//! assert!(s.pop(0).is_err());          // outer first is refused …
//! assert_eq!(s.pop(2), Ok(3));         // … inner first is admitted
//! assert_eq!(s.render(), "((3 + 3) + 4)");
//! assert_eq!(s.pop_all(), Ok(10));
//! assert_eq!(s.history().len(), 4);    // the refusal is part of the history
//! ```

use std::fmt;

/// Index of a bubble in pre-order (the order its opening bracket appears).
pub type BubbleId = usize;

/// A binary operator inside a bubble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            b'+' => Some(Op::Add),
            b'-' => Some(Op::Sub),
            b'*' => Some(Op::Mul),
            b'/' => Some(Op::Div),
            _ => None,
        }
    }

    /// The operator's source symbol.
    pub fn symbol(self) -> char {
        match self {
            Op::Add => '+',
            Op::Sub => '-',
            Op::Mul => '*',
            Op::Div => '/',
        }
    }

    fn apply(self, a: i64, b: i64) -> Result<i64, Refusal> {
        match self {
            Op::Add => a.checked_add(b).ok_or(Refusal::Overflow),
            Op::Sub => a.checked_sub(b).ok_or(Refusal::Overflow),
            Op::Mul => a.checked_mul(b).ok_or(Refusal::Overflow),
            Op::Div if b == 0 => Err(Refusal::DivisionByZero),
            Op::Div => a.checked_div(b).ok_or(Refusal::Overflow),
        }
    }
}

/// Which bracket pair drew a bubble. All three are synonyms; `Implicit` is
/// the unbracketed top level of an input like `(1 + 2) + 3`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delim {
    Round,
    Square,
    Curly,
    Implicit,
}

impl Delim {
    fn open(b: u8) -> Option<Self> {
        match b {
            b'(' => Some(Delim::Round),
            b'[' => Some(Delim::Square),
            b'{' => Some(Delim::Curly),
            _ => None,
        }
    }

    fn close_of(b: u8) -> Option<Self> {
        match b {
            b')' => Some(Delim::Round),
            b']' => Some(Delim::Square),
            b'}' => Some(Delim::Curly),
            _ => None,
        }
    }

    /// Opening and closing characters (empty for `Implicit`).
    pub fn chars(self) -> (&'static str, &'static str) {
        match self {
            Delim::Round => ("(", ")"),
            Delim::Square => ("[", "]"),
            Delim::Curly => ("{", "}"),
            Delim::Implicit => ("", ""),
        }
    }
}

/// An operand: a literal, or a nested bubble.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    Num(i64),
    Bubble(BubbleId),
}

/// One sphere in the nest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bubble {
    pub id: BubbleId,
    pub parent: Option<BubbleId>,
    pub delim: Delim,
    pub first: Operand,
    pub rest: Vec<(Op, Operand)>,
    /// `Some(v)` once popped.
    pub value: Option<i64>,
}

impl Bubble {
    /// Direct child bubbles, in source order.
    pub fn children(&self) -> impl Iterator<Item = BubbleId> + '_ {
        std::iter::once(self.first)
            .chain(self.rest.iter().map(|&(_, o)| o))
            .filter_map(|o| match o {
                Operand::Bubble(id) => Some(id),
                Operand::Num(_) => None,
            })
    }

    pub fn is_popped(&self) -> bool {
        self.value.is_some()
    }
}

/// Why a pop was refused. Refusals are recorded in the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// No bubble with that id.
    UnknownBubble,
    /// The bubble has already popped; a pop happens once.
    AlreadyPopped,
    /// An inner bubble must pop first (innermost-first).
    InnerFirst {
        blocking: BubbleId,
    },
    DivisionByZero,
    Overflow,
}

impl Refusal {
    fn code(self) -> (u8, u64) {
        match self {
            Refusal::UnknownBubble => (0, 0),
            Refusal::AlreadyPopped => (1, 0),
            Refusal::InnerFirst { blocking } => (2, blocking as u64),
            Refusal::DivisionByZero => (3, 0),
            Refusal::Overflow => (4, 0),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::UnknownBubble => write!(f, "no such bubble"),
            Refusal::AlreadyPopped => write!(f, "bubble already popped"),
            Refusal::InnerFirst { blocking } => {
                write!(f, "inner bubble {blocking} must pop first")
            }
            Refusal::DivisionByZero => write!(f, "division by zero"),
            Refusal::Overflow => write!(f, "arithmetic overflow"),
        }
    }
}

impl std::error::Error for Refusal {}

/// One entry of the append-only history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Pop { id: BubbleId, value: i64 },
    Refuse { id: BubbleId, reason: Refusal },
}

/// What went wrong while reading a Spherepop expression.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedChar(char),
    UnexpectedEnd,
    /// A bubble closed with a different bracket than it opened with.
    Mismatched,
    /// A bubble was never closed.
    Unclosed,
    /// `()` — a bubble must hold something.
    EmptyBubble,
    NumberOutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError {
    /// Byte offset into the source.
    pub pos: usize,
    pub kind: ParseErrorKind,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at byte {}: ", self.pos)?;
        match self.kind {
            ParseErrorKind::UnexpectedChar(c) => write!(f, "unexpected {c:?}"),
            ParseErrorKind::UnexpectedEnd => write!(f, "unexpected end of input"),
            ParseErrorKind::Mismatched => write!(f, "mismatched closing bracket"),
            ParseErrorKind::Unclosed => write!(f, "unclosed bubble"),
            ParseErrorKind::EmptyBubble => write!(f, "empty bubble"),
            ParseErrorKind::NumberOutOfRange => write!(f, "number out of i64 range"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Replay failed: the source did not parse, or the history diverged from
/// what the source admits at event `index`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayError {
    Parse(ParseError),
    Diverged { index: usize },
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplayError::Parse(e) => write!(f, "replay: {e}"),
            ReplayError::Diverged { index } => write!(f, "replay diverged at event {index}"),
        }
    }
}

impl std::error::Error for ReplayError {}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
    bubbles: Vec<Bubble>,
}

impl Parser<'_> {
    fn err(&self, kind: ParseErrorKind) -> ParseError {
        ParseError {
            pos: self.pos,
            kind,
        }
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn unexpected(&self) -> ParseError {
        match self.peek() {
            // Every token consumed so far is ASCII and the input is a `&str`,
            // so `pos` sits on a char boundary.
            Some(_) => {
                let c = std::str::from_utf8(&self.src[self.pos..])
                    .ok()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('\u{FFFD}');
                self.err(ParseErrorKind::UnexpectedChar(c))
            }
            None => self.err(ParseErrorKind::UnexpectedEnd),
        }
    }

    /// Reserve a bubble slot so ids follow opening-bracket (pre-) order.
    fn reserve(&mut self, parent: Option<BubbleId>, delim: Delim) -> BubbleId {
        let id = self.bubbles.len();
        self.bubbles.push(Bubble {
            id,
            parent,
            delim,
            first: Operand::Num(0),
            rest: Vec::new(),
            value: None,
        });
        id
    }

    fn number(&mut self) -> Result<i64, ParseError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while matches!(self.peek(), Some(b) if b.is_ascii_digit()) {
            self.pos += 1;
        }
        // Only ASCII was consumed, so this slice is valid UTF-8.
        let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or("");
        text.parse::<i64>().map_err(|_| ParseError {
            pos: start,
            kind: ParseErrorKind::NumberOutOfRange,
        })
    }

    fn operand(&mut self, parent: BubbleId) -> Result<Operand, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b) if b.is_ascii_digit() => Ok(Operand::Num(self.number()?)),
            Some(b'-') if matches!(self.src.get(self.pos + 1), Some(d) if d.is_ascii_digit()) => {
                Ok(Operand::Num(self.number()?))
            }
            Some(b) => match Delim::open(b) {
                Some(delim) => {
                    let open_at = self.pos;
                    self.pos += 1;
                    let id = self.reserve(Some(parent), delim);
                    self.skip_ws();
                    if self.peek().and_then(Delim::close_of).is_some() {
                        return Err(self.err(ParseErrorKind::EmptyBubble));
                    }
                    self.sequence(id)?;
                    self.skip_ws();
                    match self.peek() {
                        Some(c) if Delim::close_of(c) == Some(delim) => {
                            self.pos += 1;
                            Ok(Operand::Bubble(id))
                        }
                        Some(c) if Delim::close_of(c).is_some() => {
                            Err(self.err(ParseErrorKind::Mismatched))
                        }
                        None => Err(ParseError {
                            pos: open_at,
                            kind: ParseErrorKind::Unclosed,
                        }),
                        Some(_) => Err(self.unexpected()),
                    }
                }
                None => Err(self.unexpected()),
            },
            None => Err(self.err(ParseErrorKind::UnexpectedEnd)),
        }
    }

    /// `operand (op operand)*` into bubble `id`.
    fn sequence(&mut self, id: BubbleId) -> Result<(), ParseError> {
        let first = self.operand(id)?;
        let mut rest = Vec::new();
        loop {
            self.skip_ws();
            match self.peek().and_then(Op::from_byte) {
                Some(op) => {
                    self.pos += 1;
                    rest.push((op, self.operand(id)?));
                }
                None => break,
            }
        }
        self.bubbles[id].first = first;
        self.bubbles[id].rest = rest;
        Ok(())
    }
}

/// Usual precedence inside one bubble: fold `* /`, then `+ -`, left to right.
fn evaluate(first: i64, rest: &[(Op, i64)]) -> Result<i64, Refusal> {
    let mut total: i64 = 0;
    let mut pending = Op::Add;
    let mut term = first;
    for &(op, v) in rest {
        match op {
            Op::Mul | Op::Div => term = op.apply(term, v)?,
            Op::Add | Op::Sub => {
                total = pending.apply(total, term)?;
                pending = op;
                term = v;
            }
        }
    }
    pending.apply(total, term)
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// A parsed Spherepop expression, its pop state, and its history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sphere {
    source: String,
    bubbles: Vec<Bubble>,
    root: BubbleId,
    history: Vec<Event>,
}

impl Sphere {
    /// Read an expression. If the whole input is one bracketed bubble, that
    /// bubble is the root (id 0); otherwise an `Implicit` root wraps it.
    pub fn parse(source: &str) -> Result<Self, ParseError> {
        let mut p = Parser {
            src: source.as_bytes(),
            pos: 0,
            bubbles: Vec::new(),
        };
        let root = p.reserve(None, Delim::Implicit);
        p.sequence(root)?;
        p.skip_ws();
        if p.peek().is_some() {
            return Err(p.unexpected());
        }
        let mut bubbles = p.bubbles;
        // `(…)` alone: drop the implicit wrapper and renumber, so the visible
        // outer sphere is the root.
        if bubbles[0].rest.is_empty() && matches!(bubbles[0].first, Operand::Bubble(1)) {
            bubbles.remove(0);
            let shift = |o: Operand| match o {
                Operand::Bubble(i) => Operand::Bubble(i - 1),
                n => n,
            };
            for b in &mut bubbles {
                b.id -= 1;
                b.parent = match b.parent {
                    Some(0) | None => None,
                    Some(i) => Some(i - 1),
                };
                b.first = shift(b.first);
                for (_, o) in &mut b.rest {
                    *o = shift(*o);
                }
            }
        }
        Ok(Sphere {
            source: source.to_owned(),
            bubbles,
            root,
            history: Vec::new(),
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn root(&self) -> BubbleId {
        self.root
    }

    pub fn bubbles(&self) -> &[Bubble] {
        &self.bubbles
    }

    /// The append-only history: every pop and every refusal, in order.
    pub fn history(&self) -> &[Event] {
        &self.history
    }

    /// The root's value, once everything has popped.
    pub fn value(&self) -> Option<i64> {
        self.bubbles[self.root].value
    }

    fn first_unpopped_child(&self, id: BubbleId) -> Option<BubbleId> {
        self.bubbles[id]
            .children()
            .find(|&c| !self.bubbles[c].is_popped())
    }

    /// The admissible continuations: unpopped bubbles whose children have
    /// all popped — the innermost frontier, in ascending id order.
    pub fn admissible(&self) -> Vec<BubbleId> {
        self.bubbles
            .iter()
            .filter(|b| !b.is_popped() && self.first_unpopped_child(b.id).is_none())
            .map(|b| b.id)
            .collect()
    }

    fn try_pop(&self, id: BubbleId) -> Result<i64, Refusal> {
        let b = self.bubbles.get(id).ok_or(Refusal::UnknownBubble)?;
        if b.is_popped() {
            return Err(Refusal::AlreadyPopped);
        }
        if let Some(blocking) = self.first_unpopped_child(id) {
            return Err(Refusal::InnerFirst { blocking });
        }
        let resolve = |o: Operand| match o {
            Operand::Num(n) => n,
            // Every child has popped (checked above).
            Operand::Bubble(c) => self.bubbles[c].value.unwrap_or_default(),
        };
        let rest: Vec<(Op, i64)> = b.rest.iter().map(|&(op, o)| (op, resolve(o))).collect();
        evaluate(resolve(b.first), &rest)
    }

    /// Pop bubble `id`. Admitted pops and refusals are both appended to the
    /// history; only an admitted pop changes state.
    pub fn pop(&mut self, id: BubbleId) -> Result<i64, Refusal> {
        match self.try_pop(id) {
            Ok(value) => {
                self.bubbles[id].value = Some(value);
                self.history.push(Event::Pop { id, value });
                Ok(value)
            }
            Err(reason) => {
                self.history.push(Event::Refuse { id, reason });
                Err(reason)
            }
        }
    }

    /// Pop until the root pops, always taking the lowest admissible id —
    /// one deterministic innermost-first schedule. Stops at the first refusal.
    pub fn pop_all(&mut self) -> Result<i64, Refusal> {
        while let Some(&id) = self.admissible().first() {
            self.pop(id)?;
        }
        Ok(self.value().unwrap_or_default())
    }

    /// The sphere as it stands: popped bubbles shown as their values.
    /// The rendering is itself valid Spherepop source.
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.render_bubble(self.root, &mut out);
        out
    }

    fn render_operand(&self, o: Operand, out: &mut String) {
        match o {
            Operand::Num(n) => out.push_str(&n.to_string()),
            Operand::Bubble(id) => self.render_bubble(id, out),
        }
    }

    fn render_bubble(&self, id: BubbleId, out: &mut String) {
        let b = &self.bubbles[id];
        if let Some(v) = b.value {
            out.push_str(&v.to_string());
            return;
        }
        let (open, close) = b.delim.chars();
        out.push_str(open);
        self.render_operand(b.first, out);
        for &(op, o) in &b.rest {
            out.push(' ');
            out.push(op.symbol());
            out.push(' ');
            self.render_operand(o, out);
        }
        out.push_str(close);
    }

    /// FNV-1a over source and history. Identity is the history, not the
    /// value: same answer, different pop order → different digest.
    pub fn digest(&self) -> u64 {
        let mut h = fnv(FNV_OFFSET, self.source.as_bytes());
        for e in &self.history {
            match *e {
                Event::Pop { id, value } => {
                    h = fnv(h, &[0]);
                    h = fnv(h, &(id as u64).to_le_bytes());
                    h = fnv(h, &value.to_le_bytes());
                }
                Event::Refuse { id, reason } => {
                    let (code, arg) = reason.code();
                    h = fnv(h, &[1, code]);
                    h = fnv(h, &(id as u64).to_le_bytes());
                    h = fnv(h, &arg.to_le_bytes());
                }
            }
        }
        h
    }

    /// Rebuild a sphere from its source and history, checking every event
    /// is exactly what the source admits at that point.
    pub fn replay(source: &str, history: &[Event]) -> Result<Self, ReplayError> {
        let mut s = Sphere::parse(source).map_err(ReplayError::Parse)?;
        for (index, e) in history.iter().enumerate() {
            let (id, expected) = match *e {
                Event::Pop { id, value } => (id, Ok(value)),
                Event::Refuse { id, reason } => (id, Err(reason)),
            };
            if s.pop(id) != expected {
                return Err(ReplayError::Diverged { index });
            }
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(src: &str) -> Result<i64, Refusal> {
        Sphere::parse(src).expect("parses").pop_all()
    }

    #[test]
    fn site_examples_evaluate() {
        assert_eq!(eval("(((1 + 2) + 3) + 4)"), Ok(10));
        assert_eq!(eval("(((1 + 2) + (3 + 4)) + (5 + 6))"), Ok(21));
    }

    #[test]
    fn brackets_are_synonyms() {
        assert_eq!(eval("{[(1 + 2) + 3] + 4}"), Ok(10));
        assert_eq!(eval("[1+2]*{3}"), Ok(9));
    }

    #[test]
    fn precedence_and_negatives_inside_a_bubble() {
        assert_eq!(eval("(2 + 3 * 4 - 6 / 2)"), Ok(11));
        assert_eq!(eval("(1 - -3)"), Ok(4));
        assert_eq!(eval("-5 * (2 - 4)"), Ok(10));
        assert_eq!(eval("7"), Ok(7));
    }

    #[test]
    fn single_outer_bubble_is_the_root() {
        let s = Sphere::parse("((1 + 2) + 3)").unwrap();
        assert_eq!(s.root(), 0);
        assert_eq!(s.bubbles().len(), 2);
        assert_eq!(s.bubbles()[0].delim, Delim::Round);
        assert_eq!(s.bubbles()[1].parent, Some(0));
        assert_eq!(s.bubbles()[0].parent, None);

        let t = Sphere::parse("(1 + 2) + 3").unwrap();
        assert_eq!(t.bubbles()[0].delim, Delim::Implicit);
        assert_eq!(t.bubbles().len(), 2);
    }

    #[test]
    fn admissible_is_the_innermost_frontier() {
        let s = Sphere::parse("(((1 + 2) + (3 + 4)) + (5 + 6))").unwrap();
        // ids in opening-bracket order: 0 outer, 1 left, 2 (1+2), 3 (3+4), 4 (5+6)
        assert_eq!(s.admissible(), vec![2, 3, 4]);
    }

    #[test]
    fn outer_first_is_refused_and_recorded() {
        let mut s = Sphere::parse("((1 + 2) + 3)").unwrap();
        assert_eq!(s.pop(0), Err(Refusal::InnerFirst { blocking: 1 }));
        assert_eq!(s.value(), None);
        assert_eq!(s.pop(1), Ok(3));
        assert_eq!(s.pop(1), Err(Refusal::AlreadyPopped));
        assert_eq!(s.pop(9), Err(Refusal::UnknownBubble));
        assert_eq!(s.pop(0), Ok(6));
        assert_eq!(
            s.history(),
            &[
                Event::Refuse {
                    id: 0,
                    reason: Refusal::InnerFirst { blocking: 1 }
                },
                Event::Pop { id: 1, value: 3 },
                Event::Refuse {
                    id: 1,
                    reason: Refusal::AlreadyPopped
                },
                Event::Refuse {
                    id: 9,
                    reason: Refusal::UnknownBubble
                },
                Event::Pop { id: 0, value: 6 },
            ]
        );
    }

    #[test]
    fn render_shows_the_sphere_and_reparses() {
        let mut s = Sphere::parse("(((1 + 2) + (3 + 4)) + [5 + 6])").unwrap();
        s.pop(3).unwrap();
        assert_eq!(s.render(), "(((1 + 2) + 7) + [5 + 6])");
        // A render is valid source and reaches the same value.
        assert_eq!(eval(&s.render()), Ok(21));
        s.pop_all().unwrap();
        assert_eq!(s.render(), "21");
    }

    #[test]
    fn pop_order_is_confluent_but_history_is_identity() {
        let src = "((1 + 2) + (3 + 4))";
        let mut a = Sphere::parse(src).unwrap();
        a.pop(1).unwrap();
        a.pop(2).unwrap();
        a.pop(0).unwrap();
        let mut b = Sphere::parse(src).unwrap();
        b.pop(2).unwrap();
        b.pop(1).unwrap();
        b.pop(0).unwrap();
        assert_eq!(a.value(), b.value());
        assert_ne!(a.digest(), b.digest());
    }

    #[test]
    fn replay_reproduces_state_and_rejects_divergence() {
        let src = "{(10 / 2) * [3 - 1]}";
        let mut s = Sphere::parse(src).unwrap();
        let _ = s.pop(0);
        s.pop_all().unwrap();
        let r = Sphere::replay(src, s.history()).unwrap();
        assert_eq!(r, s);
        assert_eq!(r.digest(), s.digest());

        let mut forged = s.history().to_vec();
        forged[1] = Event::Pop { id: 1, value: 999 };
        assert_eq!(
            Sphere::replay(src, &forged),
            Err(ReplayError::Diverged { index: 1 })
        );
    }

    #[test]
    fn arithmetic_refusals() {
        let mut s = Sphere::parse("((1 - 1) + (4 / (2 - 2)))").unwrap();
        assert_eq!(s.pop_all(), Err(Refusal::DivisionByZero));
        assert_eq!(s.value(), None);
        assert!(matches!(
            s.history().last(),
            Some(Event::Refuse {
                reason: Refusal::DivisionByZero,
                ..
            })
        ));
        assert_eq!(eval("(9223372036854775807 + 1)"), Err(Refusal::Overflow));
        assert_eq!(eval("(-9223372036854775808 / -1)"), Err(Refusal::Overflow));
    }

    #[test]
    fn parse_errors() {
        let kind = |s: &str| Sphere::parse(s).unwrap_err().kind;
        assert_eq!(kind("(1 + 2]"), ParseErrorKind::Mismatched);
        assert_eq!(kind("((1 + 2)"), ParseErrorKind::Unclosed);
        assert_eq!(kind("()"), ParseErrorKind::EmptyBubble);
        assert_eq!(kind("(1 +)"), ParseErrorKind::UnexpectedChar(')'));
        assert_eq!(kind("1 + "), ParseErrorKind::UnexpectedEnd);
        assert_eq!(kind("(1 2)"), ParseErrorKind::UnexpectedChar('2'));
        assert_eq!(kind("(1) )"), ParseErrorKind::UnexpectedChar(')'));
        assert_eq!(kind("x"), ParseErrorKind::UnexpectedChar('x'));
        assert_eq!(
            kind("99999999999999999999"),
            ParseErrorKind::NumberOutOfRange
        );
        assert_eq!(Sphere::parse("((1 + 2)").unwrap_err().pos, 0);
    }
}
