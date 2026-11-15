use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::time::Instant;
use wincode::{SchemaRead, SchemaWrite};

#[derive(Serialize, Deserialize, SchemaWrite, SchemaRead, Debug, PartialEq, Clone)]
struct SimpleStruct {
    id: u64,
    value: u64,
    flag: bool,
}

#[repr(C)]
#[derive(Serialize, Deserialize, SchemaWrite, SchemaRead, Debug, PartialEq, Clone, Copy)]
struct PodStruct {
    a: [u8; 32],
    b: [u8; 16],
    c: [u8; 8],
}

#[derive(Serialize, Deserialize, SchemaWrite, SchemaRead, Debug, PartialEq, Clone)]
struct Mixed<'a> {
    name: &'a str,
    values: Vec<u32>,
    opt: Option<u64>,
    tag: u8,
}

const VARIANTS: usize = 8;

fn idx(i: u32) -> usize {
    (black_box(i) % VARIANTS as u32) as usize
}

/// Each iteration receives a runtime-varying index so that input pointers change
/// and LLVM cannot hoist the whole (otherwise pure) serialize/deserialize call.
fn bench_indexed<F, O>(iters: u32, mut f: F) -> f64
where
    F: FnMut(u32) -> O,
{
    for i in 0..(iters / 20).max(100) {
        black_box(f(i));
    }
    let mut best = f64::MAX;
    for _ in 0..5 {
        let t = Instant::now();
        for i in 0..iters {
            black_box(f(i));
        }
        let ns = t.elapsed().as_nanos() as f64 / iters as f64;
        if ns < best {
            best = ns;
        }
    }
    best
}

fn fmt_ns(ns: f64) -> String {
    if ns >= 1_000_000.0 {
        format!("{:>9.1} ms", ns / 1_000_000.0)
    } else if ns >= 1_000.0 {
        format!("{:>9.1} us", ns / 1_000.0)
    } else {
        format!("{:>9.1} ns", ns)
    }
}

macro_rules! run_case {
    ($label:expr, $base:expr, $ty:ty, $iters:expr, $buf_size:expr) => {{
        let base = $base;
        let vals: Vec<$ty> = (0..VARIANTS).map(|_| base.clone()).collect();
        let pb: Vec<Vec<u8>> = vals.iter().map(|v| postcard::to_stdvec(v).unwrap()).collect();
        let wb: Vec<Vec<u8>> = vals.iter().map(|v| wincode::serialize(v).unwrap()).collect();
        let bb: Vec<Vec<u8>> = vals.iter().map(|v| bincode::serialize(v).unwrap()).collect();
        for i in 0..VARIANTS {
            assert_eq!(postcard::from_bytes::<$ty>(&pb[i]).unwrap(), vals[i]);
            assert_eq!(wincode::deserialize::<$ty>(&wb[i]).unwrap(), vals[i]);
            assert_eq!(bincode::deserialize::<$ty>(&bb[i]).unwrap(), vals[i]);
            assert_eq!(wb[i], bb[i], "wincode must be bincode wire compatible");
        }

        let it: u32 = $iters;
        let ps = bench_indexed(it, |i| {
            black_box(postcard::to_stdvec(black_box(&vals[idx(i)])).unwrap())
        });
        let ws = bench_indexed(it, |i| {
            black_box(wincode::serialize(black_box(&vals[idx(i)])).unwrap())
        });
        let bs = bench_indexed(it, |i| {
            black_box(bincode::serialize(black_box(&vals[idx(i)])).unwrap())
        });
        let pd = bench_indexed(it, |i| {
            black_box(postcard::from_bytes::<$ty>(black_box(&pb[idx(i)])).unwrap())
        });
        let wd = bench_indexed(it, |i| {
            black_box(wincode::deserialize::<$ty>(black_box(&wb[idx(i)])).unwrap())
        });
        let bd = bench_indexed(it, |i| {
            black_box(bincode::deserialize::<$ty>(black_box(&bb[idx(i)])).unwrap())
        });

        println!(
            "{:<28} {:>12} {:>11} {:>11} {:>11} | {:>11} {:>11} {:>11}",
            $label,
            format!("{}/{}/{}", pb[0].len(), wb[0].len(), bb[0].len()),
            fmt_ns(ps), fmt_ns(ws), fmt_ns(bs), fmt_ns(pd), fmt_ns(wd), fmt_ns(bd)
        );
        println!(
            "    -> wincode / postcard:  ser {:.2}x faster,  de {:.2}x faster   (bytes: postcard {:.1}% of wincode)",
            ps / ws,
            pd / wd,
            100.0 * pb[0].len() as f64 / wb[0].len() as f64
        );

        // Second view: no heap allocation, write straight into a caller-owned buffer.
        // This is the no_std / embedded shape where postcard is designed to shine.
        let cap: usize = $buf_size;
        let mut pbuf = vec![0u8; cap];
        let mut wbuf = vec![0u8; cap];
        let ps2 = bench_indexed(it, |i| {
            let n = postcard::to_slice(black_box(&vals[idx(i)]), black_box(&mut pbuf[..]))
                .unwrap()
                .len();
            black_box(n)
        });
        let ws2 = bench_indexed(it, |i| {
            let dst: &mut [u8] = &mut wbuf[..];
            black_box(wincode::serialize_into(black_box(dst), black_box(&vals[idx(i)])).unwrap())
        });
        println!(
            "    -> into fixed buffer:   postcard {}   wincode {}   ({:.2}x)",
            fmt_ns(ps2), fmt_ns(ws2), ps2 / ws2
        );
    }};
}

fn main() {
    let out = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&out.stdout).trim());
    println!("OS: macOS / rustc release, LTO=thin, opt-level=3; 5 rounds, best-of; varying input pointers to prevent hoisting\n");
    println!(
        "{:<28} {:>12} {:>11} {:>11} {:>11} | {:>11} {:>11} {:>11}",
        "case", "bytes p/w/b", "postcard S", "wincode S", "bincode S", "postcard D", "wincode D", "bincode D"
    );

    run_case!(
        "SimpleStruct x1",
        SimpleStruct { id: 42, value: 987654321, flag: true },
        SimpleStruct,
        200_000u32,
        32usize
    );

    run_case!(
        "Vec<u64> x1000",
        (0..1000).map(|i| i as u64 * 7).collect::<Vec<u64>>(),
        Vec<u64>,
        30_000u32,
        16_384usize
    );

    run_case!(
        "Vec<PodStruct> x1000",
        (0..1000)
            .map(|i| PodStruct {
                a: [i as u8; 32],
                b: [(i / 4) as u8; 16],
                c: [(i / 16) as u8; 8],
            })
            .collect::<Vec<PodStruct>>(),
        Vec<PodStruct>,
        10_000u32,
        65_536usize
    );

    run_case!(
        "Vec<u8> x4096",
        (0..4096).map(|i| (i % 251) as u8).collect::<Vec<u8>>(),
        Vec<u8>,
        50_000u32,
        8_192usize
    );

    run_case!(
        "Mixed struct (borrowed str)",
        Mixed {
            name: "hello world",
            values: (0..256).map(|i| i * 3).collect(),
            opt: Some(123456789),
            tag: 7,
        },
        Mixed<'_>,
        30_000u32,
        2_048usize
    );
}
