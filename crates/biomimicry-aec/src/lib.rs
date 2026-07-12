//! AEC reference application (Part VIII) — wall-move scenario.
//!
//! Scaffold only. Built last (Milestone 9).

/// Placeholder entry so the crate type-checks before the scenario lands.
#[must_use]
pub fn scenario_name() -> &'static str {
    "wall-move"
}

#[cfg(test)]
mod tests {
    #[test]
    fn scaffold_crate_loads() {
        assert_eq!(super::scenario_name(), "wall-move");
    }
}
