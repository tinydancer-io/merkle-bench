use std::io::{Error, Write};
use std::os::raw::c_void;
use std::slice::from_raw_parts;
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

pub fn hash_leaf(node: &mut fd_bmtree32_node, data: &mut &[&[u8]; 2]) {
    // &mut &[&[u8]; 2])
    unsafe {
        fd_bmtree32_hash_leaf(
            node,
            data as *mut _ as *mut c_void,
            mem::size_of::<&[&[u8]; 2]>().try_into().unwrap(),
        )
    };
}

pub fn generate_leaf_nodes(
    data: Vec<(Vec<u8>, u8)>,
    mut leaves: Vec<fd_bmtree32_node>,
    leaf_cnt: u64,
) -> Vec<fd_bmtree32_node> {
    for i in 0..(leaf_cnt as usize) {
        let mut n = fd_bmtree32_node { hash: [0u8; 32] };
        let mut k = &[data[i].0.as_slice(), &data[i].1.to_be_bytes()];
        // let mut k = data[i].0.as_slice();
        hash_leaf(&mut n, &mut k);
        leaves.push(n)
    }
    leaves
}
pub fn get_root_from_tree(byte: &mut u8) -> Hash {
    let root = unsafe { from_raw_parts(byte, 32) };

    Hash::new_from_array(*slice_to_array_32(root).unwrap())
}

pub fn generate_merkle_tree(
    leaf_cnt: u64,
    leaves: Vec<fd_bmtree32_node>,
) -> (fd_bmtree32_commit_t, *mut u8) {
    let mut nodes = vec![];
    for _ in 0..63 {
        let n = fd_bmtree32_node { hash: [0u8; 32] };
        nodes.push(n);
    }
    let mut state = fd_bmtree32_commit_t {
        leaf_cnt,
        __bindgen_padding_0: [0u64; 3],
        node_buf: convert_to_array(nodes),
    };

    for i in 0..leaf_cnt {
        // using loop because its faster
        unsafe {
            fd_bmtree32_commit_append(&mut state, &leaves[i as usize], 1);
        }
    }
    let root = unsafe { fd_bmtree32_commit_fini(&mut state) };
    (state, root)
}

pub fn save_to_file(data: Vec<(Hash, u8)>, path: String) -> Result<(), Error> {
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
pub fn read_from_file(path: String) -> Vec<(Hash, u8)> {
    let file = fs::File::open(path).expect("file should open read only");
    let nodes: Vec<Receipt> = serde_json::from_reader(file).unwrap();
    nodes
        .iter()
        .map(|n| (n.signature.clone(), n.status))
        .collect()
}

#[derive(Debug)]
struct TryFromSliceError(());

fn slice_to_array_32<T>(slice: &[T]) -> Result<&[T; 32], TryFromSliceError> {
    if slice.len() <= 32 {
        let ptr = slice.as_ptr() as *const [T; 32];
        unsafe { Ok(&*ptr) }
    } else {
        Err(TryFromSliceError(()))
    }
}

/// just for storing the data, not representative of the structure used in runtime
#[derive(Serialize, Deserialize)]
pub struct Receipt {
    pub signature: Hash,
    pub status: u8,
}

#[cfg(test)]
mod tests {
    use std::slice::from_raw_parts;

    use super::*;
    use firedancer_sys::ballet::*;
    use rand::RngCore;
    use solana_merkle_tree::MerkleTree;
    #[test]
    fn main_test() {
        let mut sigs = vec![];
        let mut statuses = vec![];
        for _ in 0..100 {
            sigs.push(Signature::new_unique().to_string().as_bytes().to_owned());
            statuses.push(rand::thread_rng().gen_range(0..2) as u8);
        }

        let mut msg_hashes = vec![];
        for _ in 0..100 {
            let mut msg_hash = [0u8; 32];
            let _ = rand::thread_rng().fill_bytes(&mut msg_hash);
            msg_hashes.push(msg_hash.as_ref().to_owned());
            statuses.push(rand::thread_rng().gen_range(0..2) as u8);
        }

        save_to_file(
            msg_hashes.iter()
                .map(|s| Hash::new(&s))
                .zip(statuses.clone().into_iter())
                .collect::<Vec<(Hash, u8)>>(),
            String::from("src/data.json"),
        )
        .unwrap();

        let data: Vec<(Vec<u8>, u8)> = msg_hashes
            .clone()
            .into_iter()
            .map(|s| s)
            .zip(statuses.clone().into_iter())
            .collect();
        // println!("len {} {}", data[0].0.len(), data[0].1);
        let mut nodes = vec![];
        let mut leaves = vec![];
        for _ in 0..63 {
            let n = fd_bmtree32_node { hash: [0u8; 32] };
            nodes.push(n);
        }

        for i in 0..100 {
            let mut n = fd_bmtree32_node { hash: [0u8; 32] };

            let mut k = data[i].0.as_slice();
            unsafe {
                fd_bmtree32_hash_leaf(
                    &mut n,
                    &mut k as *mut _ as *mut c_void,
                    mem::size_of::<&[u8]>().try_into().unwrap(),
                )
            };

            leaves.push(n);
        }
        let node_buf = convert_to_array(nodes);

        let mut state = fd_bmtree32_commit_t {
            leaf_cnt: 100,
            __bindgen_padding_0: [0u64; 3],
            node_buf,
        };
        for i in 0..100 {
            unsafe {
                fd_bmtree32_commit_append(&mut state, &leaves[i], (i + 1).try_into().unwrap());
            }
        }
        // let mut k = &[data[i].0.as_slice(), (&data[i].1.to_be_bytes())];
        // mem::size_of::<&[&[u8]; 2]>().try_into().unwrap()
        // println!(
        //     "nodebuf {:?} {:?}",
        //     Hash::new_from_array(node_buf[62].hash).to_string(),
        //     node_buf[62].hash
        // );
        let _root = unsafe { fd_bmtree32_commit_fini(&mut state) };
        // println!(
        //     "check {:?}",
        //     sigs.iter()
        //         .map(|s| from_utf8(s).unwrap().to_string())
        //         .zip(statuses.iter())
        //         .collect::<Vec<(String, &u8)>>()[0]
        // );
        // println!(
        //     "root {:?}",
        //     leaves
        //         .iter()
        //         .map(|l| Hash::new_from_array(l.hash).to_string())
        //         .collect::<Vec<String>>()
        // );
        let root = unsafe { from_raw_parts(_root, 32) };
        // let mut y = [0; 32];
        // {
        //     let y: &mut [u8; 32] = &mut y;
        //     y.copy_from_slice(&root[..32]);
        // }
        // println!("root 1 {:?}", Hash::new_from_array(y).to_string());
        println!(
            "firedancer bmtree32 root {:?}",
            Hash::new_from_array(*slice_to_array_32(root).unwrap()).to_string()
        );
        let mut data = read_from_file("src/data.json".to_owned());
        // println!("check 2 {:?}", data[0]);
        let mut data = data
            .iter_mut()
            .map(|item| item.0.as_ref())
            .collect::<Vec<&[u8]>>();

        // let tree = MerkleTree::new(data.as_slice());
        // println!("solana merkle root {:?}", tree.get_root().unwrap());
    }
}
