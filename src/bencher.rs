extern crate rand;
extern crate test;

use rand::Rng;
use test::Bencher;

fn blake3_double2() {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"123123");
    hasher.update(b"qweqwe");
    let hash1 = hasher.finalize();

    let mut hasher = blake3::Hasher::new();
    hasher.update(hash1.as_bytes());
    hasher.finalize().as_bytes().to_vec();
}

fn blake3_merkle_double2() {
    let mut hasher = blake3_merkle::Hasher::new();
    hasher.update(b"123123");
    hasher.update(b"qweqwe");
    let mut hash = [0; 32];
    let hash1 = hasher.finalize(&mut hash);

    let mut hasher = blake3_merkle::Hasher::new();
    hasher.update(&mut hash);
    hasher.finalize(&mut hash);
}

#[bench]
fn blake3_merkle(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..10000 {
            blake3_merkle_double2()
        }
    })
}

#[bench]
fn blake3(b: &mut Bencher) {
    b.iter(|| {
        for _ in 0..10000 {
            blake3_double2()
        }
    })
}



