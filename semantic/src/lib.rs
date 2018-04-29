#![feature(const_type_id)]
extern crate type_info;

use std::any;
use std::fmt;
use std::str;

pub trait Semantic: DynamicSemantic {
    const CLASS: Class<'static>;

    fn visit_classes<F>(_visitor: &mut F)
    where
        F: FnMut(&'static Class<'static>) -> bool,
    {
    }
}

pub trait DynamicSemantic: any::Any + fmt::Debug {
    #[inline]
    fn class(&self) -> Class<'static>;
}

#[derive(Debug)]
pub struct Class<'a> {
    pub id: any::TypeId,
    pub role: Role,
    pub structure: Structure<'a>,
}

#[derive(Debug)]
pub enum Structure<'a> {
    Primitive,
    Unit {
        name: &'a str,
    },
    Enumeration {
        variants: &'a [&'a str],
    },
    Record {
        name: &'a str,
        fields: &'a [Field<'a>],
    },
    Union {
        variants: &'a [Variant<'a>],
    },
    Collection {
        item: &'a Class<'a>,
    },
}

#[derive(Debug)]
pub enum Role {
    Root,
    Document,
    Block,
    Inline,
    Attribute,
}

#[derive(Debug)]
pub struct Field<'a> {
    pub name: &'a str,
    pub ty: any::TypeId,
    pub is_children: bool,
}

#[derive(Debug)]
pub struct Variant<'a> {
    pub name: &'a str,
    pub ty: any::TypeId,
}

#[derive(Debug)]
pub struct Data {}

impl str::FromStr for Role {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "root" => Ok(Role::Root),
            "document" => Ok(Role::Document),
            "block" => Ok(Role::Block),
            "inline" => Ok(Role::Inline),
            "attribute" => Ok(Role::Attribute),
            _ => Err(()),
        }
    }
}

macro_rules! semantic_primitive {
    ($t:ty) => {
        impl Semantic for $t {
            const CLASS: Class<'static> = Class {
                id: any::TypeId::of::<$t>(),
                role: Role::Attribute,
                structure: Structure::Primitive,
            };
        }

        impl DynamicSemantic for $t {
            fn class(&self) -> Class<'static> {
                Self::CLASS
            }
        }
    };
}

semantic_primitive!(bool);
semantic_primitive!(u8);
semantic_primitive!(i8);
semantic_primitive!(u16);
semantic_primitive!(i16);
semantic_primitive!(u32);
semantic_primitive!(i32);
semantic_primitive!(f32);
semantic_primitive!(u64);
semantic_primitive!(i64);
semantic_primitive!(f64);
// Not implemented for usize, isize intentionally, since they are not cross-platform
semantic_primitive!(String);

impl<A> Semantic for Vec<A>
where
    A: Semantic,
{
    const CLASS: Class<'static> = Class {
        id: any::TypeId::of::<Vec<A>>(),
        role: A::CLASS.role,
        structure: Structure::Collection { item: &A::CLASS },
    };

    fn visit_classes<F>(visitor: &mut F)
    where
        F: FnMut(&'static Class<'static>) -> bool,
    {
        if visitor(&A::CLASS) {
            A::visit_classes(visitor);
        }
    }
}

impl<A> DynamicSemantic for Vec<A>
where
    A: Semantic,
{
    fn class(&self) -> Class<'static> {
        Self::CLASS
    }
}

impl<A> Semantic for Option<A>
where
    A: Semantic,
{
    const CLASS: Class<'static> = Class {
        id: any::TypeId::of::<Vec<A>>(),
        role: A::CLASS.role,
        structure: Structure::Collection { item: &A::CLASS },
    };

    fn visit_classes<F>(visitor: &mut F)
    where
        F: FnMut(&'static Class<'static>) -> bool,
    {
        if visitor(&A::CLASS) {
            A::visit_classes(visitor);
        }
    }
}

impl<A> DynamicSemantic for Option<A>
where
    A: Semantic,
{
    fn class(&self) -> Class<'static> {
        Self::CLASS
    }
}
