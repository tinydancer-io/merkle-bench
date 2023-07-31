use std::str::from_utf8;

use firedancer_sys::ballet::*;
use rand::Rng;
use solana_merkle_tree::MerkleTree;
use solana_sdk::signature::Signature;
use {glassbench::*, tiny_merkle_bench::*};

fn fd_merkle(b: &mut Bench) {
    let mut sigs = vec![];
    let mut statuses = vec![];
    for _ in 0..1000000 {
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
    // let fd_data = data[0..1000].to_vec();

    b.task(
        "Generate firedancer merkle tree of 1 million leaves",
        |task| {
            task.iter(|| {
                let leaves = vec![];
                let leaves = generate_leaf_nodes(data.to_vec().clone(), leaves, 1000000);

                let (tree, root_mem) = generate_merkle_tree(100000, leaves);
                let root = get_root_from_tree(unsafe { &mut *root_mem });
            });
        },
    );

    let data = data
        .iter()
        .map(|item| item.0.as_slice())
        .collect::<Vec<&[u8]>>();
    b.task("Solana merkle tree with 1 million leaves", |task| {
        task.iter(|| {
            let tree = MerkleTree::new(data.as_slice());
        });
    });
}

glassbench!("Benchmark merkle tree with Solanan Signatures", fd_merkle,);
