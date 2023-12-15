use buddy_alloc::tree::Tree;

fn main() {
    let mut storage = [0; 4];
    let mut tree = Tree::new(&mut storage, 3);

    println!("// initial state");
    println!("{}", tree.dot());

    println!("// alloc1, size 1:");
    let alloc1 = tree.allocate(1).unwrap();
    println!("// {:?}", alloc1);
    println!("{}", tree.dot());

    println!("// alloc2, size 1:");
    let alloc2 = tree.allocate(1).unwrap();
    println!("{:?}", alloc2);
    println!("{}", tree.dot());

    println!("// alloc3, size 2:");
    let alloc3 = tree.allocate(2).unwrap();
    println!("{:?}", alloc3);
    println!("{}", tree.dot());

    println!("// free alloc2");
    tree.free(alloc2.offset);
    println!("{}", tree.dot());

    println!("// free alloc3");
    tree.free(alloc3.offset);
    println!("{}", tree.dot());

    println!("// free alloc1");
    tree.free(alloc1.offset);
    println!("{}", tree.dot());
}
