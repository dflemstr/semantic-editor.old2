use std::str;

pub trait Semantic {
    fn type_() -> &'static str;

    fn kind() -> Kind;
}

pub trait Field {
    fn name() -> &'static str;
    fn cardinality() -> Cardinality;
}

pub enum Data {}

#[derive(Debug)]
pub enum Kind {
    Document,
    Block,
    Inline,
    Union,
}

#[derive(Debug)]
pub enum Cardinality {

}

impl str::FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "document" => Ok(Kind::Document),
            "block" => Ok(Kind::Block),
            "inline" => Ok(Kind::Inline),
            "union" => Ok(Kind::Union),
            _ => Err(()),
        }
    }
}
