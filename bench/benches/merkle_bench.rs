use std::str::from_utf8;

use rand::Rng;
use solana_sdk::signature::Signature;
use {glassbench::*, tiny_merkle_bench::*};

fn fd_merkle(b: &mut Bench) {
    let mut sigs = vec![];
    let mut statuses = vec![];
    for _ in 0..63 {
        sigs.push(Signature::new_unique().to_string().as_bytes().to_owned());
        statuses.push(rand::thread_rng().gen_range(0..2) as u8);
    }

    save_to_file(
        sigs.iter()
            .map(|s| from_utf8(s).unwrap().to_string())
            .zip(statuses.clone().into_iter())
            .collect::<Vec<(String, u8)>>(),
        String::from("src/data.json"),
    )
    .unwrap();
    let data: Vec<(Vec<u8>, u8)> = sigs.into_iter().zip(statuses.clone().into_iter()).collect();
    b.task("Generate merkle tree of 63 leaves", |task| {
        task.iter(|| {
            let leaves = vec![];
            let leaves = generate_leaf_nodes(data.clone(), leaves);
            let (tree, _root_mem) = generate_merkle_tree(63, leaves);
            let root = get_root_from_tree(tree, 63);
        });
    });
}

glassbench!("Benchmark firedancer binary merkle tree", fd_merkle,);
