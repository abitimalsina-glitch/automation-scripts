use std::time::Duration;

/// Performance metrics for a completed dataset generation run.
#[derive(Debug)]
pub struct DatasetMetrics {
    pub total_records: u64,
    pub total_bytes: u64,
    pub elapsed: Duration,
}

impl DatasetMetrics {
    /// Records generated per second.
    pub fn records_per_second(&self) -> f64 {
        let secs = self.elapsed.as_secs_f64();
        if secs == 0.0 {
            return 0.0;
        }
        self.total_records as f64 / secs
    }

    /// Output throughput in megabytes per second.
    pub fn throughput_mbps(&self) -> f64 {
        let secs = self.elapsed.as_secs_f64();
        if secs == 0.0 {
            return 0.0;
        }
        (self.total_bytes as f64 / (1024.0 * 1024.0)) / secs
    }

    /// Average bytes written per record (including the newline).
    pub fn avg_bytes_per_record(&self) -> f64 {
        if self.total_records == 0 {
            return 0.0;
        }
        self.total_bytes as f64 / self.total_records as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_per_second_known_value() {
        let m = DatasetMetrics {
            total_records: 10_000,
            total_bytes: 170_000,
            elapsed: Duration::from_secs(2),
        };
        assert!((m.records_per_second() - 5_000.0).abs() < 1e-6);
    }

    #[test]
    fn throughput_mbps_known_value() {
        let m = DatasetMetrics {
            total_records: 1_000,
            total_bytes: 1_048_576, // exactly 1 MiB
            elapsed: Duration::from_secs(1),
        };
        assert!((m.throughput_mbps() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn avg_bytes_per_record_known_value() {
        let m = DatasetMetrics {
            total_records: 500,
            total_bytes: 8_500,
            elapsed: Duration::from_secs(1),
        };
        assert!((m.avg_bytes_per_record() - 17.0).abs() < 1e-6);
    }

    #[test]
    fn zero_duration_returns_zero_rates() {
        let m = DatasetMetrics {
            total_records: 100,
            total_bytes: 1_700,
            elapsed: Duration::ZERO,
        };
        assert_eq!(m.records_per_second(), 0.0);
        assert_eq!(m.throughput_mbps(), 0.0);
    }

    #[test]
    fn zero_records_returns_zero_avg() {
        let m = DatasetMetrics {
            total_records: 0,
            total_bytes: 0,
            elapsed: Duration::from_secs(1),
        };
        assert_eq!(m.avg_bytes_per_record(), 0.0);
    }

    #[test]
    fn large_dataset_metrics() {
        let m = DatasetMetrics {
            total_records: 1_000_000,
            total_bytes: 17_000_000,
            elapsed: Duration::from_millis(500),
        };
        // 1,000,000 records / 0.5 seconds = 2,000,000 rps
        assert!((m.records_per_second() - 2_000_000.0).abs() < 1.0);
        // 17,000,000 bytes / (1024*1024) / 0.5 ≈ 32.425 MB/s
        assert!(m.throughput_mbps() > 30.0);
        // 17,000,000 / 1,000,000 = 17.0
        assert!((m.avg_bytes_per_record() - 17.0).abs() < 1e-6);
    }
}
