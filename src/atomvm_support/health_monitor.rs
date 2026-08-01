// Working health monitoring system for system diagnostics and status reporting
// Tracks system health metrics and provides real-time diagnostics

use crate::atomvm_support::performance::PerformanceMonitor;

/// System health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

/// Health metrics snapshot
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub status: HealthStatus,
    pub uptime: u64,
    pub total_operations: usize,
    pub error_rate: u32,
    pub average_latency: u64,
    pub memory_events: usize,
}

/// Health threshold configuration
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    pub error_rate_critical: u32,  // Percentage
    pub error_rate_degraded: u32,  // Percentage
    pub latency_critical: u64,     // Cycles
    pub latency_degraded: u64,     // Cycles
}

impl Default for HealthThresholds {
    fn default() -> Self {
        HealthThresholds {
            error_rate_critical: 50,
            error_rate_degraded: 20,
            latency_critical: 1000,
            latency_degraded: 500,
        }
    }
}

/// Health monitor for system diagnostics
pub struct HealthMonitor {
    thresholds: HealthThresholds,
    start_time: u64,
    total_operations: usize,
    total_errors: usize,
    memory_events: usize,
    check_count: u32,
    performance_monitor: PerformanceMonitor,
}

impl HealthMonitor {
    pub fn new() -> Self {
        HealthMonitor {
            thresholds: HealthThresholds::default(),
            start_time: 0,
            total_operations: 0,
            total_errors: 0,
            memory_events: 0,
            check_count: 0,
            performance_monitor: PerformanceMonitor::new(),
        }
    }

    pub fn with_thresholds(thresholds: HealthThresholds) -> Self {
        let mut monitor = Self::new();
        monitor.thresholds = thresholds;
        monitor
    }

    /// Start system monitoring
    pub fn start(&mut self, current_time: u64) {
        self.start_time = current_time;
    }

    /// Record an operation
    pub fn record_operation(&mut self, success: bool, latency: u64) {
        self.total_operations += 1;
        if !success {
            self.total_errors += 1;
        }
        // For performance tracking, record latency
        if success {
            self.performance_monitor.record_operation(
                crate::atomvm_support::message_dispatch::MessageOp::Get,
                latency,
            );
        }
    }

    /// Record a memory event
    pub fn record_memory_event(&mut self) {
        self.memory_events += 1;
    }

    /// Check system health
    pub fn check_health(&mut self, current_time: u64) -> HealthMetrics {
        self.check_count += 1;

        let uptime = if self.start_time > 0 {
            current_time - self.start_time
        } else {
            0
        };

        let error_rate = if self.total_operations == 0 {
            0
        } else {
            ((self.total_errors as u32 * 100) / (self.total_operations as u32))
        };

        let average_latency = self.performance_monitor.average_cycles();

        let status = self.determine_status(error_rate, average_latency);

        HealthMetrics {
            status,
            uptime,
            total_operations: self.total_operations,
            error_rate,
            average_latency,
            memory_events: self.memory_events,
        }
    }

    /// Determine health status based on metrics
    fn determine_status(&self, error_rate: u32, latency: u64) -> HealthStatus {
        if error_rate >= self.thresholds.error_rate_critical
            || latency >= self.thresholds.latency_critical
        {
            HealthStatus::Critical
        } else if error_rate >= self.thresholds.error_rate_degraded
            || latency >= self.thresholds.latency_degraded
        {
            HealthStatus::Degraded
        } else if self.total_operations > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Get health report
    pub fn get_report(&mut self, current_time: u64) -> HealthReport {
        let metrics = self.check_health(current_time);
        let recovery_eligible = matches!(metrics.status, HealthStatus::Degraded);

        HealthReport {
            metrics,
            checks_performed: self.check_count,
            recovery_eligible,
        }
    }

    /// Reset health monitor
    pub fn reset(&mut self) {
        self.total_operations = 0;
        self.total_errors = 0;
        self.memory_events = 0;
        self.check_count = 0;
        self.performance_monitor.reset();
    }
}

/// Health report
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub metrics: HealthMetrics,
    pub checks_performed: u32,
    pub recovery_eligible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_thresholds_default() {
        let thresholds = HealthThresholds::default();
        assert_eq!(thresholds.error_rate_critical, 50);
        assert_eq!(thresholds.error_rate_degraded, 20);
    }

    #[test]
    fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        assert_eq!(monitor.total_operations, 0);
        assert_eq!(monitor.total_errors, 0);
    }

    #[test]
    fn test_health_monitor_start() {
        let mut monitor = HealthMonitor::new();
        monitor.start(1000);
        assert_eq!(monitor.start_time, 1000);
    }

    #[test]
    fn test_health_monitor_record_operation() {
        let mut monitor = HealthMonitor::new();
        monitor.record_operation(true, 100);
        monitor.record_operation(true, 105);

        assert_eq!(monitor.total_operations, 2);
        assert_eq!(monitor.total_errors, 0);
    }

    #[test]
    fn test_health_monitor_record_error() {
        let mut monitor = HealthMonitor::new();
        monitor.record_operation(true, 100);
        monitor.record_operation(false, 105);
        monitor.record_operation(true, 100);

        assert_eq!(monitor.total_operations, 3);
        assert_eq!(monitor.total_errors, 1);
    }

    #[test]
    fn test_health_monitor_healthy_status() {
        let mut monitor = HealthMonitor::new();
        monitor.start(0);
        monitor.record_operation(true, 100);
        monitor.record_operation(true, 100);

        let metrics = monitor.check_health(1000);
        assert_eq!(metrics.status, HealthStatus::Healthy);
        assert_eq!(metrics.error_rate, 0);
    }

    #[test]
    fn test_health_monitor_degraded_status() {
        let mut monitor = HealthMonitor::new();
        monitor.start(0);

        // Record operations with 30% error rate
        for _ in 0..7 {
            monitor.record_operation(true, 100);
        }
        for _ in 0..3 {
            monitor.record_operation(false, 100);
        }

        let metrics = monitor.check_health(1000);
        assert_eq!(metrics.status, HealthStatus::Degraded);
        assert_eq!(metrics.error_rate, 30);
    }

    #[test]
    fn test_health_monitor_critical_status() {
        let mut monitor = HealthMonitor::new();
        monitor.start(0);

        // Record operations with 60% error rate
        for _ in 0..4 {
            monitor.record_operation(true, 100);
        }
        for _ in 0..6 {
            monitor.record_operation(false, 100);
        }

        let metrics = monitor.check_health(1000);
        assert_eq!(metrics.status, HealthStatus::Critical);
        assert_eq!(metrics.error_rate, 60);
    }

    #[test]
    fn test_health_monitor_uptime() {
        let mut monitor = HealthMonitor::new();
        monitor.start(100);

        let metrics = monitor.check_health(1100);
        assert_eq!(metrics.uptime, 1000);
    }

    #[test]
    fn test_health_monitor_memory_events() {
        let mut monitor = HealthMonitor::new();
        monitor.record_memory_event();
        monitor.record_memory_event();
        monitor.record_memory_event();

        let metrics = monitor.check_health(0);
        assert_eq!(metrics.memory_events, 3);
    }

    #[test]
    fn test_health_monitor_get_report() {
        let mut monitor = HealthMonitor::new();
        monitor.start(0);
        monitor.record_operation(true, 100);

        let report = monitor.get_report(1000);
        assert_eq!(report.metrics.status, HealthStatus::Healthy);
        assert_eq!(report.checks_performed, 1);
    }

    #[test]
    fn test_health_monitor_reset() {
        let mut monitor = HealthMonitor::new();
        monitor.record_operation(true, 100);
        monitor.record_operation(false, 100);
        monitor.record_memory_event();

        assert_eq!(monitor.total_operations, 2);
        assert_eq!(monitor.total_errors, 1);

        monitor.reset();

        assert_eq!(monitor.total_operations, 0);
        assert_eq!(monitor.total_errors, 0);
        assert_eq!(monitor.memory_events, 0);
    }

    #[test]
    fn test_health_monitor_unknown_status() {
        let mut monitor = HealthMonitor::new();
        let metrics = monitor.check_health(0);
        assert_eq!(metrics.status, HealthStatus::Unknown);
    }

    #[test]
    fn test_health_monitor_custom_thresholds() {
        let thresholds = HealthThresholds {
            error_rate_critical: 10,
            error_rate_degraded: 5,
            latency_critical: 200,
            latency_degraded: 100,
        };

        let mut monitor = HealthMonitor::with_thresholds(thresholds);
        monitor.start(0);

        // 15% error rate should trigger degraded with custom threshold
        for _ in 0..85 {
            monitor.record_operation(true, 100);
        }
        for _ in 0..15 {
            monitor.record_operation(false, 100);
        }

        let metrics = monitor.check_health(1000);
        assert_eq!(metrics.status, HealthStatus::Critical);
    }
}
