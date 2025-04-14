#[cfg(test)]
mod tests {
    use std::io::Write;

    use ant_mania::Hiveum;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_map_and_simulate() {
        let mut temp_file =
            NamedTempFile::new().expect("Failed to create temporary file");
        writeln!(temp_file, "A north=B east=C")
            .expect("Failed to write to file");
        writeln!(temp_file, "B south=A").expect("Failed to write to file");
        writeln!(temp_file, "C west=A").expect("Failed to write to file");

        let file_path =
            temp_file.path().to_str().expect("Invalid temp file path");
        let seed = 42;
        let ant_count = 3;

        let mut hiveum = Hiveum::new(file_path, ant_count, seed);

        assert_eq!(hiveum.colonies.len(), 3, "There should be 3 colonies");
        assert_eq!(
            hiveum.name_to_index.len(),
            3,
            "There should be 3 entries in name_to_index"
        );
        assert_eq!(
            hiveum.active_flags.len(),
            3,
            "There should be 3 active_flags entries"
        );

        assert_eq!(
            hiveum.ants.len(),
            ant_count,
            "Number of spawned ants should match ant_count"
        );

        let simulation_duration = hiveum.simulate();
        assert!(
            simulation_duration.as_secs_f64() >= 0.0,
            "Simulation duration must be non-negative"
        );

        let status = hiveum.hive_status();
        assert!(!status.trim().is_empty(), "Hive status should not be empty");
    }
}
