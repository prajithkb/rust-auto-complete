use completion::{naive::NaiveAutoComplete, trie::Trie};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use io::BufReader;
use std::io::{self, BufRead};
use std::{fs::File, time::Instant};

fn input() -> Vec<(String, u32)> {
    let file =
        File::open("/Users/kprajith/workspace/rust/auto-complete/100_000_words.txt").unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap())
        .map(|r| (r.clone(), r.len() as u32))
        .collect::<Vec<(String, u32)>>()
}

fn prefix() -> &'static [&'static str] {
    &[
        "imm", "ca", "di", "impe", "inter", "pre", "trans", "sub", "non", "un", "mid", "anti",
        "in", "re", "over",
    ]
}

fn validate_outputs(prefixes: &[&str], trie: &Trie, naive: &NaiveAutoComplete) {
    prefixes
        .iter()
        .for_each(|p|{
            assert_eq!(trie.suggestions(*p), naive.suggestions(*p));
        });
    println!("Validated that the outputs match!");
}

fn completion_bench_tests(c: &mut Criterion) {
    println!("Running completion_bench_tests...");
    let start = Instant::now();
    let inp = input();
    println!("Input created in {} ns", start.elapsed().as_nanos());
    let before_input = Instant::now();
    let input = inp
        .iter()
        .map(|(s, u)| (&s[..], *u))
        .collect::<Vec<(&str, u32)>>();
    println!(
        "Input transformed in {} ns",
        before_input.elapsed().as_nanos()
    );
    let before_prefix = Instant::now();
    let prefix = prefix();
    println!(
        "Prefix created in {} ns",
        before_prefix.elapsed().as_nanos()
    );
    let before_trie = Instant::now();
    let trie = Trie::new(&input[..]);
    println!(
        "Trie auto-complete created in {} ns",
        before_trie.elapsed().as_nanos()
    );
    let before_naive = Instant::now();
    let naive_autocomplete = NaiveAutoComplete::new(&input[..]);
    println!(
        "Naive auto-complete created in {} ns",
        before_naive.elapsed().as_nanos()
    );
    validate_outputs(prefix, &trie, &naive_autocomplete);
    let mut group = c.benchmark_group("Performance");
    for prefix in prefix.iter() {
        group.bench_with_input(
            BenchmarkId::new("Trie", format!("{}/{}", prefix, prefix.len())),
            prefix,
            |b, &prefix| {
                b.iter(|| trie.suggestions(prefix));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Naive", format!("{}/{}", prefix, prefix.len())),
            prefix,
            |b, &prefix| {
                b.iter(|| naive_autocomplete.suggestions(prefix));
            },
        );
    }
    group.finish();
}
criterion_group!(benches, completion_bench_tests);
criterion_main!(benches);
