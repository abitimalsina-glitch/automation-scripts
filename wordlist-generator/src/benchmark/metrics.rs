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
