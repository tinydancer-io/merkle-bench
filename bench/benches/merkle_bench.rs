use rand::Rng;

use {glassbench::*, tiny_merkle_bench::*};

fn fd_merkle(b: &mut Bench) {
    b.task("desc", |task| {
        task.iter(|| todo!());
    });
}

glassbench!("Benchmark", fd_merkle,);
