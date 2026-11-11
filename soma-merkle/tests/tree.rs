use soma_merkle::{MerkleHasher, MerkleTree, Sha256Hasher};
type Tree = MerkleTree<Sha256Hasher>;

#[test]
fn build_one_leaf() {
    let data = b"hello";

    let tree = Tree::build(data, 10).unwrap();

    assert_eq!(tree.root(), Sha256Hasher::leaf(data));
    assert_eq!(tree.leaf_count(), 1);
}

#[test]
fn build_two_leaves() {
    let data = b"abcdefgh";

    let tree = Tree::build(data, 4).unwrap();

    let leaf0 = Sha256Hasher::leaf(b"abcd");
    let leaf1 = Sha256Hasher::leaf(b"efgh");

    let expected_root = Sha256Hasher::node(&leaf0, &leaf1);

    assert_eq!(tree.root(), expected_root);
    assert_eq!(tree.leaf_count(), 2);
}

#[test]
fn build_three_leaves() {
    let data = b"abcdefghijkl";

    let tree = Tree::build(data, 4).unwrap();

    let leaf0 = Sha256Hasher::leaf(b"abcd");
    let leaf1 = Sha256Hasher::leaf(b"efgh");
    let leaf2 = Sha256Hasher::leaf(b"ijkl");
    let zero_hash = [0u8; 32];

    let left = Sha256Hasher::node(&leaf0, &leaf1);
    let right = Sha256Hasher::node(&leaf2, &zero_hash);

    let expected_root = Sha256Hasher::node(&left, &right);

    assert_eq!(tree.root(), expected_root);
    assert_eq!(tree.leaf_count(), 3);
}

#[test]
fn changing_any_byte_changes_root() {
    let original = b"abcdefghijkl";
    let root = Tree::build(original, 4).unwrap().root();

    for i in 0..original.len() {
        let mut modified = original.to_vec();
        modified[i] ^= 1;
        let modified_root = Tree::build(&modified, 4).unwrap().root();
        assert_ne!(root, modified_root, "byte {i} did not change the root");
    }
}

#[test]
fn different_leaf_sizes_produce_different_roots() {
    let data = b"abcdefghijkl";

    let tree_size_2 = Tree::build(data, 2).unwrap();
    let tree_size_4 = Tree::build(data, 4).unwrap();

    assert_ne!(tree_size_2.root(), tree_size_4.root());
}

#[test]
fn empty_data_returns_none() {
    let tree = Tree::build(b"", 4);

    assert!(tree.is_none());
}

#[test]
fn invalid_input_returns_none() {
    assert!(Tree::build(b"", 4).is_none());
    assert!(Tree::build(b"abc", 0).is_none());
}
