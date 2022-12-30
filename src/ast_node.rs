use crate::{ASTNode, NodeImpl, StreamPtr};
use ptree::TreeItem;
use std::fmt::{Debug, Display, Formatter};

impl<TNode: Debug> Display for ASTNode<TNode> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let children_string = self.children.iter().map(|c| format!("{}", c));
        f.debug_struct("")
            .field("value", &(&self.node, &self.start, &self.end))
            .field("children", &children_string)
            .finish()
    }
}
impl<TNode: Debug> Debug for ASTNode<TNode> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("ASTNode");
        debug_struct
            .field("token", &self.node)
            .field("start", &self.start)
            .field("end", &self.end);
        if self.children.len() > 0 {
            debug_struct.field("children", &self.children);
        }
        debug_struct.finish()
    }
}

impl<TNode> ASTNode<TNode> {
    /// Create new AST node.
    pub fn new(
        node: TNode,
        start: usize,
        end: usize,
        bound: Option<(StreamPtr, StreamPtr)>,
        children: Vec<ASTNode<TNode>>,
    ) -> Self {
        Self {
            node,
            start,
            end,
            bound,
            children,
        }
    }
    /// Create AST leaf node
    pub fn leaf(
        node: TNode,
        start: usize,
        end: usize,
        bound: Option<(StreamPtr, StreamPtr)>,
    ) -> Self {
        ASTNode::new(node, start, end, bound, Vec::with_capacity(0))
    }
}
impl<TNode: NodeImpl> ASTNode<TNode> {
    /// Create AST of a null production
    pub fn null(pointer: usize, seg_index: Option<StreamPtr>) -> Self {
        ASTNode::new(
            TNode::null(),
            pointer,
            pointer,
            seg_index.map(|i| (i, i)),
            Vec::with_capacity(0),
        )
    }
}

impl<TNode: Debug + Clone> TreeItem for ASTNode<TNode> {
    type Child = Self;

    fn write_self<W: std::io::Write>(&self, f: &mut W, _: &ptree::Style) -> std::io::Result<()> {
        write!(f, "{:?} # {}-{}", self.node, self.start, self.end)
    }

    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        std::borrow::Cow::from(&self.children)
    }
}

impl<TNode: Debug + Clone> ASTNode<TNode> {
    pub fn print(&self) -> Result<(), std::io::Error> {
        ptree::print_tree(self)
    }
}
impl<TNode: Debug + Clone + Eq> ASTNode<TNode> {
    /// Find a AST child node for a given Token searching through all nested children  

    pub fn find_tree_with_node(&self, node: &TNode) -> Option<&ASTNode<TNode>> {
        if &self.node == node {
            Some(self)
        } else {
            self.children
                .iter()
                .find_map(|child| child.find_tree_with_node(node))
        }
    }

    /// Find a AST child node for a given list of Token searching through all nested children  

    pub fn find_nested_tree_with_node(&self, tokens: &[TNode]) -> Option<&ASTNode<TNode>> {
        let mut r: Option<&ASTNode<TNode>> = Some(self);
        for t in tokens {
            r = r.map(|tree| tree.find_tree_with_node(t))?;
        }

        r
    }

    /// Search through all nested children and return the first match AST child node
    pub fn find_tree<TF: Fn(&ASTNode<TNode>) -> bool>(&self, p: &TF) -> Option<&ASTNode<TNode>> {
        if p(self) {
            Some(self)
        } else {
            self.children.iter().find_map(|child| child.find_tree(p))
        }
    }

    /// Return all the match children node for a given node value
    pub fn list_tree_with_token<'this>(&'this self, node: &TNode) -> Vec<&ASTNode<TNode>> {
        let mut list_tree: Vec<&'this ASTNode<TNode>> = Vec::new();
        self.walk_tree(&mut list_tree, &|tree, list| {
            if &tree.node == node {
                list.push(tree);
            }
        });
        list_tree
    }
    pub fn list_tree<'this, TF: Fn(&ASTNode<TNode>) -> bool>(
        &'this self,
        p: &TF,
    ) -> Vec<&ASTNode<TNode>> {
        let mut list_tree: Vec<&'this ASTNode<TNode>> = Vec::new();
        self.walk_tree(&mut list_tree, &|tree, list| {
            if p(&tree) {
                list.push(tree);
            }
        });
        list_tree
    }

    pub fn get_child(&self, node: &TNode) -> Option<&ASTNode<TNode>> {
        self.children.iter().find(|child| &child.node == node)
    }
    pub fn contains(&self, node: &TNode) -> bool {
        &self.node == node || self.children.iter().any(|child| child.contains(node))
    }

    fn walk_tree<'this, TR, TF: Fn(&'this Self, &mut TR)>(&'this self, r: &mut TR, p: &TF) {
        p(self, r);
        self.children.iter().for_each(|child| child.walk_tree(r, p));
    }
}
