#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate criterion;

use rand::prelude::*;
use std::mem::MaybeUninit;
use core::time::Duration;
use criterion::{BatchSize, Criterion, ParameterizedBenchmark, Throughput, black_box};

const LEN: usize = 1_000_000;
const WARM_UP_TIME: Duration = Duration::from_secs(1);
const MEASUREMENT_TIME: Duration = Duration::from_secs(5);

fn cmp(c: &mut Criterion) {
    //let core_ids = core_affinity::get_core_ids().unwrap();
    //core_affinity::set_for_current(core_ids[0]);

    let mut rng = rand::thread_rng();
    let mut bin = vec![0; 2048];
    rng.fill_bytes(&mut bin);

    c.bench("base64", ParameterizedBenchmark::new(
        "from",
        |b, data| {
            b.iter_batched(
                || base64::encode_config(&data[..], base64::URL_SAFE),
                |value| {
                    black_box(base64::decode_config(&value, base64::URL_SAFE).unwrap())
                },
                BatchSize::NumIterations(LEN as u64),
            )
        },
        vec![bin.clone()],
    )
    .with_function("to", |b, data| {
        b.iter_batched(
            || data,
            |value| {
                black_box(base64::encode_config(&value[..], base64::URL_SAFE))
            },
            BatchSize::NumIterations(LEN as u64),
        )
    },)
    .throughput(|d| Throughput::Bytes(d.len() as u64))
    .warm_up_time(WARM_UP_TIME)
    .measurement_time(MEASUREMENT_TIME));

    c.bench("radix64", ParameterizedBenchmark::new(
        "from",
        |b, data| {
            b.iter_batched(
                || base64::encode_config(&data[..], base64::URL_SAFE),
                |value| {
                    black_box(radix64::URL_SAFE.decode(value.as_bytes()).unwrap())
                },
                BatchSize::NumIterations(LEN as u64),
            )
        },
        vec![bin.clone()],
    )
    .with_function("to", |b, data| {
        b.iter_batched(
            || data,
            |value| {
                black_box(radix64::URL_SAFE.encode(&value[..]))
            },
            BatchSize::NumIterations(LEN as u64),
        )
    },)
    .throughput(|d| Throughput::Bytes(d.len() as u64))
    .warm_up_time(WARM_UP_TIME)
    .measurement_time(MEASUREMENT_TIME));

    c.bench("hex", ParameterizedBenchmark::new(
        "from",
        |b, data| {
            b.iter_batched(
                || hex::encode(&data[..]),
                |value| {
                    black_box(hex::decode(&value).unwrap())
                },
                BatchSize::NumIterations(LEN as u64),
            )
        },
        vec![bin.clone()],
    )
    .with_function("to", |b, data| {
        b.iter_batched(
            || data,
            |value| {
                black_box(hex::encode(&value[..]))
            },
            BatchSize::NumIterations(LEN as u64),
        )
    })
    .throughput(|d| Throughput::Bytes(d.len() as u64))
    .warm_up_time(WARM_UP_TIME)
    .measurement_time(MEASUREMENT_TIME));

    c.bench("base16", ParameterizedBenchmark::new(
        "from",
        |b, data| {
            b.iter_batched(
                || hex::encode(&data[..]),
                |value| {
                    black_box(base16::decode(&value).unwrap())
                },
                BatchSize::NumIterations(LEN as u64),
            )
        },
        vec![bin.clone()],
    )
    .with_function("to", |b, data| {
        b.iter_batched(
            || data,
            |value| {
                black_box(base16::encode_lower(&value[..]))
            },
            BatchSize::NumIterations(LEN as u64),
        )
    })
    .throughput(|d| Throughput::Bytes(d.len() as u64))
    .warm_up_time(WARM_UP_TIME)
    .measurement_time(MEASUREMENT_TIME));

    c.bench("faster-hex", ParameterizedBenchmark::new(
        "from",
        |b, data| {
            b.iter_batched(
                || hex::encode(&data[..]),
                |value| {
                    let l = value.len() >> 1;
                    let mut v= Vec::with_capacity(l);
                    v.resize(l, 0);
                    black_box((faster_hex::hex_decode(value.as_bytes(), &mut v[..]).unwrap(), v))
                },
                BatchSize::NumIterations(LEN as u64),
            )
        },
        vec![bin.clone()],
    )
    .with_function("to", |b, data| {
        b.iter_batched(
            || data,
            |value| {
                let l = value.len() << 1;
                let mut v= Vec::with_capacity(l);
                v.resize(l, 0);
                black_box((faster_hex::hex_encode(value, &mut v[..]).unwrap(), v))
            },
            BatchSize::NumIterations(LEN as u64),
        )
    })
    .throughput(|d| Throughput::Bytes(d.len() as u64))
    .warm_up_time(WARM_UP_TIME)
    .measurement_time(MEASUREMENT_TIME));

    c.bench("bintext", ParameterizedBenchmark::new(
        "from",
        |b, data| {
            let hex = hex::encode(&data);
            assert_eq!(hex::decode(&hex).unwrap(), bintext::hex::decode_no(&hex).unwrap());
            b.iter_batched(
                || &hex,
                |value| {
                    black_box(bintext::hex::decode_no(value).unwrap())
                },
                BatchSize::NumIterations(LEN as u64),
            )
        },
        vec![bin.clone()],
    )
    .with_function("to", |b, data| {
        assert_eq!(hex::encode(&data), bintext::hex::encode(&data[..]));
        b.iter_batched(
            || data,
            |value| {
                black_box(bintext::hex::encode(&value[..]))
            },
            BatchSize::NumIterations(LEN as u64),
        )
    },)
    .throughput(|d| Throughput::Bytes(d.len() as u64))
    .warm_up_time(WARM_UP_TIME)
    .measurement_time(MEASUREMENT_TIME));
}

criterion_group!(benches, cmp);
criterion_main!(benches);