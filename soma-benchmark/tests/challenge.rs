use soma_benchmark::challenge::LeafReader;
use soma_benchmark::file::generate;

#[test]
fn leaf_reader_buffered_matches_file_contents() -> std::io::Result<()> {
    let p = std::env::temp_dir().join("soma_leaf_reader_buffered.bin");
    generate(&p, 3 * 4096 + 77)?;
    let bytes = std::fs::read(&p)?;
    let result = (|| {
        for leaf_size in [1, 100, 1024, 5000] {
            let mut reader = LeafReader::open(&p, leaf_size, false)?;
            for (i, expected) in bytes.chunks(leaf_size).enumerate() {
                assert_eq!(reader.read(i)?, expected, "leaf_size={leaf_size} i={i}");
            }
        }
        Ok(())
    })();
    std::fs::remove_file(&p)?;
    result
}
