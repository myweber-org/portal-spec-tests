rust
use std::cmp::Ordering;

type Link<T> = Option<Box<Node<T>>>;

struct Node<T: Ord> {
    value: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T: Ord> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

pub struct BinarySearchTree<T: Ord> {
    root: Link<T>,
}

impl<T: Ord> BinarySearchTree<T> {
    pub fn new() -> Self {
        BinarySearchTree { root: None }
    }

    pub fn insert(&mut self, value: T) {
        let mut current = &mut self.root;
        while let Some(ref mut node) = current {
            match value.cmp(&node.value) {
                Ordering::Less => current = &mut node.left,
                Ordering::Greater => current = &mut node.right,
                Ordering::Equal => return,
            }
        }
        *current = Some(Box::new(Node::new(value)));
    }

    pub fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        self.inorder_traversal(&self.root, &mut result);
        result
    }

    fn inorder_traversal<'a>(&'a self, node: &'a Link<T>, result: &mut Vec<&'a T>) {
        if let Some(ref n) = node {
            self.inorder_traversal(&n.left, result);
            result.push(&n.value);
            self.inorder_traversal(&n.right, result);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bst_operations() {
        let mut bst = BinarySearchTree::new();
        assert_eq!(bst.inorder().len(), 0);

        bst.insert(5);
        bst.insert(3);
        bst.insert(7);
        bst.insert(2);
        bst.insert(4);
        bst.insert(6);
        bst.insert(8);

        let values: Vec<i32> = bst.inorder().into_iter().copied().collect();
        assert_eq!(values, vec![2, 3, 4, 5, 6, 7, 8]);
    }
}
```