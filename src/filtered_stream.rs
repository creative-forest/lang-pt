use crate::{ASTNode, FltrPtr, Lex, NodeImpl, SuccessData, TokenImpl, TokenPtr, TokenStream};
use std::ops::Index;

impl<'lex, TNode> TokenStream<'lex, TNode> {
    pub fn new(original_stream: &'lex Vec<Lex<TNode>>, filtered_stream: Vec<TokenPtr>) -> Self {
        Self {
            original_stream,
            filtered_stream,
        }
    }
}
impl<'lex, TNode: TokenImpl> From<&'lex Vec<Lex<TNode>>> for TokenStream<'lex, TNode> {
    fn from(segments: &'lex Vec<Lex<TNode>>) -> Self {
        let filtered_indices: Vec<TokenPtr> = segments
            .iter()
            .enumerate()
            .filter_map(|(j, data)| {
                if data.token.is_structural() {
                    Some(TokenPtr(j))
                } else {
                    None
                }
            })
            .collect();

        Self::new(segments, filtered_indices)
    }
}
impl<'lex, TToken: TokenImpl> TokenStream<'lex, TToken> {
    pub fn is_eos(&self, index: FltrPtr) -> bool {
        self[index].token == TToken::eof()
    }
    pub fn is_eos_segment(&self, index: TokenPtr) -> bool {
        self[index].token == TToken::eof()
    }
}
impl<'lex, TToken> TokenStream<'lex, TToken> {
    pub fn create_node<TN: NodeImpl>(
        &self,
        index: FltrPtr,
        node_value: TN,
        data: SuccessData<FltrPtr, TN>,
    ) -> SuccessData<FltrPtr, TN> {
        let node = ASTNode::new(
            node_value.clone(),
            self.pointer(index),
            self.pointer(data.consumed_index),
            Some((
                self.get_token_ptr(index),
                self.get_token_ptr(data.consumed_index),
            )),
            data.children,
        );
        SuccessData::tree(data.consumed_index, node)
    }

    pub fn filtered_index_at(&self, code_pointer: usize) -> Result<FltrPtr, FltrPtr> {
        match self
            .filtered_stream
            .binary_search_by_key(&code_pointer, |s| self[*s].start)
        {
            Ok(s) => Ok(FltrPtr(s)),
            Err(s) => Err(FltrPtr(s)),
        }
    }
    pub fn find_filtered_index(&self, index: &TokenPtr) -> Result<FltrPtr, FltrPtr> {
        match self.filtered_stream.binary_search(index) {
            Ok(s) => Ok(FltrPtr(s)),
            Err(s) => Err(FltrPtr(s)),
        }
    }

    /// Get original lexical data at code pointer.
    pub fn lex_data_at(&self, code_pointer: usize) -> Result<&Lex<TToken>, &Lex<TToken>> {
        match self
            .original_stream
            .binary_search_by_key(&code_pointer, |segment| segment.start)
        {
            Ok(index) => Ok(&self.original_stream[index]),
            Err(index) => Err(&self.original_stream[index]),
        }
    }
    /// Get original lexical data ai code pointer.
    pub fn filtered_data_at(&self, code_pointer: usize) -> Result<&Lex<TToken>, &Lex<TToken>> {
        match self
            .filtered_stream
            .binary_search_by_key(&code_pointer, |s| self[*s].start)
        {
            Ok(index) => Ok(&self.original_stream[index]),
            Err(index) => Err(&self.original_stream[index]),
        }
    }
    pub fn pointer(&self, filtered_index: FltrPtr) -> usize {
        self[filtered_index].start
    }
    pub fn get(&self, index: FltrPtr) -> Option<&Lex<TToken>> {
        self.filtered_stream.get(index.0).map(|s| &self[*s])
    }

    pub fn length(&self) -> usize {
        self.filtered_stream.len()
    }

    pub fn has_stream(&self, index: FltrPtr) -> bool {
        self.filtered_stream.len() < index.0
    }
    pub fn eos_pointer(&self) -> usize {
        self.original_stream[self.original_stream.len() - 1].end
    }

    pub fn iter_lex(&'lex self) -> impl Iterator<Item = &'lex Lex<TToken>> {
        self.filtered_stream.iter().map(|s| &self[*s])
    }
    pub fn iter_lex_at(&self, index: FltrPtr) -> impl Iterator<Item = &Lex<TToken>> {
        self.filtered_stream[index.0..].iter().map(|s| &self[*s])
    }
    pub fn iter_at(&self, index: usize) -> impl Iterator<Item = &Lex<TToken>> {
        self.original_stream[index..].iter()
    }
    pub fn iter_range(&self, start: usize, end: usize) -> impl Iterator<Item = &Lex<TToken>> {
        self.original_stream[start..end].iter()
    }
    pub fn iter_lex_range(
        &self,
        start: FltrPtr,
        end: FltrPtr,
    ) -> impl Iterator<Item = &Lex<TToken>> {
        self.filtered_stream[start.0..end.0]
            .iter()
            .map(|s| &self[*s])
    }

    pub fn get_segments(&self) -> &Vec<Lex<TToken>> {
        &self.original_stream
    }

    pub fn token_pair<'code>(&self, code: &'code str) -> Vec<(&TToken, &'code str)> {
        self.original_stream
            .iter()
            .map(|d| (&d.token, &code[d.start..d.end]))
            .collect()
    }

    pub fn get_token_ptr(&self, index: FltrPtr) -> TokenPtr {
        self.filtered_stream[index.0]
    }
    pub fn find_filter_ptr(&self, index: TokenPtr) -> Result<FltrPtr, FltrPtr> {
        match self.filtered_stream.binary_search(&index) {
            Ok(i) => Ok(FltrPtr(i)),
            Err(i) => Ok(FltrPtr(i)),
        }
    }
    pub fn last_segment_index(&self, lex_index: &FltrPtr) -> Option<TokenPtr> {
        if lex_index.0 > 0 {
            Some(self.filtered_stream[lex_index.0 - 1])
        } else {
            None
        }
    }
}

impl<'lex, TNode> Index<FltrPtr> for TokenStream<'lex, TNode> {
    type Output = Lex<TNode>;

    fn index(&self, index: FltrPtr) -> &Self::Output {
        debug_assert!(
            index.0 < self.filtered_stream.len(),
            "Trying to access index '{:?}' from filtered stream length '{}'",
            index,
            self.filtered_stream.len()
        );
        &self.original_stream[self.filtered_stream[index.0].0]
    }
}

impl<'lex, TNode> Index<TokenPtr> for TokenStream<'lex, TNode> {
    type Output = Lex<TNode>;

    fn index(&self, index: TokenPtr) -> &Self::Output {
        debug_assert!(
            index.0 < self.original_stream.len(),
            "Trying to access index '{:?}' from lex stream length '{}'",
            index,
            self.filtered_stream.len()
        );
        &self.original_stream[index.0]
    }
}
