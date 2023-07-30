use std::io::{Error, Write};
use std::os::raw::c_void;
use std::str::{from_utf8, FromStr};
use std::{fs, io, mem};

use firedancer_sys::ballet::*;
use rand::Rng;

use serde::{Deserialize, Serialize};

use solana_sdk::hash::Hash;
use solana_sdk::signature::Signature;
use solana_sdk::{blake3::hashv, hash::hash};

pub fn convert_to_array<T>(v: Vec<T>) -> [T; 63] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length 63 but it was {}", v.len()))
}

pub fn hash_leaf(node: &mut fd_bmtree20_node, data: &mut &[&[u8]; 2]) {
    unsafe {
        fd_bmtree20_hash_leaf(
            node,
            data as *mut _ as *mut c_void,
            mem::size_of::<&[&[u8]; 2]>().try_into().unwrap(),
        )
    };
}

pub fn generate_leaf_nodes(
    data: Vec<(Vec<u8>, u8)>,
    mut leaves: Vec<fd_bmtree20_node>,
) -> Vec<fd_bmtree20_node> {
    for i in 0..63 {
        let mut n = fd_bmtree20_node { hash: [0u8; 32] };
        let mut k = &[data[i].0.as_slice(), (&data[i].1.to_be_bytes())];
        hash_leaf(&mut n, &mut k);
        leaves.push(n)
    }
    leaves
}
pub fn get_root_from_tree(tree: fd_bmtree20_commit, leaf_cnt: usize) -> Hash {
    Hash::new_from_array(tree.node_buf[leaf_cnt - 1].hash)
}

pub fn generate_merkle_tree(
    leaf_cnt: u64,
    nodes: Vec<fd_bmtree20_node>,
) -> (fd_bmtree20_commit_t, u8) {
    let mut state = fd_bmtree20_commit_t {
        leaf_cnt,
        __bindgen_padding_0: [0u64; 3],
        node_buf: convert_to_array(nodes),
    };

    let root = unsafe { fd_bmtree20_commit_fini(&mut state) };
    (state, unsafe { *root })
}

pub fn save_to_file(data: Vec<(String, u8)>, path: String) -> Result<(), Error> {
    fs::write(
        path,
        serde_json::to_string(
            &data
                .into_iter()
                .map(|d| Receipt {
                    signature: d.0,
                    status: d.1,
                })
                .collect::<Vec<Receipt>>(),
        )
        .unwrap(),
    )?;
    Ok(())
}
pub fn read_from_file(path: String) -> Vec<(String, u8)> {
    let file = fs::File::open(path).expect("file should open read only");
    let nodes: Vec<Receipt> = serde_json::from_reader(file).unwrap();
    nodes
        .iter()
        .map(|n| (n.signature.clone(), n.status))
        .collect()
}

/// just for storing the data, not representative of the structure used in runtime
#[derive(Serialize, Deserialize)]
pub struct Receipt {
    pub signature: String,
    pub status: u8,
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn main_test() {
        let mut sigs = vec![];
        let mut statuses = vec![];
        for _ in 0..63 {
            sigs.push(Signature::new_unique().to_string().as_bytes().to_owned());
            statuses.push(rand::thread_rng().gen_range(0..2) as u8);
        }
        let data: Vec<(Vec<u8>, u8)> = sigs
            .clone()
            .into_iter()
            .map(|s| s)
            .zip(statuses.clone().into_iter())
            .collect();
        // println!("len {} {}", data[0].0.len(), data[0].1);
        let mut nodes = vec![];

        for i in 0..63 {
            let mut n = fd_bmtree20_node { hash: [0u8; 32] };
            let mut k = &[data[i].0.as_slice(), (&data[i].1.to_be_bytes())];
            unsafe {
                fd_bmtree20_hash_leaf(
                    &mut n,
                    &mut k as *mut _ as *mut c_void,
                    mem::size_of::<&[&[u8]; 2]>().try_into().unwrap(),
                )
            };

            nodes.push(n);
        }

        let mut state = fd_bmtree20_commit_t {
            leaf_cnt: 63,
            __bindgen_padding_0: [0u64; 3],
            node_buf: convert_to_array(nodes),
        };

        let _root = unsafe { fd_bmtree20_commit_fini(&mut state) };
        println!(
            "data {:?}",
            sigs.iter()
                .map(|s| from_utf8(s).unwrap().to_string())
                .zip(statuses.iter())
                .collect::<Vec<(String, &u8)>>()
        );
        println!(
            "root {:?}",
            Hash::new_from_array(state.node_buf[62].hash).to_string()
        );
        println!("root 1 {}", unsafe { *_root });
    }
}
