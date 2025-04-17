
        fn main() {
            let _tree: Tree<Tree<Tree<()>>> = Tree::Branch(
                Tree::Branch(Tree::Leaf, Tree::Leaf),
                Tree::Branch(Tree::Leaf, Tree::Leaf),
            );
        }

        enum Tree<T> {
            Branch(T, T),
            Leaf,
        }
        