use std::collections::HashMap;
#[derive(Debug)]
pub struct Trie<T> {
    root: Node<T>,
}

#[derive(Debug)]
struct Node<T> {
    children: HashMap<char, Node<T>>,
    values: Vec<T>,
}
impl<T> Trie<T> {
    pub fn new() -> Self {
        Self {
            root: Node {
                children: HashMap::new(),
                values: Vec::new(),
            },
        }
    }
    pub fn insert(&mut self, word: String, value: T) {
        let mut node = &mut self.root;
        for c in word.chars() {
            node = node.children.entry(c).or_insert(Node {
                children: HashMap::new(),
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
    pub fn search_prefix(&self, prefix: String) -> Vec<&T> {
        let mut result: Vec<&T> = Vec::new();
        let mut node = &self.root;
        for c in prefix.chars() {
            match node.children.get(&c) {
                Some(n) => node = n,
                None => return result,
            }
        }
        let mut stack = Vec::new();
        stack.push(node);
        while let Some(n) = stack.pop() {
            if !n.values.is_empty() {
                result.extend(&n.values);
            }
            for (_, child) in &n.children {
                stack.push(child);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trie() {
        let mut trie = Trie::new();
        trie.insert("hello".to_string(), 1);
        trie.insert("world".to_string(), 2);
        trie.insert("hell".to_string(), 3);
        trie.insert("hell".to_string(), 4);
        assert_eq!(trie.search("hello".to_string()), Some(&vec![1]));
        assert_eq!(trie.search("world".to_string()), Some(&vec![2]));
        assert_eq!(trie.search("hell".to_string()), Some(&vec![3, 4]));
        assert_eq!(trie.search("h".to_string()), None);
        assert_eq!(trie.search("".to_string()), None);
        assert_eq!(trie.search("helloo".to_string()), None);
        assert_eq!(trie.search_prefix("hell".to_string()), vec![&3, &4, &1]);
        assert_eq!(trie.search_prefix("h".to_string()), vec![&3, &4, &1]);
    }
}
