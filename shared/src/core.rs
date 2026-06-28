pub fn greeting(name: &str) -> String {
    let trimmed = name.trim();
    let name = if trimmed.is_empty() { "there" } else { trimmed };
    format!("Hello, {name}. This message came from shared Rust.")
}

pub fn lattice_score(seed: i32) -> i32 {
    let value = seed.rem_euclid(97);
    (value * value + 17).rem_euclid(97)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeting_falls_back_for_blank_names() {
        assert_eq!(
            greeting("  "),
            "Hello, there. This message came from shared Rust."
        );
    }

    #[test]
    fn score_is_stable() {
        assert_eq!(lattice_score(5), 42);
        assert_eq!(lattice_score(-92), 42);
    }
}
