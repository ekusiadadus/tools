use std::{fmt, iter, marker::PhantomData, ops::Range};

use crate::cursor::SyntaxSlot;
use crate::{
	cursor::{self},
	Direction, GreenNode, NodeOrToken, SyntaxKind, SyntaxText, TextRange, TextSize, TokenAtOffset,
	WalkEvent,
};

pub trait Language: Sized + Clone + Copy + fmt::Debug + Eq + Ord + std::hash::Hash {
	type Kind: fmt::Debug + PartialEq;

	fn kind_from_raw(raw: SyntaxKind) -> Self::Kind;
	fn kind_to_raw(kind: Self::Kind) -> SyntaxKind;
	fn list_kind() -> Self::Kind;
}

#[derive(Debug, Default, Hash, Copy, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct RawLanguage;

impl Language for RawLanguage {
	type Kind = SyntaxKind;

	fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
		raw
	}

	fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
		kind
	}

	fn list_kind() -> Self::Kind {
		SyntaxKind(0)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TriviaPiece {
	Whitespace(usize),
	Comments(usize),
}

impl TriviaPiece {
	#[inline]
	pub fn text_len(&self) -> TextSize {
		match self {
			TriviaPiece::Whitespace(n) => (*n as u32).into(),
			TriviaPiece::Comments(n) => (*n as u32).into(),
		}
	}
}

pub struct SyntaxTriviaPieceWhitespace<L: Language>(SyntaxTriviaPiece<L>);
pub struct SyntaxTriviaPieceComments<L: Language>(SyntaxTriviaPiece<L>);

/// [SyntaxTriviaPiece] gives access to the most granular information about the trivia
/// that was specified by the lexer at the token creation time.
///
/// For example:
///
/// ```ignore
/// builder.token_with_trivia(
///     SyntaxKind(1),
///     "\n\t /**/let \t\t",
///     vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
///     vec![TriviaPiece::Whitespace(3)],
/// );
/// });
///
/// This token has two pieces in the leading trivia, and one piece at the trailing trivia. Each
/// piece is defined by the [TriviaPiece]; its content is irrelevant.
/// ```
#[derive(Clone)]
pub struct SyntaxTriviaPiece<L: Language> {
	raw: cursor::SyntaxTrivia,
	offset: TextSize,
	trivia: TriviaPiece,
	_p: PhantomData<L>,
}

impl<L: Language> SyntaxTriviaPiece<L> {
	/// Returns the associated text just for this trivia piece. This is different from [SyntaxTrivia::text()],
	/// which returns the text of the whole trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// use std::iter::Iterator;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t /**/let \t\t",
	///         vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
	/// assert_eq!("\n\t ", pieces[0].text());
	/// ```
	pub fn text(&self) -> &str {
		let txt = self.raw.text();
		let start = self.offset - self.raw.offset();
		let end = start + self.text_len();

		&txt[start.into()..end.into()]
	}

	/// Returns the associated text length just for this trivia piece. This is different from [SyntaxTrivia::text_len()],
	/// which returns the text length of the whole trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// use std::iter::Iterator;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t /**/let \t\t",
	///         vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
	/// assert_eq!(TextSize::from(3), pieces[0].text_len());
	/// ```
	pub fn text_len(&self) -> TextSize {
		self.trivia.text_len()
	}

	/// Returns the associated text range just for this trivia piece. This is different from [SyntaxTrivia::text_range()],
	/// which returns the text range of the whole trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// use std::iter::Iterator;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t /**/let \t\t",
	///         vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
	/// assert_eq!(TextRange::new(0.into(), 3.into()), pieces[0].text_range());
	/// ```
	pub fn text_range(&self) -> TextRange {
		TextRange::at(self.offset, self.text_len())
	}

	/// Cast this trivia piece to [SyntaxTriviaPieceWhitespace].
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// use std::iter::Iterator;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t /**/let \t\t",
	///         vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
	/// let w = pieces[0].as_whitespace();
	/// assert!(w.is_some());
	/// let w = pieces[1].as_whitespace();
	/// assert!(w.is_none());
	/// ```
	pub fn as_whitespace(&self) -> Option<SyntaxTriviaPieceWhitespace<L>> {
		match &self.trivia {
			TriviaPiece::Whitespace(_) => Some(SyntaxTriviaPieceWhitespace(self.clone())),
			_ => None,
		}
	}

	/// Cast this trivia piece to [SyntaxTriviaPieceComments].
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// use std::iter::Iterator;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t /**/let \t\t",
	///         vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
	/// let w = pieces[0].as_comments();
	/// assert!(w.is_none());
	/// let w = pieces[1].as_comments();
	/// assert!(w.is_some());
	/// ```
	pub fn as_comments(&self) -> Option<SyntaxTriviaPieceComments<L>> {
		match &self.trivia {
			TriviaPiece::Comments(_) => Some(SyntaxTriviaPieceComments(self.clone())),
			_ => None,
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxTrivia<L: Language> {
	raw: cursor::SyntaxTrivia,
	_p: PhantomData<L>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxNode<L: Language> {
	raw: cursor::SyntaxNode,
	_p: PhantomData<L>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken<L: Language> {
	raw: cursor::SyntaxToken,
	_p: PhantomData<L>,
}

pub type SyntaxElement<L> = NodeOrToken<SyntaxNode<L>, SyntaxToken<L>>;

impl<L: Language> fmt::Debug for SyntaxNode<L> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			let mut level = 0;
			for event in self.raw.preorder_slots() {
				match event {
					WalkEvent::Enter(element) => {
						for _ in 0..level {
							write!(f, "  ")?;
						}
						match element {
							SyntaxSlot::Node(node) => {
								writeln!(f, "{}: {:?}", node.index(), SyntaxNode::<L>::from(node))?
							}
							SyntaxSlot::Token(token) => writeln!(
								f,
								"{}: {:?}",
								token.index(),
								SyntaxToken::<L>::from(token)
							)?,
							SyntaxSlot::Empty { index, .. } => writeln!(f, "{}: (empty)", index)?,
						}
						level += 1;
					}
					WalkEvent::Leave(_) => level -= 1,
				}
			}
			assert_eq!(level, 0);
			Ok(())
		} else {
			write!(f, "{:?}@{:?}", self.kind(), self.text_range())
		}
	}
}

impl<L: Language> fmt::Display for SyntaxNode<L> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.raw, f)
	}
}

fn print_debug_str<S: AsRef<str>>(text: S, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	let text = text.as_ref();
	if text.len() < 25 {
		return write!(f, "{:?}", text);
	} else {
		for idx in 21..25 {
			if text.is_char_boundary(idx) {
				let text = format!("{} ...", &text[..idx]);
				return write!(f, "{:?}", text);
			}
		}
		return write!(f, "");
	}
}

fn print_debug_trivia_piece<L: Language>(
	piece: SyntaxTriviaPiece<L>,
	f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
	match piece.trivia {
		TriviaPiece::Whitespace(_) => {
			write!(f, "Whitespace(")?;
			print_debug_str(piece.text(), f)?;
			write!(f, ")")
		}
		TriviaPiece::Comments(_) => {
			write!(f, "Comments(")?;
			print_debug_str(piece.text(), f)?;
			write!(f, ")")
		}
	}
}

impl<L: Language> fmt::Debug for SyntaxToken<L> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{:?}@{:?} {:?} ",
			self.kind(),
			self.text_range(),
			self.text_trimmed()
		)?;

		write!(f, "[")?;
		let mut first_piece = true;
		for piece in self.leading_trivia().pieces() {
			if !first_piece {
				write!(f, ", ")?;
			}
			first_piece = false;
			print_debug_trivia_piece(piece, f)?;
		}
		write!(f, "] [")?;

		let mut first_piece = true;
		for piece in self.trailing_trivia().pieces() {
			if !first_piece {
				write!(f, ", ")?;
			}
			first_piece = false;
			print_debug_trivia_piece(piece, f)?;
		}
		write!(f, "]")
	}
}

impl<L: Language> fmt::Display for SyntaxToken<L> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.raw, f)
	}
}

impl<L: Language> From<SyntaxNode<L>> for SyntaxElement<L> {
	fn from(node: SyntaxNode<L>) -> SyntaxElement<L> {
		NodeOrToken::Node(node)
	}
}

impl<L: Language> From<SyntaxToken<L>> for SyntaxElement<L> {
	fn from(token: SyntaxToken<L>) -> SyntaxElement<L> {
		NodeOrToken::Token(token)
	}
}

pub struct SyntaxTriviaPiecesIterator<L: Language> {
	iter: cursor::SyntaxTriviaPiecesIterator,
	_p: PhantomData<L>,
}

impl<L: Language> Iterator for SyntaxTriviaPiecesIterator<L> {
	type Item = SyntaxTriviaPiece<L>;

	fn next(&mut self) -> Option<Self::Item> {
		let (offset, trivia) = self.iter.next()?;
		Some(SyntaxTriviaPiece {
			raw: self.iter.raw.clone(),
			offset,
			trivia,
			_p: PhantomData,
		})
	}
}

impl<L: Language> SyntaxTrivia<L> {
	/// Returns all [SyntaxTriviaPiece] of this trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// use std::iter::Iterator;
	/// use crate::*;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	/// builder.token_with_trivia(
	///     SyntaxKind(1),
	///     "\n\t /**/let \t\t",
	///     vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
	///     vec![TriviaPiece::Whitespace(3)],
	/// );
	/// });
	/// let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
	/// assert_eq!(2, pieces.len());
	/// let pieces: Vec<_> = node.last_trailing_trivia().unwrap().pieces().collect();
	/// assert_eq!(1, pieces.len());
	/// ```
	pub fn pieces(&self) -> SyntaxTriviaPiecesIterator<L> {
		SyntaxTriviaPiecesIterator {
			iter: self.raw.pieces(),
			_p: PhantomData,
		}
	}

	pub fn text(&self) -> &str {
		self.raw.text()
	}

	pub fn text_range(&self) -> TextRange {
		self.raw.text_range()
	}
}

impl<L: Language> SyntaxNode<L> {
	pub(crate) fn new_root(green: GreenNode) -> SyntaxNode<L> {
		SyntaxNode::from(cursor::SyntaxNode::new_root(green))
	}

	/// Returns the element stored in the slot with the given index. Returns [None] if the slot is empty.
	///
	/// ## Panics
	/// If the slot index is out of bounds
	pub fn element_in_slot(&self, slot: u32) -> Option<SyntaxElement<L>> {
		self.raw.element_in_slot(slot).map(SyntaxElement::from)
	}

	pub fn kind(&self) -> L::Kind {
		L::kind_from_raw(self.raw.kind())
	}

	/// Returns the text of all descendants tokens combined, including all trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	///     builder.token(SyntaxKind(1), "a");
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "; \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// assert_eq!("\n\t let \t\ta; \t\t", node.text());
	/// ```
	pub fn text(&self) -> SyntaxText {
		self.raw.text()
	}

	/// Returns the text of all descendants tokens combined,
	/// excluding the first token leading trivia, and the last token trailing trivia.
	/// All other trivia is included.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	///     builder.token(SyntaxKind(1), "a");
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "; \t\t",
	///         vec![],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// assert_eq!("let \t\ta;", node.text_trimmed());
	/// ```
	pub fn text_trimmed(&self) -> SyntaxText {
		self.raw.text_trimmed()
	}

	/// Returns the range corresponding for the text of all descendants tokens combined, including all trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	///     builder.token(SyntaxKind(1), "a");
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "; \t\t",
	///         vec![],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let range = node.text_range();
	/// assert_eq!(0u32, range.start().into());
	/// assert_eq!(14u32, range.end().into());
	/// ```
	pub fn text_range(&self) -> TextRange {
		self.raw.text_range()
	}

	/// Returns the range corresponding for the text of all descendants tokens combined,
	/// excluding the first token leading  trivia, and the last token trailing trivia.
	/// All other trivia is included.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	///     builder.token(SyntaxKind(1), "a");
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "; \t\t",
	///         vec![],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let range = node.text_trimmed_range();
	/// assert_eq!(3u32, range.start().into());
	/// assert_eq!(11u32, range.end().into());
	/// ```
	pub fn text_trimmed_range(&self) -> TextRange {
		self.raw.text_trimmed_range()
	}

	/// Returns the leading trivia of the [first_token](SyntaxNode::first_token), or [None] if the node does not have any descendant tokens.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	///     builder.token(SyntaxKind(1), "a");
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "; \t\t",
	///         vec![],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let trivia = node.first_leading_trivia();
	/// assert!(trivia.is_some());
	/// assert_eq!("\n\t ", trivia.unwrap().text());
	///
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {});
	/// let trivia = node.first_leading_trivia();
	/// assert!(trivia.is_none());
	/// ```
	pub fn first_leading_trivia(&self) -> Option<SyntaxTrivia<L>> {
		self.raw.first_leading_trivia().map(|raw| SyntaxTrivia {
			raw,
			_p: PhantomData,
		})
	}

	/// Returns the trailing trivia of the  [last_token](SyntaxNode::last_token), or [None] if the node does not have any descendant tokens.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	///     builder.token(SyntaxKind(1), "a");
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "; \t\t",
	///         vec![],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// });
	/// let trivia = node.last_trailing_trivia();
	/// assert!(trivia.is_some());
	/// assert_eq!(" \t\t", trivia.unwrap().text());
	///
	/// let mut node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {});
	/// let trivia = node.last_trailing_trivia();
	/// assert!(trivia.is_none());
	/// ```
	pub fn last_trailing_trivia(&self) -> Option<SyntaxTrivia<L>> {
		self.raw.last_trailing_trivia().map(|raw| SyntaxTrivia {
			raw,
			_p: PhantomData,
		})
	}

	pub fn parent(&self) -> Option<SyntaxNode<L>> {
		self.raw.parent().map(Self::from)
	}

	pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> {
		self.raw.ancestors().map(SyntaxNode::from)
	}

	pub fn children(&self) -> SyntaxNodeChildren<L> {
		SyntaxNodeChildren {
			raw: self.raw.children(),
			_p: PhantomData,
		}
	}

	pub fn children_with_tokens(&self) -> SyntaxElementChildren<L> {
		SyntaxElementChildren {
			raw: self.raw.children_with_tokens(),
			_p: PhantomData,
		}
	}

	pub fn first_child(&self) -> Option<SyntaxNode<L>> {
		self.raw.first_child().map(Self::from)
	}
	pub fn last_child(&self) -> Option<SyntaxNode<L>> {
		self.raw.last_child().map(Self::from)
	}

	pub fn first_child_or_token(&self) -> Option<SyntaxElement<L>> {
		self.raw.first_child_or_token().map(NodeOrToken::from)
	}
	pub fn last_child_or_token(&self) -> Option<SyntaxElement<L>> {
		self.raw.last_child_or_token().map(NodeOrToken::from)
	}

	pub fn next_sibling(&self) -> Option<SyntaxNode<L>> {
		self.raw.next_sibling().map(Self::from)
	}
	pub fn prev_sibling(&self) -> Option<SyntaxNode<L>> {
		self.raw.prev_sibling().map(Self::from)
	}

	pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
		self.raw.next_sibling_or_token().map(NodeOrToken::from)
	}
	pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
		self.raw.prev_sibling_or_token().map(NodeOrToken::from)
	}

	/// Return the leftmost token in the subtree of this node.
	pub fn first_token(&self) -> Option<SyntaxToken<L>> {
		self.raw.first_token().map(SyntaxToken::from)
	}
	/// Return the rightmost token in the subtree of this node.
	pub fn last_token(&self) -> Option<SyntaxToken<L>> {
		self.raw.last_token().map(SyntaxToken::from)
	}

	pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = SyntaxNode<L>> {
		self.raw.siblings(direction).map(SyntaxNode::from)
	}

	pub fn siblings_with_tokens(
		&self,
		direction: Direction,
	) -> impl Iterator<Item = SyntaxElement<L>> {
		self.raw
			.siblings_with_tokens(direction)
			.map(SyntaxElement::from)
	}

	pub fn descendants(&self) -> impl Iterator<Item = SyntaxNode<L>> {
		self.raw.descendants().map(SyntaxNode::from)
	}

	pub fn descendants_tokens(&self) -> impl Iterator<Item = SyntaxToken<L>> {
		self.descendants_with_tokens()
			.filter_map(|x| x.as_token().cloned())
	}

	pub fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement<L>> {
		self.raw.descendants_with_tokens().map(NodeOrToken::from)
	}

	/// Traverse the subtree rooted at the current node (including the current
	/// node) in preorder, excluding tokens.
	pub fn preorder(&self) -> Preorder<L> {
		Preorder {
			raw: self.raw.preorder(),
			_p: PhantomData,
		}
	}

	/// Traverse the subtree rooted at the current node (including the current
	/// node) in preorder, including tokens.
	pub fn preorder_with_tokens(&self) -> PreorderWithTokens<L> {
		PreorderWithTokens {
			raw: self.raw.preorder_with_tokens(),
			_p: PhantomData,
		}
	}

	/// Find a token in the subtree corresponding to this node, which covers the offset.
	/// Precondition: offset must be withing node's range.
	pub fn token_at_offset(&self, offset: TextSize) -> TokenAtOffset<SyntaxToken<L>> {
		self.raw.token_at_offset(offset).map(SyntaxToken::from)
	}

	/// Return the deepest node or token in the current subtree that fully
	/// contains the range. If the range is empty and is contained in two leaf
	/// nodes, either one can be returned. Precondition: range must be contained
	/// withing the current node
	pub fn covering_element(&self, range: TextRange) -> SyntaxElement<L> {
		NodeOrToken::from(self.raw.covering_element(range))
	}

	/// Finds a [`SyntaxElement`] which intersects with a given `range`. If
	/// there are several intersecting elements, any one can be returned.
	///
	/// The method uses binary search internally, so it's complexity is
	/// `O(log(N))` where `N = self.children_with_tokens().count()`.
	pub fn child_or_token_at_range(&self, range: TextRange) -> Option<SyntaxElement<L>> {
		self.raw
			.child_or_token_at_range(range)
			.map(SyntaxElement::from)
	}

	/// Returns an independent copy of the subtree rooted at this node.
	///
	/// The parent of the returned node will be `None`, the start offset will be
	/// zero, but, otherwise, it'll be equivalent to the source node.
	pub fn clone_subtree(&self) -> SyntaxNode<L> {
		SyntaxNode::from(self.raw.clone_subtree())
	}

	pub fn clone_for_update(&self) -> SyntaxNode<L> {
		SyntaxNode::from(self.raw.clone_for_update())
	}

	pub fn detach(&self) {
		self.raw.detach()
	}

	pub fn splice_children(&self, to_delete: Range<usize>, to_insert: Vec<SyntaxElement<L>>) {
		let to_insert = to_insert
			.into_iter()
			.map(cursor::SyntaxElement::from)
			.collect::<Vec<_>>();
		self.raw.splice_children(to_delete, to_insert)
	}

	pub fn into_list(self) -> Option<SyntaxList<L>> {
		if self.kind() == L::list_kind() {
			Some(SyntaxList::new(self))
		} else {
			None
		}
	}
}

impl<L: Language> SyntaxToken<L> {
	pub fn kind(&self) -> L::Kind {
		L::kind_from_raw(self.raw.kind())
	}

	pub fn text_range(&self) -> TextRange {
		self.raw.text_range()
	}

	pub fn text_trimmed_range(&self) -> TextRange {
		self.raw.text_trimmed_range()
	}

	pub fn index(&self) -> usize {
		self.raw.index()
	}

	/// Returns the text of the token, including all trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut token = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// }).first_token().unwrap();
	/// assert_eq!("\n\t let \t\t", token.text());
	/// ```
	pub fn text(&self) -> &str {
		self.raw.text()
	}

	/// Returns the text of the token, excluding all trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut token = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// }).first_token().unwrap();
	/// assert_eq!("let", token.text_trimmed());
	/// ```
	pub fn text_trimmed(&self) -> &str {
		self.raw.text_trimmed()
	}

	pub fn parent(&self) -> Option<SyntaxNode<L>> {
		self.raw.parent().map(SyntaxNode::from)
	}

	pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> {
		self.raw.ancestors().map(SyntaxNode::from)
	}

	pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
		self.raw.next_sibling_or_token().map(NodeOrToken::from)
	}
	pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
		self.raw.prev_sibling_or_token().map(NodeOrToken::from)
	}

	pub fn siblings_with_tokens(
		&self,
		direction: Direction,
	) -> impl Iterator<Item = SyntaxElement<L>> {
		self.raw
			.siblings_with_tokens(direction)
			.map(SyntaxElement::from)
	}

	/// Next token in the tree (i.e, not necessary a sibling).
	pub fn next_token(&self) -> Option<SyntaxToken<L>> {
		self.raw.next_token().map(SyntaxToken::from)
	}
	/// Previous token in the tree (i.e, not necessary a sibling).
	pub fn prev_token(&self) -> Option<SyntaxToken<L>> {
		self.raw.prev_token().map(SyntaxToken::from)
	}

	pub fn detach(&self) {
		self.raw.detach()
	}

	/// Returns the token leading trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut token = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// }).first_token().unwrap();
	/// assert_eq!("\n\t ", token.leading_trivia().text());
	/// ```
	#[inline]
	pub fn leading_trivia(&self) -> SyntaxTrivia<L> {
		SyntaxTrivia {
			raw: self.raw.leading_trivia(),
			_p: PhantomData,
		}
	}

	/// Returns the token trailing trivia.
	///
	/// ```
	/// use rome_rowan::*;
	/// use rome_rowan::api::RawLanguage;
	/// let mut token = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0),|builder| {
	///     builder.token_with_trivia(
	///         SyntaxKind(1),
	///         "\n\t let \t\t",
	///         vec![TriviaPiece::Whitespace(3)],
	///         vec![TriviaPiece::Whitespace(3)],
	///     );
	/// }).first_token().unwrap();
	/// assert_eq!(" \t\t", token.trailing_trivia().text());
	/// ```
	#[inline]
	pub fn trailing_trivia(&self) -> SyntaxTrivia<L> {
		SyntaxTrivia {
			raw: self.raw.trailing_trivia(),
			_p: PhantomData,
		}
	}
}

impl<L: Language> SyntaxElement<L> {
	pub fn text_range(&self) -> TextRange {
		match self {
			NodeOrToken::Node(it) => it.text_range(),
			NodeOrToken::Token(it) => it.text_range(),
		}
	}

	pub fn text_trimmed_range(&self) -> TextRange {
		match self {
			NodeOrToken::Node(it) => it.text_trimmed_range(),
			NodeOrToken::Token(it) => it.text_trimmed_range(),
		}
	}

	pub fn leading_trivia(&self) -> Option<SyntaxTrivia<L>> {
		match self {
			NodeOrToken::Node(it) => it.first_leading_trivia(),
			NodeOrToken::Token(it) => Some(it.leading_trivia()),
		}
	}

	pub fn trailing_trivia(&self) -> Option<SyntaxTrivia<L>> {
		match self {
			NodeOrToken::Node(it) => it.last_trailing_trivia(),
			NodeOrToken::Token(it) => Some(it.trailing_trivia()),
		}
	}

	pub fn kind(&self) -> L::Kind {
		match self {
			NodeOrToken::Node(it) => it.kind(),
			NodeOrToken::Token(it) => it.kind(),
		}
	}

	pub fn parent(&self) -> Option<SyntaxNode<L>> {
		match self {
			NodeOrToken::Node(it) => it.parent(),
			NodeOrToken::Token(it) => it.parent(),
		}
	}

	pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> {
		let first = match self {
			NodeOrToken::Node(it) => Some(it.clone()),
			NodeOrToken::Token(it) => it.parent(),
		};
		iter::successors(first, SyntaxNode::parent)
	}

	pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
		match self {
			NodeOrToken::Node(it) => it.next_sibling_or_token(),
			NodeOrToken::Token(it) => it.next_sibling_or_token(),
		}
	}

	pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
		match self {
			NodeOrToken::Node(it) => it.prev_sibling_or_token(),
			NodeOrToken::Token(it) => it.prev_sibling_or_token(),
		}
	}

	pub fn detach(&self) {
		match self {
			NodeOrToken::Node(it) => it.detach(),
			NodeOrToken::Token(it) => it.detach(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct SyntaxNodeChildren<L: Language> {
	raw: cursor::SyntaxNodeChildren,
	_p: PhantomData<L>,
}

impl<L: Language> Iterator for SyntaxNodeChildren<L> {
	type Item = SyntaxNode<L>;
	fn next(&mut self) -> Option<Self::Item> {
		self.raw.next().map(SyntaxNode::from)
	}
}

#[derive(Debug, Clone)]
pub struct SyntaxElementChildren<L: Language> {
	raw: cursor::SyntaxElementChildren,
	_p: PhantomData<L>,
}

impl<L: Language> Default for SyntaxElementChildren<L> {
	fn default() -> Self {
		SyntaxElementChildren {
			raw: cursor::SyntaxElementChildren::default(),
			_p: PhantomData,
		}
	}
}

impl<L: Language> Iterator for SyntaxElementChildren<L> {
	type Item = SyntaxElement<L>;
	fn next(&mut self) -> Option<Self::Item> {
		self.raw.next().map(NodeOrToken::from)
	}
}

pub struct Preorder<L: Language> {
	raw: cursor::Preorder,
	_p: PhantomData<L>,
}

impl<L: Language> Preorder<L> {
	pub fn skip_subtree(&mut self) {
		self.raw.skip_subtree()
	}
}

impl<L: Language> Iterator for Preorder<L> {
	type Item = WalkEvent<SyntaxNode<L>>;
	fn next(&mut self) -> Option<Self::Item> {
		self.raw.next().map(|it| it.map(SyntaxNode::from))
	}
}

pub struct PreorderWithTokens<L: Language> {
	raw: cursor::PreorderWithTokens,
	_p: PhantomData<L>,
}

impl<L: Language> PreorderWithTokens<L> {
	pub fn skip_subtree(&mut self) {
		self.raw.skip_subtree()
	}
}

impl<L: Language> Iterator for PreorderWithTokens<L> {
	type Item = WalkEvent<SyntaxElement<L>>;
	fn next(&mut self) -> Option<Self::Item> {
		self.raw.next().map(|it| it.map(SyntaxElement::from))
	}
}

impl<L: Language> From<cursor::SyntaxNode> for SyntaxNode<L> {
	fn from(raw: cursor::SyntaxNode) -> SyntaxNode<L> {
		SyntaxNode {
			raw,
			_p: PhantomData,
		}
	}
}

impl<L: Language> From<SyntaxNode<L>> for cursor::SyntaxNode {
	fn from(node: SyntaxNode<L>) -> cursor::SyntaxNode {
		node.raw
	}
}

impl<L: Language> From<cursor::SyntaxToken> for SyntaxToken<L> {
	fn from(raw: cursor::SyntaxToken) -> SyntaxToken<L> {
		SyntaxToken {
			raw,
			_p: PhantomData,
		}
	}
}

impl<L: Language> From<SyntaxToken<L>> for cursor::SyntaxToken {
	fn from(token: SyntaxToken<L>) -> cursor::SyntaxToken {
		token.raw
	}
}

impl<L: Language> From<cursor::SyntaxElement> for SyntaxElement<L> {
	fn from(raw: cursor::SyntaxElement) -> SyntaxElement<L> {
		match raw {
			NodeOrToken::Node(it) => NodeOrToken::Node(it.into()),
			NodeOrToken::Token(it) => NodeOrToken::Token(it.into()),
		}
	}
}

impl<L: Language> From<SyntaxElement<L>> for cursor::SyntaxElement {
	fn from(element: SyntaxElement<L>) -> cursor::SyntaxElement {
		match element {
			NodeOrToken::Node(it) => NodeOrToken::Node(it.into()),
			NodeOrToken::Token(it) => NodeOrToken::Token(it.into()),
		}
	}
}

/// A list of `SyntaxNode`s and/or `SyntaxToken`s
#[derive(Debug, Clone, Default)]
pub struct SyntaxList<L: Language> {
	list: Option<SyntaxNode<L>>,
}

impl<L: Language> SyntaxList<L> {
	/// Creates a new list wrapping a List `SyntaxNode`
	fn new(node: SyntaxNode<L>) -> Self {
		assert_eq!(node.kind(), L::list_kind());

		Self { list: Some(node) }
	}

	/// Iterates over the elements in the list.
	pub fn iter(&self) -> SyntaxElementChildren<L> {
		if let Some(list) = &self.list {
			list.children_with_tokens()
		} else {
			SyntaxElementChildren::<L>::default()
		}
	}

	/// Returns the number of items in this list
	pub fn len(&self) -> usize {
		if let Some(list) = &self.list {
			list.raw.green().slots().len()
		} else {
			0
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn first(&self) -> Option<SyntaxElement<L>> {
		if let Some(list) = &self.list {
			list.first_child_or_token()
		} else {
			None
		}
	}

	pub fn last(&self) -> Option<SyntaxElement<L>> {
		if let Some(list) = &self.list {
			list.last_child_or_token()
		} else {
			None
		}
	}
}

impl<L: Language> IntoIterator for &SyntaxList<L> {
	type Item = SyntaxElement<L>;
	type IntoIter = SyntaxElementChildren<L>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<L: Language> IntoIterator for SyntaxList<L> {
	type Item = SyntaxElement<L>;
	type IntoIter = SyntaxElementChildren<L>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

#[cfg(test)]
mod tests {
	use text_size::TextRange;

	use crate::api::{RawLanguage, TriviaPiece};
	use crate::{Direction, Language, SyntaxKind, SyntaxList, TreeBuilder};

	#[test]
	fn empty_list() {
		let list = SyntaxList::<RawLanguage>::default();

		assert!(list.is_empty());
		assert_eq!(list.len(), 0);

		assert_eq!(list.first(), None);
		assert_eq!(list.last(), None);

		assert_eq!(list.iter().collect::<Vec<_>>(), Vec::default());
	}

	#[test]
	fn node_list() {
		let mut builder: TreeBuilder<RawLanguage> = TreeBuilder::new();

		builder.start_node(RawLanguage::list_kind());

		builder.start_node(SyntaxKind(1));
		builder.token(SyntaxKind(2), "1");
		builder.finish_node();

		builder.start_node(SyntaxKind(1));
		builder.token(SyntaxKind(2), "2");
		builder.finish_node();

		builder.finish_node();

		let node = builder.finish();
		let list = node.into_list().unwrap();

		assert!(!list.is_empty());
		assert_eq!(list.len(), 2);

		let first = list.first().and_then(|e| e.into_node()).unwrap();
		assert_eq!(first.kind(), SyntaxKind(1));
		assert_eq!(first.text(), "1");

		let last = list.last().and_then(|e| e.into_node()).unwrap();
		assert_eq!(last.kind(), SyntaxKind(1));
		assert_eq!(last.text(), "2");

		let node_texts: Vec<_> = list
			.iter()
			.map(|e| e.into_node().map(|n| n.text().to_string()))
			.collect();

		assert_eq!(
			node_texts,
			vec![Some(String::from("1")), Some(String::from("2"))]
		)
	}

	#[test]
	fn node_or_token_list() {
		let mut builder: TreeBuilder<RawLanguage> = TreeBuilder::new();

		builder.start_node(RawLanguage::list_kind());

		builder.start_node(SyntaxKind(1));
		builder.token(SyntaxKind(2), "1");
		builder.finish_node();

		builder.token(SyntaxKind(3), ",");

		builder.start_node(SyntaxKind(1));
		builder.token(SyntaxKind(2), "2");
		builder.finish_node();

		builder.finish_node();

		let node = builder.finish();
		let list = node.into_list().unwrap();

		assert!(!list.is_empty());
		assert_eq!(list.len(), 3);

		let first = list.first().and_then(|e| e.into_node()).unwrap();
		assert_eq!(first.kind(), SyntaxKind(1));
		assert_eq!(first.text(), "1");

		let last = list.last().and_then(|e| e.into_node()).unwrap();
		assert_eq!(last.kind(), SyntaxKind(1));
		assert_eq!(last.text(), "2");

		let kinds: Vec<_> = list.iter().map(|e| e.kind()).collect();

		assert_eq!(kinds, vec![SyntaxKind(1), SyntaxKind(3), SyntaxKind(1)])
	}

	#[test]
	fn siblings() {
		let mut builder: TreeBuilder<RawLanguage> = TreeBuilder::new();

		// list
		builder.start_node(SyntaxKind(1));

		// element 1
		builder.start_node(SyntaxKind(2));
		builder.token(SyntaxKind(3), "a");
		builder.finish_node();

		// element 2
		builder.start_node(SyntaxKind(2));
		builder.token(SyntaxKind(3), "b");
		builder.finish_node();

		// Missing ,
		builder.missing();

		// element 3
		builder.start_node(SyntaxKind(2));
		builder.token(SyntaxKind(3), "c");
		builder.finish_node();

		builder.finish_node();

		let root = builder.finish();

		let first = root.children().next().unwrap();
		assert_eq!(first.text().to_string(), "a");
		assert_eq!(
			first.next_sibling().map(|e| e.text().to_string()),
			Some(String::from("b"))
		);

		let second = root.children().nth(1).unwrap();
		assert_eq!(second.text().to_string(), "b");

		// Skips the missing element
		assert_eq!(
			second.next_sibling().map(|e| e.text().to_string()),
			Some(String::from("c"))
		);

		assert_eq!(
			second.prev_sibling().map(|e| e.text().to_string()),
			Some(String::from("a"))
		);

		let last = root.children().last().unwrap();
		assert_eq!(last.text(), "c");
		assert_eq!(last.next_sibling(), None);
		assert_eq!(
			last.prev_sibling().map(|e| e.text().to_string()),
			Some(String::from("b"))
		);

		assert_eq!(
			first
				.siblings(Direction::Next)
				.map(|s| s.text().to_string())
				.collect::<Vec<_>>(),
			vec!["a", "b", "c"]
		);

		assert_eq!(
			last.siblings(Direction::Prev)
				.map(|s| s.text().to_string())
				.collect::<Vec<_>>(),
			vec!["c", "b", "a"]
		);
	}

	#[test]
	fn siblings_with_tokens() {
		let mut builder: TreeBuilder<RawLanguage> = TreeBuilder::new();

		builder.start_node(RawLanguage::list_kind());

		builder.token(SyntaxKind(1), "for");
		builder.token(SyntaxKind(2), "(");
		builder.token(SyntaxKind(3), ";");

		builder.start_node(SyntaxKind(4));
		builder.token(SyntaxKind(5), "x");
		builder.finish_node();

		builder.token(SyntaxKind(3), ";");
		builder.token(SyntaxKind(6), ")");

		builder.finish_node();

		let root = builder.finish();

		let first_semicolon = root
			.children_with_tokens()
			.nth(2)
			.and_then(|e| e.into_token())
			.unwrap();

		assert_eq!(first_semicolon.text(), ";");

		assert_eq!(
			first_semicolon
				.siblings_with_tokens(Direction::Next)
				.map(|e| e.to_string())
				.collect::<Vec<_>>(),
			vec!["x", ";", ")"]
		);

		assert_eq!(
			first_semicolon.next_sibling_or_token(),
			first_semicolon.siblings_with_tokens(Direction::Next).next()
		);
		assert_eq!(
			first_semicolon.prev_sibling_or_token(),
			first_semicolon.siblings_with_tokens(Direction::Prev).next()
		);
	}

	#[test]
	pub fn syntax_text_and_len() {
		let mut builder: crate::TreeBuilder<crate::api::RawLanguage> = crate::TreeBuilder::new();
		builder.start_node(crate::SyntaxKind(0));
		builder.token_with_trivia(
			crate::SyntaxKind(0),
			"\n\t let \t\t",
			vec![TriviaPiece::Whitespace(3)],
			vec![TriviaPiece::Whitespace(3)],
		);
		builder.finish_node();

		// // Node texts

		let node = builder.finish();
		assert_eq!("\n\t let \t\t", node.text());
		assert_eq!("let", node.text_trimmed());
		assert_eq!("\n\t ", node.first_leading_trivia().unwrap().text());
		assert_eq!(" \t\t", node.last_trailing_trivia().unwrap().text());

		// Token texts

		let token = node.first_token().unwrap();
		assert_eq!("\n\t let \t\t", token.text());
		assert_eq!("let", token.text_trimmed());
		assert_eq!("\n\t ", token.leading_trivia().text());
		assert_eq!(" \t\t", token.trailing_trivia().text());
	}

	#[test]
	pub fn syntax_range() {
		let mut builder: crate::TreeBuilder<crate::api::RawLanguage> = crate::TreeBuilder::new();
		builder.start_node(crate::SyntaxKind(0));
		builder.token_with_trivia(
			crate::SyntaxKind(0),
			"\n\t let \t\t",
			vec![TriviaPiece::Whitespace(3)],
			vec![TriviaPiece::Whitespace(3)],
		);
		builder.token_with_trivia(
			crate::SyntaxKind(0),
			"a ",
			vec![TriviaPiece::Whitespace(0)],
			vec![TriviaPiece::Whitespace(1)],
		);
		builder.token_with_trivia(
			crate::SyntaxKind(1),
			"\n=\n",
			vec![TriviaPiece::Whitespace(1)],
			vec![TriviaPiece::Whitespace(1)],
		);
		builder.token(crate::SyntaxKind(0), "1");
		builder.token_with_trivia(
			crate::SyntaxKind(0),
			";\t\t",
			vec![],
			vec![TriviaPiece::Whitespace(2)],
		);
		builder.finish_node();

		let node = builder.finish();

		// Node Ranges

		assert_eq!(TextRange::new(0.into(), 18.into()), node.text_range());
		assert_eq!(
			TextRange::new(3.into(), 16.into()),
			node.text_trimmed_range()
		);
		assert_eq!(
			TextRange::new(0.into(), 3.into()),
			node.first_leading_trivia().unwrap().text_range()
		);
		assert_eq!(
			TextRange::new(16.into(), 18.into()),
			node.last_trailing_trivia().unwrap().text_range()
		);

		// as NodeOrToken

		let eq_token = node
			.descendants_with_tokens()
			.find(|x| x.kind().0 == 1)
			.unwrap();

		assert_eq!(TextRange::new(11.into(), 14.into()), eq_token.text_range());
		assert_eq!(
			TextRange::new(12.into(), 13.into()),
			eq_token.text_trimmed_range()
		);
		assert_eq!(
			TextRange::new(11.into(), 12.into()),
			eq_token.leading_trivia().unwrap().text_range()
		);
		assert_eq!(
			TextRange::new(13.into(), 14.into()),
			eq_token.trailing_trivia().unwrap().text_range()
		);

		// as Token

		let eq_token = eq_token.as_token().unwrap();
		assert_eq!(TextRange::new(11.into(), 14.into()), eq_token.text_range());
		assert_eq!(
			TextRange::new(12.into(), 13.into()),
			eq_token.text_trimmed_range()
		);
		assert_eq!(
			TextRange::new(11.into(), 12.into()),
			eq_token.leading_trivia().text_range()
		);
		assert_eq!(
			TextRange::new(13.into(), 14.into()),
			eq_token.trailing_trivia().text_range()
		);
	}

	#[test]
	pub fn syntax_trivia_pieces() {
		use crate::*;
		let node = TreeBuilder::<RawLanguage>::wrap_with_node(SyntaxKind(0), |builder| {
			builder.token_with_trivia(
				SyntaxKind(1),
				"\n\t /**/let \t\t",
				vec![TriviaPiece::Whitespace(3), TriviaPiece::Comments(4)],
				vec![TriviaPiece::Whitespace(3)],
			);
		});
		let pieces: Vec<_> = node.first_leading_trivia().unwrap().pieces().collect();
		assert_eq!(2, pieces.len());

		assert_eq!("\n\t ", pieces[0].text());
		assert_eq!(TextSize::from(3), pieces[0].text_len());
		assert_eq!(TextRange::new(0.into(), 3.into()), pieces[0].text_range());
		assert!(pieces[0].as_whitespace().is_some());

		assert_eq!("/**/", pieces[1].text());
		assert_eq!(TextSize::from(4), pieces[1].text_len());
		assert_eq!(TextRange::new(3.into(), 7.into()), pieces[1].text_range());
		assert!(pieces[1].as_comments().is_some());
	}
}
