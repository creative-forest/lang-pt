use super::{
    Hidden, List, Lookahead, Node, Nullable, ProductionBuilder, SeparatedList, Suffixes, Validator,
};
use crate::{ASTNode, IProduction, ProductionError};
use std::rc::Rc;

impl<T: IProduction> ProductionBuilder for T {
    fn into_list(self) -> List<Self>
    where
        Self: Sized,
    {
        List::new(&Rc::new(self))
    }

    fn into_node(self, node_value: Self::Node) -> Node<Self>
    where
        Self: Sized,
    {
        Node::new(&Rc::new(self), node_value)
    }

    fn into_hidden(self) -> Hidden<Self>
    where
        Self: Sized,
    {
        Hidden::new(&Rc::new(self))
    }

    fn into_lookahead(self, node_value: Option<Self::Node>) -> Lookahead<Self>
    where
        Self: Sized,
    {
        Lookahead::new(&Rc::new(self), node_value)
    }

    fn into_separated_list<TS: IProduction<Node = Self::Node, Token = Self::Token>>(
        self,
        sep: &Rc<TS>,
        inclusive: bool,
    ) -> SeparatedList<Self, TS>
    where
        Self: Sized,
    {
        SeparatedList::new(&Rc::new(self), sep, inclusive)
    }

    fn into_suffixes<TP: IProduction<Node = Self::Node>>(
        self,
        id: &'static str,
        standalone: bool,
    ) -> Suffixes<Self>
    where
        Self: Sized,
    {
        Suffixes::init(id, &Rc::new(self), standalone)
    }

    fn into_nullable(self) -> Nullable<Self>
    where
        Self: Sized,
    {
        Nullable::new(&Rc::new(self))
    }

    fn validate_with<TF: Fn(&Vec<ASTNode<Self::Node>>, &[u8]) -> Result<(), ProductionError>>(
        self,
        validation_fn: TF,
    ) -> Validator<Self, TF>
    where
        Self: Sized,
    {
        Validator::new(&Rc::new(self), validation_fn)
    }

    fn into_null_hidden(self) -> Nullable<Self>
    where
        Self: Sized,
    {
        Nullable::hidden(&Rc::new(self))
    }
}
