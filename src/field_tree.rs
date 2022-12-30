use super::FieldTree;

impl<TToken> FieldTree<TToken> {
    pub fn new() -> Self {
        Self {
            token: None,
            children: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: &[u8], token: TToken) -> Result<(), TToken> {
        if value.len() > 0 {
            match self
                .children
                .binary_search_by_key(&value[0], |child| child.0)
            {
                Ok(index) => self.children[index].1.insert(&value[1..], token),
                Err(index) => {
                    let mut field = FieldTree::new();
                    field.insert(&value[1..], token)?;
                    self.children.insert(index, (value[0], field));
                    Ok(())
                }
            }
        } else {
            match self.token.replace(token) {
                Some(t) => Err(t),
                None => Ok(()),
            }
        }
    }
}
impl<TToken: Clone> FieldTree<TToken> {
    pub fn find(&self, code_part: &[u8]) -> Option<(TToken, usize)> {
        let mut current_field = self;
        let mut index = 0;

        loop {
            if code_part.len() > index {
                match current_field
                    .children
                    .binary_search_by_key(&code_part[index], |s| s.0)
                {
                    Ok(i) => {
                        index += 1;
                        current_field = &current_field.children[i].1;
                    }
                    Err(_) => {
                        break current_field.token.as_ref().map(|t| (t.clone(), index));
                    }
                }
            } else {
                break current_field.token.as_ref().map(|t| (t.clone(), index));
            }
        }
    }
}
