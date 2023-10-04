use std::str::from_utf8;

use firedancer_sys::ballet::*;
use rand::{Rng, RngCore};
use solana_merkle_tree::MerkleTree;
use solana_sdk::{signature::Signature, transaction::{SanitizedTransaction, Transaction}};
use {glassbench::*, tiny_merkle_bench::*};

fn fd_merkle(b: &mut Bench) {
    let mut sigs = vec![];
    let mut statuses = vec![];
    for _ in 0..1000000 {
        sigs.push(Signature::new_unique().to_string().as_bytes().to_owned());
        statuses.push(rand::thread_rng().gen_range(0..2) as u8);
    }

    let mut msg_hashes = vec![];
    for _ in 0..1000000 {
        // let st = SanitizedTransaction::from_transaction_for_tests(Transaction::default());
        let mut msg_hash = [0u8; 32];
        let _ = rand::thread_rng().fill_bytes(&mut msg_hash);
        msg_hashes.push(msg_hash.as_ref().to_owned());
        statuses.push(rand::thread_rng().gen_range(0..2) as u8);
    }
    // save_to_file(
    //     sigs.iter()
    //         .map(|s| from_utf8(s).unwrap().to_string())
    //         .zip(statuses.clone().into_iter())
    //         .collect::<Vec<(String, u8)>>(),
    //     String::from("src/data.json"),
    // )
    // .unwrap();
    let data: Vec<(Vec<u8>, u8)> = msg_hashes.into_iter().zip(statuses.clone().into_iter()).collect();
    // let fd_data = data[0..1000].to_vec();

    b.task("Firedancer bmtree32 | 1M Leaves | Message Hash + Status", |task| {
        task.iter(|| {
            let leaves = vec![];
            let leaves = generate_leaf_nodes(data.to_vec().clone(), leaves, 1000000);

            let (_tree, _root_mem) = generate_merkle_tree(100000, leaves);
            // let root = get_root_from_tree(unsafe { &mut *root_mem });
            // println!("Firedancer's root {:?}", root.to_string());
        });
    });
    let data = data
        .iter()
        .map(|item| vec![item.0.clone(), item.1.to_be_bytes().to_vec()])
        .collect::<Vec<Vec<Vec<u8>>>>();

    // let data = data
    //     .iter()
    //     .map(|item| item.0.as_slice())
    //     .collect::<Vec<&[u8]>>();
    b.task("solana-merkle-tree | 1M Leaves | Message Hash + Status", |task| {
        task.iter(|| {
            let _tree = MerkleTree::new_custom(data.clone());
            // println!(
            //     "Solana Labs's root {:?}",
            //     tree.get_root().unwrap().to_string()
            // );
        });
    });
}

glassbench!("Benchmark merkle tree with Solana transaction message hashes", fd_merkle,);
