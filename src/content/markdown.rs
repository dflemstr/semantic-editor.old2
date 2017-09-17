#[derive(Semantic)]
pub struct Markdown {
    pub children: Vec<Block>,
    pub declarations: Vec<Declaration>,
}

#[derive(Semantic)]
pub enum Block {
    Paragraph(Paragraph),
    Blockquote(Blockquote),
    Heading(Heading),
    Code(Code),
    Yaml(Yaml),
    Html(Html),
    List(List),
    Table(Table),
    ThematicBreak(ThematicBreak),
    Image(Image),
}

#[derive(Semantic)]
pub enum Inline {
    Text(Text),
    InlineCode(InlineCode),
    Break(Break),
    Emphasis(Emphasis),
    Strong(Strong),
    Delete(Delete),
    Link(Link),
    Footnote(Footnote),
}

#[derive(Semantic)]
pub enum Declaration {
    LinkReference(LinkReference),
    ImageReference(ImageReference),
    FootnoteReference(FootnoteReference),
    Definition(Definition),
    FootnoteDefinition(FootnoteDefinition),
}

#[derive(Semantic)]
pub struct Paragraph {
    pub children: Vec<Block>,
}

#[derive(Semantic)]
pub struct Blockquote {
    pub children: Vec<Block>,
}

#[derive(Semantic)]
pub struct Heading {
    pub depth: u32,
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub struct Code {
    pub lang: String,
    // TODO(dflemstr): insert foreign code AST here
    pub value: String,
}

#[derive(Semantic)]
pub struct InlineCode {
    pub value: String,
}

#[derive(Semantic)]
pub struct Yaml {
    // TODO(dflemstr): insert YAML AST here
    pub value: String,
}

#[derive(Semantic)]
pub struct Html {
    // TODO(dflemstr): insert HTML AST here
    pub value: String,
}

#[derive(Semantic)]
pub struct List {
    pub ordered: bool,
    pub start: Option<u32>,
    pub loose: bool,
    pub children: Vec<ListItem>,
}

#[derive(Semantic)]
pub struct ListItem {
    pub loose: bool,
    pub checked: Option<bool>,
    pub children: Vec<Block>,
}

#[derive(Semantic)]
pub enum AlignType {
    Left,
    Right,
    Center,
}

#[derive(Semantic)]
pub struct Table {
    pub align: Option<AlignType>,
    pub children: Vec<TableRow>,
}

#[derive(Semantic)]
pub struct TableRow {
    pub children: Vec<TableCell>,
}

#[derive(Semantic)]
pub struct TableCell {
    pub children: Vec<Block>,
}

#[derive(Semantic)]
pub struct ThematicBreak;

#[derive(Semantic)]
pub struct Break;

#[derive(Semantic)]
pub struct Emphasis {
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub struct Strong {
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub struct Delete {
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub struct Link {
    pub title: Option<String>,
    // TODO: use symbol
    pub url: String,
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub struct Image {
    pub title: Option<String>,
    pub alt: Option<String>,
    // TODO: use symbol
    pub url: String,
}

#[derive(Semantic)]
pub struct Footnote {
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub enum ReferenceType {
    Shortcut,
    Collapsed,
    Full,
}

#[derive(Semantic)]
pub struct LinkReference {
    // TODO: use symbol
    pub identifier: String,
    pub reference_type: ReferenceType,
}

#[derive(Semantic)]
pub struct ImageReference {
    // TODO: use symbol
    pub identifier: String,
    pub reference_type: ReferenceType,
    pub alt: Option<String>,
}

#[derive(Semantic)]
pub struct FootnoteReference {
    // TODO: use symbol
    pub identifier: String,
}

#[derive(Semantic)]
pub struct Definition {
    // TODO: use symbol
    pub identifier: String,
    pub title: Option<String>,
    // TODO: use symbol
    pub url: String,
}

#[derive(Semantic)]
pub struct FootnoteDefinition {
    // TODO: use symbol
    pub identifier: String,
    pub children: Vec<Inline>,
}

#[derive(Semantic)]
pub struct Text {
    pub content: String,
}
