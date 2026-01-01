// Performance monitoring and benchmarking module
// Tracks operation timing and statistics

use crate::atomvm_support::message_dispatch::MessageOp;

/// Performance metrics for operations
#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub operation: MessageOp,
    pub count: usize,
    pub total_cycles: u64,
    pub min_cycles: u64,
    pub max_cycles: u64,
}

impl OperationMetrics {
    pub fn new(operation: MessageOp) -> Self {
        OperationMetrics {
            operation,
            count: 0,
            total_cycles: 0,
            min_cycles: u64::MAX,
            max_cycles: 0,
        }
    }

    pub fn record(&mut self, cycles: u64) {
        self.count += 1;
        self.total_cycles += cycles;
        if cycles < self.min_cycles {
            self.min_cycles = cycles;
        }
        if cycles > self.max_cycles {
            self.max_cycles = cycles;
        }
    }

    pub fn average(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.total_cycles / (self.count as u64)
        }
    }

    pub fn throughput_per_second(&self) -> u64 {
        if self.total_cycles == 0 {
            0
        } else {
            // Simplified: assumes 1 GHz = 1 billion cycles per second
            (self.count as u64 * 1_000_000_000) / self.total_cycles
        }
    }
}

/// Performance monitor for tracking system metrics
pub struct PerformanceMonitor {
    inc_metrics: OperationMetrics,
    dec_metrics: OperationMetrics,
    get_metrics: OperationMetrics,
    reset_metrics: OperationMetrics,
    total_operations: usize,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        PerformanceMonitor {
            inc_metrics: OperationMetrics::new(MessageOp::Inc),
            dec_metrics: OperationMetrics::new(MessageOp::Dec),
            get_metrics: OperationMetrics::new(MessageOp::Get),
            reset_metrics: OperationMetrics::new(MessageOp::Reset),
            total_operations: 0,
        }
    }

    /// Record operation timing
    pub fn record_operation(&mut self, op: MessageOp, cycles: u64) {
        match op {
            MessageOp::Inc => self.inc_metrics.record(cycles),
            MessageOp::Dec => self.dec_metrics.record(cycles),
            MessageOp::Get => self.get_metrics.record(cycles),
            MessageOp::Reset => self.reset_metrics.record(cycles),
            MessageOp::Unknown => {}
        }
        self.total_operations += 1;
    }

    /// Get metrics for specific operation
    pub fn get_metrics(&self, op: MessageOp) -> Option<&OperationMetrics> {
        match op {
            MessageOp::Inc => Some(&self.inc_metrics),
            MessageOp::Dec => Some(&self.dec_metrics),
            MessageOp::Get => Some(&self.get_metrics),
            MessageOp::Reset => Some(&self.reset_metrics),
            MessageOp::Unknown => None,
        }
    }

    /// Get total operations
    pub fn total_operations(&self) -> usize {
        self.total_operations
    }

    /// Get average cycles per operation
    pub fn average_cycles(&self) -> u64 {
        if self.total_operations == 0 {
            0
        } else {
            let total: u64 = [
                self.inc_metrics.total_cycles,
                self.dec_metrics.total_cycles,
                self.get_metrics.total_cycles,
                self.reset_metrics.total_cycles,
            ]
            .iter()
            .sum();

            total / (self.total_operations as u64)
        }
    }

    /// Get fastest operation type
    pub fn fastest_operation(&self) -> (MessageOp, u64) {
        let ops = [
            (MessageOp::Inc, self.inc_metrics.average()),
            (MessageOp::Dec, self.dec_metrics.average()),
            (MessageOp::Get, self.get_metrics.average()),
            (MessageOp::Reset, self.reset_metrics.average()),
        ];

        ops.iter()
            .filter(|(_, avg)| *avg > 0)
            .min_by_key(|(_, avg)| avg)
            .map(|(op, avg)| (*op, *avg))
            .unwrap_or((MessageOp::Unknown, 0))
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.inc_metrics = OperationMetrics::new(MessageOp::Inc);
        self.dec_metrics = OperationMetrics::new(MessageOp::Dec);
        self.get_metrics = OperationMetrics::new(MessageOp::Get);
        self.reset_metrics = OperationMetrics::new(MessageOp::Reset);
        self.total_operations = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_metrics_creation() {
        let metrics = OperationMetrics::new(MessageOp::Inc);
        assert_eq!(metrics.count, 0);
        assert_eq!(metrics.average(), 0);
    }

    #[test]
    fn test_operation_metrics_recording() {
        let mut metrics = OperationMetrics::new(MessageOp::Inc);

        metrics.record(100);
        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.total_cycles, 100);
        assert_eq!(metrics.min_cycles, 100);
        assert_eq!(metrics.max_cycles, 100);
        assert_eq!(metrics.average(), 100);
    }

    #[test]
    fn test_operation_metrics_multiple_recordings() {
        let mut metrics = OperationMetrics::new(MessageOp::Inc);

        metrics.record(100);
        metrics.record(200);
        metrics.record(50);

        assert_eq!(metrics.count, 3);
        assert_eq!(metrics.total_cycles, 350);
        assert_eq!(metrics.min_cycles, 50);
        assert_eq!(metrics.max_cycles, 200);
        assert_eq!(metrics.average(), 116);
    }

    #[test]
    fn test_operation_metrics_throughput() {
        let mut metrics = OperationMetrics::new(MessageOp::Inc);

        // Record 10 operations taking 10 cycles total
        for _ in 0..10 {
            metrics.record(1);
        }

        let throughput = metrics.throughput_per_second();
        assert!(throughput > 0);
    }

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.total_operations(), 0);
        assert_eq!(monitor.average_cycles(), 0);
    }

    #[test]
    fn test_performance_monitor_recording() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_operation(MessageOp::Inc, 100);
        monitor.record_operation(MessageOp::Dec, 90);
        monitor.record_operation(MessageOp::Get, 80);

        assert_eq!(monitor.total_operations(), 3);
        assert!(monitor.average_cycles() > 0);
    }

    #[test]
    fn test_performance_monitor_get_metrics() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_operation(MessageOp::Inc, 100);
        monitor.record_operation(MessageOp::Inc, 110);

        let inc_metrics = monitor.get_metrics(MessageOp::Inc).unwrap();
        assert_eq!(inc_metrics.count, 2);
        assert_eq!(inc_metrics.average(), 105);
    }

    #[test]
    fn test_performance_monitor_fastest_operation() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_operation(MessageOp::Inc, 100);
        monitor.record_operation(MessageOp::Dec, 90);
        monitor.record_operation(MessageOp::Get, 80);

        let (fastest_op, fastest_avg) = monitor.fastest_operation();
        assert_eq!(fastest_op, MessageOp::Get);
        assert_eq!(fastest_avg, 80);
    }

    #[test]
    fn test_performance_monitor_reset() {
        let mut monitor = PerformanceMonitor::new();

        monitor.record_operation(MessageOp::Inc, 100);
        assert_eq!(monitor.total_operations(), 1);

        monitor.reset();
        assert_eq!(monitor.total_operations(), 0);
        assert_eq!(monitor.average_cycles(), 0);
    }

    #[test]
    fn test_performance_monitor_mixed_operations() {
        let mut monitor = PerformanceMonitor::new();

        for _ in 0..100 {
            monitor.record_operation(MessageOp::Inc, 100);
            monitor.record_operation(MessageOp::Dec, 95);
            monitor.record_operation(MessageOp::Get, 80);
        }

        assert_eq!(monitor.total_operations(), 300);
        let inc_metrics = monitor.get_metrics(MessageOp::Inc).unwrap();
        assert_eq!(inc_metrics.count, 100);
    }
}
