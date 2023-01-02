use super::{NTHelper, ProductionLogger};
use crate::{ImplementationError, Log};
use once_cell::unsync::OnceCell;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
};
mod concat;
mod suffixes;
mod union;

fn build_circular_format(id: &str, connected_set: &HashMap<&str, usize>) -> String {
    let mut first_set_trees: Vec<(&&str, &usize)> = connected_set.iter().collect();
    first_set_trees.sort_by_key(|s| s.1);
    println!("First set productions:{:?}", first_set_trees);

    let mut circular_string: String = String::new();

    let mut vec_iter = first_set_trees.iter();
    write!(circular_string, "{}", id).unwrap();
    loop {
        match vec_iter.next_back() {
            Some((prod_id, _)) => {
                write!(circular_string, "{:^6}", "<-").unwrap();
                write!(circular_string, "{}", prod_id).unwrap();
                if *prod_id == &id {
                    break circular_string;
                }
            }
            None => break circular_string,
        }
    }
}

impl NTHelper {
    fn new(identifier: &'static str) -> Self {
        Self {
            identifier,
            nullability: OnceCell::new(),
            null_hidden: OnceCell::new(),
            debugger: OnceCell::new(),
        }
    }

    fn validate_circular_dependency<'id>(
        &'id self,
        visited_set: &mut HashMap<&'id str, usize>,
    ) -> Result<(), ImplementationError> {
        let l = visited_set.len();
        if visited_set.insert(self.identifier, l).is_none() {
            Ok(())
        } else {
            Err(ImplementationError::new(
                "LeftRecursive".into(),
                build_circular_format(self.identifier, &visited_set),
            ))
        }
    }

    fn has_visited<'id>(
        &'id self,
        connected_set: &mut HashMap<&'id str, usize>,
        visited_prod: &mut HashSet<&'id str>,
    ) -> Result<bool, ImplementationError> {
        if connected_set.contains_key(self.identifier) {
            Err(ImplementationError::new(
                "LeftRecursion.".to_string(),
                build_circular_format(self.identifier, &connected_set),
            ))
        } else {
            if visited_prod.insert(self.identifier) {
                let l = connected_set.len();
                connected_set.insert(self.identifier, l);
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }
    fn assign_debugger(&self, debugger: Log<&'static str>) -> Result<(), String> {
        self.debugger
            .set(debugger)
            .map_err(|err| format!("Debugger {} is already set for this production.", err))
    }
}
impl NTHelper {
    // fn init_first<TF: FnOnce() -> HashSet<TToken>>(&self, f: TF) -> &Vec<TToken> {
    //     self.first_set.get_or_init(|| {
    //         let token_set_map = f();
    //         let mut children_set: Vec<TToken> = token_set_map.into_iter().collect();
    //         children_set.sort();
    //         children_set
    //     })
    // }
}

impl ProductionLogger for NTHelper {
    fn get_debugger(&self) -> Option<&crate::Log<&'static str>> {
        self.debugger.get()
    }
}
