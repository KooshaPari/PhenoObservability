fn main() { println!("PhenoObservability"); }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_runs_without_panic() {
        main();
    }
}
