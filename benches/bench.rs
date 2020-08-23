#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate criterion;

use core::time::Duration;
use criterion::{black_box, BatchSize, Criterion, ParameterizedBenchmark, Throughput};
use rand::prelude::*;
use std::fmt;

const LEN: usize = 1_000_000;
const WARM_UP_TIME: Duration = Duration::from_secs(1);
const MEASUREMENT_TIME: Duration = Duration::from_secs(5);

#[derive(Clone, PartialEq, Eq)]
struct Bytes(Vec<u8>);

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bytes", self.0.len())
    }
}

fn cmp(c: &mut Criterion) {
    let core_ids = core_affinity::get_core_ids().unwrap();
    core_affinity::set_for_current(core_ids[0]);

    // Defines the tests to a wider range of byte slices
    let mut test_set = vec![];
    let mut rng = rand::thread_rng();
    for i in [3, 8, 20, 32, 80, 150, 256, 600, 951, 1532, 2048, 3254].iter() {
        let mut bin = vec![0; *i];
        rng.fill_bytes(&mut bin);
        test_set.push(Bytes(bin));
    }

    c.bench(
        "decode",
        ParameterizedBenchmark::new(
            "base64",
            |b, data| {
                b.iter_batched(
                    || base64::encode_config(&data.0[..], base64::URL_SAFE),
                    |value| black_box(base64::decode_config(&value, base64::URL_SAFE).unwrap()),
                    BatchSize::NumIterations(LEN as u64),
                )
            },
            test_set.clone(),
        )
        .with_function("radix64", |b, data| {
            b.iter_batched(
                || base64::encode_config(&data.0[..], base64::URL_SAFE),
                |value| black_box(radix64::URL_SAFE.decode(value.as_bytes()).unwrap()),
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .throughput(|d| Throughput::Bytes(d.0.len() as u64))
        .warm_up_time(WARM_UP_TIME)
        .measurement_time(MEASUREMENT_TIME),
    );

    c.bench(
        "encode",
        ParameterizedBenchmark::new(
            "base64",
            |b, data| {
                b.iter_batched(
                    || &data.0,
                    |value| black_box(base64::encode_config(&value[..], base64::URL_SAFE)),
                    BatchSize::NumIterations(LEN as u64),
                )
            },
            test_set.clone(),
        )
        .with_function("radix64", |b, data| {
            b.iter_batched(
                || &data.0,
                |value| black_box(radix64::URL_SAFE.encode(&value[..])),
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .throughput(|d| Throughput::Bytes(d.0.len() as u64))
        .warm_up_time(WARM_UP_TIME)
        .measurement_time(MEASUREMENT_TIME),
    );

    ///////////////////////////////////////////////////////////////////////////////

    c.bench(
        "decode",
        ParameterizedBenchmark::new(
            "hex",
            |b, data| {
                b.iter_batched(
                    || hex::encode(&data.0[..]),
                    |value| black_box(hex::decode(&value).unwrap()),
                    BatchSize::NumIterations(LEN as u64),
                )
            },
            test_set.clone(),
        )
        .with_function("base16", |b, data| {
            b.iter_batched(
                || hex::encode(&data.0[..]),
                |value| black_box(base16::decode(&value).unwrap()),
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .with_function("faster-hex", |b, data| {
            b.iter_batched(
                || hex::encode(&data.0[..]),
                |value| {
                    let l = value.len() >> 1;
                    let mut v = Vec::with_capacity(l);
                    v.resize(l, 0);
                    black_box((
                        faster_hex::hex_decode(value.as_bytes(), &mut v[..]).unwrap(),
                        v,
                    ))
                },
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .with_function("bintext", |b, data| {
            let hex = hex::encode(&data.0);
            assert_eq!(
                hex::decode(&hex).unwrap(),
                bintext::hex::decode_noerr(&hex).unwrap()
            );
            b.iter_batched(
                || &hex,
                |value| black_box(bintext::hex::decode_noerr(value).unwrap()),
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .throughput(|d| Throughput::Bytes(d.0.len() as u64))
        .warm_up_time(WARM_UP_TIME)
        .measurement_time(MEASUREMENT_TIME),
    );

    c.bench(
        "encode",
        ParameterizedBenchmark::new(
            "hex",
            |b, data| {
                b.iter_batched(
                    || &data.0,
                    |value| black_box(hex::encode(&value[..])),
                    BatchSize::NumIterations(LEN as u64),
                )
            },
            test_set.clone(),
        )
        .with_function("base16", |b, data| {
            b.iter_batched(
                || &data.0,
                |value| black_box(base16::encode_lower(&value[..])),
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .with_function("faster-hex", |b, data| {
            b.iter_batched(
                || &data.0,
                |value| {
                    let l = value.len() << 1;
                    let mut v = Vec::with_capacity(l);
                    v.resize(l, 0);
                    black_box((faster_hex::hex_encode(value, &mut v[..]).unwrap(), v))
                },
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .with_function("bintext", |b, data| {
            assert_eq!(hex::encode(&data.0), bintext::hex::encode(&data.0[..]));
            b.iter_batched(
                || &data.0,
                |value| black_box(bintext::hex::encode(&value[..])),
                BatchSize::NumIterations(LEN as u64),
            )
        })
        .throughput(|d| Throughput::Bytes(d.0.len() as u64))
        .warm_up_time(WARM_UP_TIME)
        .measurement_time(MEASUREMENT_TIME),
    );
}

criterion_group!(benches, cmp);
criterion_main!(benches);
