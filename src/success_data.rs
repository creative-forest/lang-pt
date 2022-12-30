use crate::{ASTNode, SuccessData};

impl<I, TNode> SuccessData<I, TNode> {
    pub fn new(consumed_index: I, children: Vec<ASTNode<TNode>>) -> Self {
        Self {
            consumed_index,
            children,
        }
    }
    pub fn hidden(consumed_index: I) -> Self {
        Self {
            consumed_index,
            children: Vec::with_capacity(0),
        }
    }
    pub fn tree(consumed_index: I, tree: ASTNode<TNode>) -> Self {
        Self {
            consumed_index,
            children: vec![tree],
        }
    }

    pub fn range(&self) -> Option<(usize, usize)> {
        if self.children.len() > 0 {
            Some((
                self.children[0].start,
                self.children[self.children.len() - 1].end,
            ))
        } else {
            None
        }
    }
}
