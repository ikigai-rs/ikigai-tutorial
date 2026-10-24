use book_transcripts::{matches, normalize};

#[test]
fn browser_and_native_share_match_cases() {
    for line in include_str!("fixtures/matches.tsv")
        .lines()
        .filter(|l| !l.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let decode = |s: &str| s.replace("\\n", "\n").replace("\\t", "\t");
        let expected = decode(fields[2]);
        let actual = decode(fields[3]);
        assert_eq!(
            matches(&normalize(expected.lines()), &normalize(actual.lines())),
            fields[1] == "1",
            "{}",
            fields[0]
        );
    }
}
