use std::any;
use std::fmt;
use std::str;

pub trait Semantic: any::Any + fmt::Debug {
    const CLASS: Class<'static>;

    #[inline]
    fn class(&self) -> Class<'static> {
        Self::CLASS
    }
    fn field(&self, field: &str);
    fn field_mut(&mut self, field: &str);
    fn variant(&self, variant: &str);
    fn variant_mut(&mut self, variant: &str);
}

#[derive(Debug)]
pub struct Class<'a> {
    pub name: &'a str,
    pub id: any::TypeId,
    pub kind: Kind,
    pub role: Role,
    pub fields: &'a [Field<'a>],
}

#[derive(Debug)]
pub enum Kind {
    Unit,
    Record,
    Union,
}

#[derive(Debug)]
pub enum Role {
    Document,
    Block,
    Inline,
}

#[derive(Debug)]
pub struct Field<'a> {
    pub name: &'a str,
    pub is_children: bool,
}

#[derive(Debug)]
pub enum Cardinality {
    One,
    Many,
}

#[derive(Debug)]
pub struct Data {}

impl str::FromStr for Kind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unit" => Ok(Kind::Unit),
            "record" => Ok(Kind::Record),
            "union" => Ok(Kind::Union),
            _ => Err(()),
        }
    }
}

impl str::FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "document" => Ok(Role::Document),
            "block" => Ok(Role::Block),
            "inline" => Ok(Role::Inline),
            _ => Err(()),
        }
    }
}
