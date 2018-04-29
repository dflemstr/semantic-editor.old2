//! The Markdown format, using the [CommonMark](http://commonmark.org/) standard.
//!
//! Mostly taken from <https://github.com/syntax-tree/mdast>.
use error;
use pulldown_cmark;
use std::io;

/// Houses all nodes.
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "document")]
pub struct Markdown {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Block>,
    /// Document declarations.
    pub declarations: Vec<Declaration>,
}

/// A union of all possible block elements.
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub enum Block {
    /// The `Paragraph` variant.
    Paragraph(Paragraph),
    /// The `Blockquote` variant.
    Blockquote(Blockquote),
    /// The `Heading` variant.
    Heading(Heading),
    /// The `Code` variant.
    Code(Code),
    /// The `Yaml` variant.
    Yaml(Yaml),
    /// The `Html` variant.
    Html(Html),
    /// The `List` variant.
    List(List),
    /// The `Table` variant.
    Table(Table),
    /// The `ThematicBreak` variant.
    ThematicBreak(ThematicBreak),
    /// The `Image` variant.
    Image(Image),
}

/// A union of all possible inline elements.
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub enum Inline {
    /// The `Text` variant.
    Text(Text),
    /// The `InlineCode` variant.
    InlineCode(InlineCode),
    /// The `Break` variant.
    Break(Break),
    /// The `Emphasis` variant.
    Emphasis(Emphasis),
    /// The `Strong` variant.
    Strong(Strong),
    /// The `Delete` variant.
    Delete(Delete),
    /// The `Link` variant.
    Link(Link),
    /// The `Footnote` variant.
    Footnote(Footnote),
}

/// A union of all possible declaration elements.
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub enum Declaration {
    /// The `LinkReference` variant.
    LinkReference(LinkReference),
    /// The `ImageReference` variant.
    ImageReference(ImageReference),
    /// The `FootnoteReference` variant.
    FootnoteReference(FootnoteReference),
    /// The `Definition` variant.
    Definition(Definition),
    /// The `FootnoteDefinition` variant.
    FootnoteDefinition(FootnoteDefinition),
}

/// Represents a unit of discourse dealing
/// with a particular point or idea.
///
/// ```idl
/// interface Paragraph <: Parent {
///   type: "paragraph";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// Alpha bravo charlie.
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "paragraph",
///   "children": [{
///     "type": "text",
///     "value": "Alpha bravo charlie."
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct Paragraph {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Block>,
}

/// Represents a quote.
///
/// ```idl
/// interface Blockquote <: Parent {
///   type: "blockquote";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// > Alpha bravo charlie.
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "blockquote",
///   "children": [{
///     "type": "paragraph",
///     "children": [{
///       "type": "text",
///       "value": "Alpha bravo charlie."
///     }]
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct Blockquote {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Block>,
}

/// `Heading` ([`Parent`](./struct.Parent.html)), just like with HTML, with a level greater
/// than or equal to 1, lower than or equal to 6.
///
/// ```idl
/// interface Heading <: Parent {
///   type: "heading";
///   depth: 1 <= uint32 <= 6;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// # Alpha
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "heading",
///   "depth": 1,
///   "children": [{
///     "type": "text",
///     "value": "Alpha"
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct Heading {
    /// The nesting depth of the heading (1-6).
    pub depth: u32,
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// Occurs at block level (see
/// [`InlineCode`](./struct.InlineCode.html) for code spans).  `Code` sports a language
/// tag (when using GitHub Flavoured Markdown fences with a flag, `null`
/// otherwise).
///
/// ```idl
/// interface Code <: Text {
///   type: "code";
///   lang: string | null;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
///     foo()
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "code",
///   "lang": null,
///   "value": "foo()"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Code {
    /// The language that the code is written in.
    pub lang: String,
    // TODO(dflemstr): insert foreign code AST here
    /// The code contents.
    pub value: String,
}

/// Occurs inline (see [`Code`](./struct.Code.html) for
/// blocks). Inline code does not sport a `lang` attribute.
///
/// ```idl
/// interface InlineCode <: Text {
///   type: "inlineCode";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// `foo()`
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "inlineCode",
///   "value": "foo()"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct InlineCode {
    /// The code contents.
    pub value: String,
}

/// Can occur at the start of a document, and
/// contains embedded YAML data.
///
/// ```idl
/// interface YAML <: Text {
///   type: "yaml";
/// }
/// ```
///
/// > **Note**: YAML used to be available through the core of remark and thus
/// > is specified here.  Support for it now moved to
/// > [`remark-frontmatter`][frontmatter], and the definition here may be removed
/// > in the future.
///
/// For example, the following markdown:
///
/// ```md
/// ---
/// foo: bar
/// ---
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "yaml",
///   "value": "foo: bar"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct Yaml {
    // TODO(dflemstr): insert YAML AST here
    /// The YAML contents.
    pub value: String,
}

/// Contains embedded HTML.
///
/// ```idl
/// interface HTML <: Text {
///   type: "html";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// <div>
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "html",
///   "value": "<div>"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Html {
    // TODO(dflemstr): insert HTML AST here
    /// The HTML contents.
    pub value: String,
}

/// Contains [`ListItem`s](./struct.ListItem.html).  No other nodes
/// may occur in lists.
///
/// The `start` property contains the starting number of the list when
/// `ordered: true`; `null` otherwise.
///
/// When all list items have `loose: false`, the list’s `loose` property is also
/// `false`.  Otherwise, `loose: true`.
///
/// ```idl
/// interface List <: Parent {
///   type: "list";
///   ordered: true | false;
///   start: uint32 | null;
///   loose: true | false;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// 1. [x] foo
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "list",
///   "ordered": true,
///   "start": 1,
///   "loose": false,
///   "children": [{
///     "type": "listItem",
///     "loose": false,
///     "checked": true,
///     "children": [{
///       "type": "paragraph",
///       "children": [{
///         "type": "text",
///         "value": "foo",
///       }]
///     }]
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct List {
    /// Whether the list is ordered (with numbers) or not.
    pub ordered: bool,
    /// The start number for an ordered list, or `None` if numbers should be auto-assigned.
    pub start: Option<u32>,
    /// Whether any of the children are `loose`.
    pub loose: bool,
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<ListItem>,
}

/// Is a child of a [`List`](./struct.List.html).
///
/// Loose `ListItem`s often contain more than one block-level elements.
///
/// A checked property exists on `ListItem`s, set to `true` (when checked),
/// `false` (when unchecked), or `null` (when not containing a checkbox).
///
/// ```idl
/// interface ListItem <: Parent {
///   type: "listItem";
///   loose: true | false;
///   checked: true | false | null;
/// }
/// ```
///
/// For an example, see the definition of [`List`](./struct.List.html).
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct ListItem {
    /// Whether this item can contain more than one block element.
    pub loose: bool,
    /// Whether this item is checked, or `None` if it does not contain a checkbox.
    pub checked: Option<bool>,
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Block>,
}

/// The align type for a `Table`.
#[derive(Clone, Copy, Debug, Semantic, TypeInfo)]
#[semantic(role = "attribute")]
pub enum AlignType {
    /// Align to the left.
    Left,
    /// Align to the right.
    Right,
    /// Align to the center.
    Center,
}

/// Represents tabular data, with alignment.
/// Its children are [`TableRow`](./struct.TableRow.html)s, the first of which acts as
/// a table header row.
///
/// `table.align` represents the alignment of columns.
///
/// ```idl
/// interface Table <: Parent {
///   type: "table";
///   align: [alignType];
/// }
/// ```
///
/// ```idl
/// enum alignType {
///   "left" | "right" | "center" | null;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// | foo | bar |
/// | :-- | :-: |
/// | baz | qux |
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "table",
///   "align": ["left", "center"],
///   "children": [
///     {
///       "type": "tableRow",
///       "children": [
///         {
///           "type": "tableCell",
///           "children": [{
///             "type": "text",
///             "value": "foo"
///           }]
///         },
///         {
///           "type": "tableCell",
///           "children": [{
///             "type": "text",
///             "value": "bar"
///           }]
///         }
///       ]
///     },
///     {
///       "type": "tableRow",
///       "children": [
///         {
///           "type": "tableCell",
///           "children": [{
///             "type": "text",
///             "value": "baz"
///           }]
///         },
///         {
///           "type": "tableCell",
///           "children": [{
///             "type": "text",
///             "value": "qux"
///           }]
///         }
///       ]
///     }
///   ]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct Table {
    /// The alignment of the table columns.
    pub align: Vec<AlignType>,
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<TableRow>,
}

/// `TableRow` ([`Parent`](./struct.Parent.html)).  Its children are always
/// [`TableCell`](./struct.TableCell.html).
///
/// ```idl
/// interface TableRow <: Parent {
///   type: "tableRow";
/// }
/// ```
///
/// For an example, see the definition of `Table`.
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct TableRow {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<TableCell>,
}

/// `TableCell` ([`Parent`](./struct.Parent.html)).  Contains a single tabular field.
///
/// ```idl
/// interface TableCell <: Parent {
///   type: "tableCell";
/// }
/// ```
///
/// For an example, see the definition of [`Table`](./struct.Table.html).
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct TableCell {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Block>,
}

/// A Represents a break in content,
/// often shown as a horizontal rule, or by two HTML section elements.
///
/// ```idl
/// interface ThematicBreak <: Node {
///   type: "thematicBreak";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// ***
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "thematicBreak"
/// }
/// ```
#[derive(Clone, Copy, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct ThematicBreak;

/// Represents an explicit line break.
///
/// ```idl
/// interface Break <: Node {
///   type: "break";
/// }
/// ```
///
/// For example, the following markdown (interpuncts represent spaces):
///
/// ```md
/// foo··
/// bar
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "paragraph",
///   "children": [
///     {
///       "type": "text",
///       "value": "foo"
///     },
///     {
///       "type": "break"
///     },
///     {
///       "type": "text",
///       "value": "bar"
///     }
///   ]
/// }
/// ```
#[derive(Clone, Copy, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Break;

/// Represents slight emphasis.
///
/// ```idl
/// interface Emphasis <: Parent {
///   type: "emphasis";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// *alpha* _bravo_
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "paragraph",
///   "children": [
///     {
///       "type": "emphasis",
///       "children": [{
///         "type": "text",
///         "value": "alpha"
///       }]
///     },
///     {
///       "type": "text",
///       "value": " "
///     },
///     {
///       "type": "emphasis",
///       "children": [{
///         "type": "text",
///         "value": "bravo"
///       }]
///     }
///   ]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Emphasis {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// Represents strong emphasis.
///
/// ```idl
/// interface Strong <: Parent {
///   type: "strong";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// **alpha** __bravo__
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "paragraph",
///   "children": [
///     {
///       "type": "strong",
///       "children": [{
///         "type": "text",
///         "value": "alpha"
///       }]
///     },
///     {
///       "type": "text",
///       "value": " "
///     },
///     {
///       "type": "strong",
///       "children": [{
///         "type": "text",
///         "value": "bravo"
///       }]
///     }
///   ]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Strong {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// Represents text ready for removal.
///
/// ```idl
/// interface Delete <: Parent {
///   type: "delete";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// ~~alpha~~
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "delete",
///   "children": [{
///     "type": "text",
///     "value": "alpha"
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Delete {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// Represents the humble hyperlink.
///
/// ```idl
/// interface Link <: Parent {
///   type: "link";
///   title: string | null;
///   url: string;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// [alpha](http://example.com "bravo")
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "link",
///   "title": "bravo",
///   "url": "http://example.com",
///   "children": [{
///     "type": "text",
///     "value": "alpha"
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Link {
    /// The title of the link.
    pub title: Option<String>,
    // TODO: use symbol
    /// The URL of the link.
    pub url: String,
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// Represents the figurative figure.
///
/// ```idl
/// interface Image <: Node {
///   type: "image";
///   title: string | null;
///   alt: string | null;
///   url: string;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// ![alpha](http://example.com/favicon.ico "bravo")
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "image",
///   "title": "bravo",
///   "url": "http://example.com",
///   "alt": "alpha"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Image {
    /// The title of the image.
    pub title: Option<String>,
    /// The alternative title of the image.
    pub alt: Option<String>,
    // TODO: use symbol
    /// The URL of the image.
    pub url: String,
}

/// Represents an inline marker, whose
/// content relates to the document but is outside its flow.
///
/// ```idl
/// interface Footnote <: Parent {
///   type: "footnote";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// [^alpha bravo]
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "footnote",
///   "children": [{
///     "type": "text",
///     "value": "alpha bravo"
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Footnote {
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// The reference type for a `LinkReference`.
#[derive(Clone, Copy, Debug, Semantic, TypeInfo)]
#[semantic(role = "attribute")]
pub enum ReferenceType {
    /// The reference is implicit, like `[foo]`.
    Shortcut,
    /// The reference is collapsed, like `[foo][]`.
    Collapsed,
    /// The reference is full, like `[Hello][foo]`.
    Full,
}

/// Represents a humble hyperlink,
/// its `url` and `title` defined somewhere else in the document by a
/// [`Definition`](./struct.Definition.html).
///
/// `referenceType` is needed to detect if a reference was meant as a
/// reference (`[foo][]`) or just unescaped brackets (`[foo]`).
///
/// ```idl
/// interface LinkReference <: Parent {
///   type: "linkReference";
///   identifier: string;
///   referenceType: referenceType;
/// }
/// ```
///
/// ```idl
/// enum referenceType {
///   "shortcut" | "collapsed" | "full";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// [alpha][bravo]
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "linkReference",
///   "identifier": "bravo",
///   "referenceType": "full",
///   "children": [{
///     "type": "text",
///     "value": "alpha"
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct LinkReference {
    // TODO: use symbol
    /// The identifier/label that is the reference target.
    pub identifier: String,
    /// The type of link reference this is.
    pub reference_type: ReferenceType,
}

/// Represents a figurative figure,
/// its `url` and `title` defined somewhere else in the document by a
/// [`Definition`](./struct.Definition.html).
///
/// `referenceType` is needed to detect if a reference was meant as a
/// reference (`![foo][]`) or just unescaped brackets (`![foo]`).
/// See [`LinkReference`](./struct.LinkReference.html) for the definition of `referenceType`.
///
/// ```idl
/// interface ImageReference <: Node {
///   type: "imageReference";
///   identifier: string;
///   referenceType: referenceType;
///   alt: string | null;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// ![alpha][bravo]
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "imageReference",
///   "identifier": "bravo",
///   "referenceType": "full",
///   "alt": "alpha"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct ImageReference {
    // TODO: use symbol
    /// The identifier/label that is the reference target.
    pub identifier: String,
    /// The type of link reference this is.
    pub reference_type: ReferenceType,
    /// An inline alternate title.
    pub alt: Option<String>,
}

/// Is like [`Footnote`](./struct.Footnote.html),
/// but its content is already outside the documents flow: placed in a
/// [`FootnoteDefinition`](./struct.FootnoteDefinition.html).
///
/// ```idl
/// interface FootnoteReference <: Node {
///   type: "footnoteReference";
///   identifier: string;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// [^alpha]
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "footnoteReference",
///   "identifier": "alpha"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct FootnoteReference {
    // TODO: use symbol
    /// The identifier/label that is the reference target.
    pub identifier: String,
}

/// Represents the definition (i.e., location
/// and title) of a [`LinkReference`](./struct.LinkReference.html) or an
/// [`ImageReference`](./struct.ImageReference.html).
///
/// ```idl
/// interface Definition <: Node {
///   type: "definition";
///   identifier: string;
///   title: string | null;
///   url: string;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// [alpha]: http://example.com
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "definition",
///   "identifier": "alpha",
///   "title": null,
///   "url": "http://example.com"
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct Definition {
    // TODO: use symbol
    /// The identifier/label of the definition.
    pub identifier: String,
    /// The title of the definition.
    pub title: Option<String>,
    // TODO: use symbol
    /// The URL of the definition.
    pub url: String,
}

/// Represents the definition
/// (i.e., content) of a [`FootnoteReference`](./struct.FootnoteReference.html).
///
/// ```idl
/// interface FootnoteDefinition <: Parent {
///   type: "footnoteDefinition";
///   identifier: string;
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// [^alpha]: bravo and charlie.
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "footnoteDefinition",
///   "identifier": "alpha",
///   "children": [{
///     "type": "paragraph",
///     "children": [{
///       "type": "text",
///       "value": "bravo and charlie."
///     }]
///   }]
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "block")]
pub struct FootnoteDefinition {
    // TODO: use symbol
    /// The identifier/label that is the reference target.
    pub identifier: String,
    /// Child elements.
    #[semantic(children)]
    pub children: Vec<Inline>,
}

/// Represents everything that is just text.
/// Note that its `type` property is `text`, but it is different from
/// [`Text`](./struct.Text.html).
///
/// ```idl
/// interface TextNode <: Text {
///   type: "text";
/// }
/// ```
///
/// For example, the following markdown:
///
/// ```md
/// Alpha bravo charlie.
/// ```
///
/// Yields:
///
/// ```json
/// {
///   "type": "text",
///   "value": "Alpha bravo charlie."
/// }
/// ```
#[derive(Clone, Debug, Semantic, TypeInfo)]
#[semantic(role = "inline")]
pub struct Text {
    /// The text content.
    pub content: String,
}

impl Markdown {
    fn read<R: io::Read>(mut read: R) -> error::Result<Self> {
        let mut text = String::new();
        read.read_to_string(&mut text)?;
        let _parser = pulldown_cmark::Parser::new(&text);
        unimplemented!()
    }
}
