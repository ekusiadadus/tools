//! Definitions for the ECMAScript AST used for codegen
//! Based on the rust analyzer parser and ast definitions

use quote::format_ident;

const LANGUAGE_PREFIXES: [&str; 4] = ["js_", "ts_", "jsx_", "tsx_"];

pub struct KindsSrc<'a> {
	pub punct: &'a [(&'a str, &'a str)],
	pub keywords: &'a [&'a str],
	pub literals: &'a [&'a str],
	pub tokens: &'a [&'a str],
	pub nodes: &'a [&'a str],
}

pub const KINDS_SRC: KindsSrc = KindsSrc {
	punct: &[
		(";", "SEMICOLON"),
		(",", "COMMA"),
		("(", "L_PAREN"),
		(")", "R_PAREN"),
		("{", "L_CURLY"),
		("}", "R_CURLY"),
		("[", "L_BRACK"),
		("]", "R_BRACK"),
		("<", "L_ANGLE"),
		(">", "R_ANGLE"),
		("~", "TILDE"),
		("?", "QUESTION"),
		("??", "QUESTION2"),
		// These are *not* question AND dot tokens, they are one
		// to distinguish between `? .3134` and `?.` per ecma specs
		("?.", "QUESTIONDOT"),
		("&", "AMP"),
		("|", "PIPE"),
		("+", "PLUS"),
		("++", "PLUS2"),
		("*", "STAR"),
		("**", "STAR2"),
		("/", "SLASH"),
		("^", "CARET"),
		("%", "PERCENT"),
		(".", "DOT"),
		("...", "DOT2"),
		(":", "COLON"),
		("=", "EQ"),
		("==", "EQ2"),
		("===", "EQ3"),
		("=>", "FAT_ARROW"),
		("!", "BANG"),
		("!=", "NEQ"),
		("!==", "NEQ2"),
		("-", "MINUS"),
		("--", "MINUS2"),
		("<=", "LTEQ"),
		(">=", "GTEQ"),
		("+=", "PLUSEQ"),
		("-=", "MINUSEQ"),
		("|=", "PIPEEQ"),
		("&=", "AMPEQ"),
		("^=", "CARETEQ"),
		("/=", "SLASHEQ"),
		("*=", "STAREQ"),
		("%=", "PERCENTEQ"),
		("&&", "AMP2"),
		("||", "PIPE2"),
		("<<", "SHL"),
		(">>", "SHR"),
		(">>>", "USHR"),
		("<<=", "SHLEQ"),
		(">>=", "SHREQ"),
		(">>>=", "USHREQ"),
		("&&=", "AMP2EQ"),
		("||=", "PIPE2EQ"),
		("**=", "STAR2EQ"),
		("??=", "QUESTION2EQ"),
		("@", "AT"),
		("`", "BACKTICK"),
	],
	keywords: &[
		"await",
		"break",
		"case",
		"catch",
		"class",
		"const",
		"continue",
		"debugger",
		"default",
		"delete",
		"do",
		"else",
		"enum",
		"export",
		"extends",
		"false",
		"finally",
		"for",
		"function",
		"if",
		"in",
		"instanceof",
		"interface",
		"import",
		"implements",
		"new",
		"null",
		"package",
		"private",
		"protected",
		"public",
		"return",
		"super",
		"switch",
		"this",
		"throw",
		"try",
		"true",
		"typeof",
		"var",
		"void",
		"while",
		"with",
		"yield",
		// contextual keywords
		"readonly",
		"keyof",
		"unique",
		"declare",
		"abstract",
		"static",
		"async",
		"type",
		"from",
		"as",
		"require",
		"namespace",
		"assert",
		"module",
		"global",
		"infer",
		"get",
		"set",
		"of",
		"target",
		"never",
		"unknown",
		"any",
		"undefined",
		"let",
		"float",
		"number",
	],
	literals: &[
		"JS_NUMBER_LITERAL",
		"JS_BIG_INT_LITERAL",
		"JS_STRING_LITERAL",
		"JS_REGEX_LITERAL",
	],
	tokens: &[
		"HASH", // #
		"TEMPLATE_CHUNK",
		"DOLLARCURLY", // ${
		"ERROR_TOKEN",
		"IDENT",
		"WHITESPACE",
		"COMMENT",
		"JS_SHEBANG",
	],
	nodes: &[
		"JS_ROOT",
		"JS_DIRECTIVE",
		"ERROR",
		"JS_BLOCK_STATEMENT",
		"JS_FUNCTION_BODY",
		"JS_VARIABLE_DECLARATION_STATEMENT",
		"JS_VARIABLE_DECLARATION",
		"JS_VARIABLE_DECLARATOR",
		"JS_EQUAL_VALUE_CLAUSE",
		"JS_EMPTY_STATEMENT",
		"JS_EXPRESSION_STATEMENT",
		"JS_IF_STATEMENT",
		"JS_ELSE_CLAUSE",
		"JS_DO_WHILE_STATEMENT",
		"JS_WHILE_STATEMENT",
		"FOR_STMT",
		"FOR_IN_STMT",
		"JS_CONTINUE_STATEMENT",
		"JS_BREAK_STATEMENT",
		"JS_RETURN_STATEMENT",
		"JS_WITH_STATEMENT",
		"JS_SWITCH_STATEMENT",
		"JS_CASE_CLAUSE",
		"JS_DEFAULT_CLAUSE",
		"JS_LABELED_STATEMENT",
		"JS_THROW_STATEMENT",
		"JS_TRY_STATEMENT",
		"JS_TRY_FINALLY_STATEMENT",
		"JS_CATCH_CLAUSE",
		"JS_CATCH_DECLARATION",
		"JS_FINALLY_CLAUSE",
		"JS_DEBUGGER_STATEMENT",
		"JS_FUNCTION_DECLARATION",
		"JS_PARAMETER_LIST",
		"JS_REST_PARAMETER",
		"TS_TYPE_ANNOTATION",
		"JS_IDENTIFIER_BINDING",
		"NAME",
		"JS_REFERENCE_IDENTIFIER_EXPRESSION",
		"JS_REFERENCE_IDENTIFIER_MEMBER",
		"JS_REFERENCE_PRIVATE_MEMBER",
		"JS_THIS_EXPRESSION",
		"JS_ARRAY_EXPRESSION",
		"JS_ARRAY_HOLE",
		"JS_COMPUTED_MEMBER_NAME",
		"JS_LITERAL_MEMBER_NAME",
		"JS_OBJECT_EXPRESSION",
		"JS_PROPERTY_OBJECT_MEMBER",
		"JS_GETTER_OBJECT_MEMBER",
		"JS_SETTER_OBJECT_MEMBER",
		"JS_METHOD_OBJECT_MEMBER",
		"JS_SUPER_EXPRESSION",
		"JS_PARENTHESIZED_EXPRESSION",
		"NEW_EXPR",
		"JS_FUNCTION_EXPRESSION",
		"JS_STATIC_MEMBER_EXPRESSION",
		"JS_COMPUTED_MEMBER_EXPRESSION",
		"CALL_EXPR",
		"JS_UNARY_EXPRESSION",
		"JS_PRE_UPDATE_EXPRESSION",
		"JS_POST_UPDATE_EXPRESSION",
		"JS_BINARY_EXPRESSION",
		"JS_LOGICAL_EXPRESSION",
		"JS_CONDITIONAL_EXPRESSION",
		"ASSIGN_EXPR",
		"JS_SEQUENCE_EXPRESSION",
		"ARG_LIST",
		"JS_STRING_LITERAL_EXPRESSION",
		"JS_NUMBER_LITERAL_EXPRESSION",
		"JS_BIG_INT_LITERAL_EXPRESSION",
		"JS_BOOLEAN_LITERAL_EXPRESSION",
		"JS_NULL_LITERAL_EXPRESSION",
		"JS_REGEX_LITERAL_EXPRESSION",
		"TEMPLATE",
		"TEMPLATE_ELEMENT",
		"SPREAD_ELEMENT",
		"JS_IMPORT_CALL_EXPRESSION",
		"NEW_TARGET",
		"IMPORT_META",
		"JS_SHORTHAND_PROPERTY_OBJECT_MEMBER",
		"JS_SPREAD",
		"INITIALIZED_PROP",
		"OBJECT_PATTERN",
		"ARRAY_PATTERN",
		"ASSIGN_PATTERN",
		"REST_PATTERN",
		"KEY_VALUE_PATTERN",
		"FOR_OF_STMT",
		"SINGLE_PATTERN",
		"JS_ARROW_FUNCTION_EXPRESSION",
		"JS_YIELD_EXPRESSION",
		"JS_CLASS_DECLARATION",
		"JS_CLASS_EXPRESSION",
		"JS_EXTENDS_CLAUSE",
		"JS_PRIVATE_CLASS_MEMBER_NAME",
		"JS_CONSTRUCTOR_CLASS_MEMBER",
		"JS_CONSTRUCTOR_PARAMETER_LIST",
		"JS_CONSTRUCTOR_PARAMETER",
		"JS_PROPERTY_CLASS_MEMBER",
		"JS_METHOD_CLASS_MEMBER",
		"JS_GETTER_CLASS_MEMBER",
		"JS_SETTER_CLASS_MEMBER",
		"JS_EMPTY_CLASS_MEMBER",
		"IMPORT_DECL",
		"EXPORT_DECL",
		"EXPORT_NAMED",
		"EXPORT_DEFAULT_DECL",
		"EXPORT_DEFAULT_EXPR",
		"EXPORT_WILDCARD",
		"WILDCARD_IMPORT",
		"NAMED_IMPORTS",
		"SPECIFIER",
		"JS_AWAIT_EXPRESSION",
		// These three are just hacks for converting to ast node without
		// having to handle every error recovery case.
		// in the future we might just tag the underlying rowan nodes
		"FOR_STMT_TEST",
		"FOR_STMT_UPDATE",
		"FOR_STMT_INIT",
		"IMPORT_STRING_SPECIFIER",
		"EXPR_PATTERN",
		// TypeScript
		"TS_ANY",
		"TS_UNKNOWN",
		"TS_NUMBER",
		"TS_OBJECT",
		"TS_BOOLEAN",
		"TS_BIGINT",
		"TS_STRING",
		"TS_SYMBOL",
		"TS_VOID",
		"TS_UNDEFINED",
		"TS_NULL",
		"TS_NEVER",
		"TS_THIS",
		"TS_LITERAL",
		"TS_PREDICATE",
		"TS_TUPLE",
		"TS_TUPLE_ELEMENT",
		"TS_PAREN",
		"TS_TYPE_REF",
		"TS_QUALIFIED_PATH",
		"TS_TYPE_NAME",
		"TS_TEMPLATE",
		"TS_TEMPLATE_ELEMENT",
		"TS_MAPPED_TYPE",
		"TS_MAPPED_TYPE_PARAM",
		"TS_MAPPED_TYPE_READONLY",
		"TS_TYPE_QUERY",
		"TS_TYPE_QUERY_EXPR",
		"TS_IMPORT",
		"TS_TYPE_ARGS",
		"TS_ARRAY",
		"TS_INDEXED_ARRAY",
		"TS_TYPE_OPERATOR",
		"TS_INTERSECTION",
		"TS_UNION",
		"TS_TYPE_PARAMS",
		"TS_FN_TYPE",
		"TS_CONSTRUCTOR_TYPE",
		"TS_IMPLEMENTS_CLAUSE",
		"TS_EXTENDS",
		"TS_CONDITIONAL_TYPE",
		"TS_CONSTRAINT",
		"TS_DEFAULT",
		"TS_TYPE_PARAM",
		"TS_NON_NULL",
		"TS_ASSERTION",
		"TS_CONST_ASSERTION",
		"TS_ENUM",
		"TS_ENUM_MEMBER",
		"TS_TYPE_ALIAS_DECL",
		"TS_NAMESPACE_DECL",
		"TS_MODULE_BLOCK",
		"TS_MODULE_DECL",
		"TS_CONSTRUCTOR_PARAM",
		"TS_CALL_SIGNATURE_DECL",
		"TS_CONSTRUCT_SIGNATURE_DECL",
		"TS_INDEX_SIGNATURE",
		"TS_METHOD_SIGNATURE",
		"TS_PROPERTY_SIGNATURE",
		"TS_INTERFACE_DECL",
		"TS_ACCESSIBILITY",
		"TS_OBJECT_TYPE",
		"TS_EXPR_WITH_TYPE_ARGS",
		"TS_IMPORT_EQUALS_DECL",
		"TS_MODULE_REF",
		"TS_EXTERNAL_MODULE_REF",
		"TS_EXPORT_ASSIGNMENT",
		"TS_NAMESPACE_EXPORT_DECL",
		"TS_DECORATOR",
		"DEFAULT_CASE",
		"TS_INFER",
		"NULL",
		"UNDEFINED",
		"PROP_NAME",
		"STMT",
		"DECL",
		"PATTERN",
		"TS_ENTITY_NAME",
		"BOOLEAN",
		"BIG_INT_VALUE",
		// unknown nodes
		"JS_UNKNOWN_EXPRESSION",
		"JS_UNKNOWN_STATEMENT",
		"JS_UNKNOWN_PATTERN",
		"JS_UNKNOWN_MEMBER",
		"JS_UNKNOWN_BINDING",
		"JS_UNKNOWN_ASSIGNMENT_TARGET",
	],
};

#[derive(Default, Debug)]
pub struct AstSrc {
	pub tokens: Vec<String>,
	pub nodes: Vec<AstNodeSrc>,
	pub enums: Vec<AstEnumSrc>,
}

#[derive(Debug)]
pub struct AstNodeSrc {
	pub documentation: Vec<String>,
	pub name: String,
	// pub traits: Vec<String>,
	pub fields: Vec<Field>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
	Single(String),
	Many(Vec<String>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Field {
	Token {
		name: String,
		kind: TokenKind,
		optional: bool,
	},
	Node {
		name: String,
		ty: String,
		optional: bool,
		has_many: bool,
		separated: bool,
	},
}

#[derive(Debug, Clone)]
pub struct AstEnumSrc {
	pub documentation: Vec<String>,
	pub name: String,
	// pub traits: Vec<String>,
	pub variants: Vec<String>,
}

impl Field {
	#[allow(dead_code)]
	pub fn is_many(&self) -> bool {
		matches!(self, Field::Node { .. })
	}

	pub fn method_name(&self) -> proc_macro2::Ident {
		match self {
			Field::Token { name, .. } => {
				let name = match name.as_str() {
					";" => "semicolon",
					"'{'" => "l_curly",
					"'}'" => "r_curly",
					"'('" => "l_paren",
					"')'" => "r_paren",
					"'['" => "l_brack",
					"']'" => "r_brack",
					"'`'" => "backtick",
					"<" => "l_angle",
					">" => "r_angle",
					"=" => "eq",
					"!" => "excl",
					"*" => "star",
					"&" => "amp",
					"." => "dot",
					"..." => "dotdotdot",
					"=>" => "fat_arrow",
					":" => "colon",
					"?" => "question_mark",
					"+" => "plus",
					"-" => "minus",
					"#" => "hash",
					"@" => "at",
					"+=" => "add_assign",
					"-=" => "subtract_assign",
					"*=" => "times_assign",
					"%=" => "remainder_assign",
					"**=" => "exponent_assign",
					">>=" => "left_shift_assign",
					"<<=" => "right_shift_assign",
					">>>=" => "unsigned_right_shift_assign",
					"~" => "bitwise_not",
					"&=" => "bitwise_and_assign",
					"|=" => "bitwise_or_assign",
					"^=" => "bitwise_xor_assign",
					"&&=" => "bitwise_logical_and_assign",
					"||=" => "bitwise_logical_or_assign",
					"??=" => "bitwise_nullish_coalescing_assign",
					"++" => "increment",
					"--" => "decrement",
					"<=" => "less_than_equal",
					">=" => "greater_than_equal",
					"==" => "equality",
					"===" => "strict_equality",
					"!=" => "inequality",
					"!==" => "strict_inequality",
					"/" => "divide",
					"%" => "reminder",
					"**" => "exponent",
					"<<" => "left_shift",
					">>" => "right_shift",
					">>>" => "unsigned_right_shift",
					"|" => "bitwise_or",
					"^" => "bitwise_xor",
					"??" => "nullish_coalescing",
					"||" => "logical_or",
					"&&" => "logical_and",
					_ => name,
				};
				format_ident!("{}_token", name)
			}
			Field::Node { name, .. } => {
				let name = name;
				let (prefix, tail) = name.split_once('_').unwrap_or(("", name));
				let final_name = if LANGUAGE_PREFIXES.contains(&prefix) {
					tail
				} else {
					name.as_str()
				};

				// this check here is to avoid emitting methods called "type()",
				// where "type" is a reserved word
				if final_name == "type" {
					format_ident!("ty")
				} else {
					format_ident!("{}", final_name)
				}
			}
		}
	}
	#[allow(dead_code)]
	pub fn ty(&self) -> proc_macro2::Ident {
		match self {
			Field::Token { .. } => format_ident!("SyntaxToken"),
			Field::Node { ty, .. } => format_ident!("{}", ty),
		}
	}

	pub fn is_optional(&self) -> bool {
		match self {
			Field::Node { optional, .. } => *optional,
			Field::Token { optional, .. } => *optional,
		}
	}
}
