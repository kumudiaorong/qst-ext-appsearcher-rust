use std::collections::{BTreeMap, HashSet};
#[derive(Debug)]
pub struct Trie<T> {
    root: Node<T>,
    miss_count: usize,
    ignore_case: bool,
}
impl<T> Default for Trie<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Extend<(String, T)> for Trie<T> {
    fn extend<I: IntoIterator<Item = (String, T)>>(&mut self, iter: I) {
        for (word, value) in iter {
            self.insert(word, value);
        }
    }
}
#[derive(Debug)]
struct Node<T> {
    children: BTreeMap<char, Node<T>>,
    values: Vec<T>,
}
impl<T> Trie<T>
{
    pub fn new() -> Self {
        Self {
            root: Node {
                children: BTreeMap::new(),
                values: Vec::new(),
            },
            miss_count: 3,
            ignore_case: true,
        }
    }
    pub fn insert(&mut self, mut word: String, value: T) {
        let mut node = &mut self.root;
        if self.ignore_case {
            word = word.to_lowercase();
        }
        for c in word.chars() {
            node = node.children.entry(c).or_insert(Node {
                children: BTreeMap::new(),
                values: Vec::new(),
            });
        }
        node.values.push(value);
    }
    pub fn search(&self, word: String) -> Option<&Vec<T>> {
        let mut node = &self.root;
        for c in word.chars() {
            if let Some(n) = node.children.get(&c) {
                node = n;
            } else {
                return None;
            }
        }
        (!node.values.is_empty()).then_some(&node.values)
    }
    pub fn search_prefix(&self, mut prefix: String) -> Vec<&T> {
        let mut result: Vec<&T> = Vec::new();
        if self.ignore_case {
            prefix = prefix.to_lowercase();
        }
        let mut valid_nodes = Vec::new();
        let mut dedup = HashSet::new();
        let mut stack = Vec::new();
        stack.push((&self.root, prefix.chars(), 0));
        while let Some((cur_node, mut words, miss_count)) = stack.pop() {
            let back = words.clone();
            match words.next() {
                Some(c) => {
                    for (kw, node) in cur_node.children.iter().rev() {
                        if kw == &c {
                            stack.push((node, words.clone(), 0));
                        }
                        if miss_count < self.miss_count {
                            stack.push((node, back.clone(), miss_count + 1));
                        }
                    }
                }
                None => {
                    if dedup.insert(cur_node as *const Node<T>) {
                        valid_nodes.push(cur_node);
                    }
                }
            }
        }
        dedup.clear();
        let mut over = Vec::new();
        while let Some(n) = valid_nodes.pop() {
            if !n.values.is_empty() && dedup.insert(n as *const Node<T>) {
                over.push(n);
            }
            for node in n.children.values() {
                valid_nodes.push(node);
            }
        }
        over.iter().rev().for_each(|n| result.extend(&n.values));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        trie.insert("lol".to_string(), 2);
        assert_eq!(trie.search("l".to_string()), Some(&vec![2]));
    }
}
