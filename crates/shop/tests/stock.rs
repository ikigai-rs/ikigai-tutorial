//! Audit the stock the chapter actually teaches: a replaceable CSV table, with
//! quantities interpreted by jq, rather than an order or money model.
use std::sync::{Arc, Barrier};

use futures::executor::block_on;
use ikigai_core::{Capability, Kernel};
use shop::{sink, source, Host, PREFIXES, PREFIX_TABLE, STOCK, SUMMARY};

#[test]
fn an_edit_recalculates_the_summary_with_zero_and_negative_counts() {
    let mut host = Host::start("audit");
    host.write(PREFIXES, PREFIX_TABLE);
    assert_eq!(
        host.read_in(source(SUMMARY, &[])).unwrap(),
        "4 items, 49 on the shelves; out of stock: sugar."
    );
    assert!(host.is_cached_in(&source(SUMMARY, &[])));
    host.write(
        STOCK,
        "item,shelf,count\nempty,A,0\nreturned,B,-2\nfull,C,5\n",
    );
    assert!(!host.is_cached_in(&source(SUMMARY, &[])));
    assert_eq!(
        host.read_in(source(SUMMARY, &[])).unwrap(),
        "3 items, 3 on the shelves; out of stock: empty."
    );
    assert_eq!(
        host.read_in(source(
            "urn:json:query",
            &[
                ("src", "shop:stock"),
                ("q", "[.[] | select(.item == \"unknown\")] | length"),
            ]
        ))
        .unwrap(),
        "0"
    );
}

#[test]
fn concurrent_replacements_store_one_whole_table() {
    let kernel = Arc::new(Kernel::new(Arc::new(shop::space(
        Arc::default(),
        Arc::default(),
    ))));
    let barrier = Arc::new(Barrier::new(3));
    let tables = ["item,shelf,count\na,A,0\n", "item,shelf,count\nb,B,-2\n"];
    // Prime the stock cache before both writes.
    block_on(kernel.issue(source(STOCK, &[]), &Capability::root())).unwrap();
    std::thread::scope(|scope| {
        for table in tables {
            let (kernel, barrier) = (Arc::clone(&kernel), Arc::clone(&barrier));
            scope.spawn(move || {
                barrier.wait();
                block_on(kernel.issue(sink(STOCK, table), &Capability::root())).unwrap();
            });
        }
        barrier.wait();
    });
    assert!(!kernel.is_cached(&source(STOCK, &[]), &Capability::root()));
    let answer = block_on(kernel.issue(source(STOCK, &[]), &Capability::root())).unwrap();
    let text = String::from_utf8(answer.bytes).unwrap();
    assert!(tables.contains(&text.as_str()), "{text}");
}
