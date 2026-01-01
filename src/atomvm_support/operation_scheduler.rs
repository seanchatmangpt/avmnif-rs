// Working operation scheduler for batch processing and timing control
// Manages operation queues, batching, and execution scheduling

use alloc::vec::Vec;
use crate::atomvm_support::command_executor::{Command, CommandExecutor, CommandResult};

/// Scheduling priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Scheduled operation
#[derive(Debug, Clone)]
pub struct ScheduledOp {
    pub command: Command,
    pub priority: Priority,
    pub scheduled_time: u64,
    pub execution_time: Option<u64>,
    pub result: Option<CommandResult>,
}

impl ScheduledOp {
    pub fn new(command: Command, priority: Priority, scheduled_time: u64) -> Self {
        ScheduledOp {
            command,
            priority,
            scheduled_time,
            execution_time: None,
            result: None,
        }
    }
}

/// Scheduler for managing operation execution
pub struct OperationScheduler {
    executor: CommandExecutor,
    queue: Vec<ScheduledOp>,
    executed: Vec<ScheduledOp>,
    batch_size: usize,
    current_time: u64,
}

impl OperationScheduler {
    pub fn new(initial_value: i32) -> Self {
        OperationScheduler {
            executor: CommandExecutor::new(initial_value),
            queue: Vec::new(),
            executed: Vec::new(),
            batch_size: 10,
            current_time: 0,
        }
    }

    /// Set current simulation time
    pub fn set_time(&mut self, time: u64) {
        self.current_time = time;
    }

    /// Schedule an operation
    pub fn schedule(&mut self, command: Command, priority: Priority) {
        self.schedule_at(command, priority, self.current_time);
    }

    /// Schedule an operation for a specific time
    pub fn schedule_at(&mut self, command: Command, priority: Priority, time: u64) {
        self.queue.push(ScheduledOp::new(command, priority, time));

        // Keep queue sorted by priority (highest first) then by time
        self.queue.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.scheduled_time.cmp(&b.scheduled_time))
        });
    }

    /// Execute pending operations
    pub fn execute_pending(&mut self) -> Vec<CommandResult> {
        let mut results = Vec::new();
        let executable: Vec<_> = self
            .queue
            .iter()
            .filter(|op| op.scheduled_time <= self.current_time)
            .cloned()
            .collect();

        for mut op in executable {
            let result = self.executor.execute(op.command);
            op.execution_time = Some(self.current_time);
            op.result = Some(result.clone());

            results.push(result);
            self.executed.push(op.clone());

            // Remove from queue
            self.queue.retain(|queued| queued.scheduled_time != op.scheduled_time);
        }

        results
    }

    /// Execute a batch of operations
    pub fn execute_batch(&mut self, count: usize) -> Vec<CommandResult> {
        let take_count = core::cmp::min(count, self.queue.len());
        let batch: Vec<_> = self.queue.drain(0..take_count).collect();

        let mut results = Vec::new();
        for mut op in batch {
            let result = self.executor.execute(op.command);
            op.execution_time = Some(self.current_time);
            op.result = Some(result.clone());

            results.push(result);
            self.executed.push(op);
        }

        results
    }

    /// Get queue size
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Get executed count
    pub fn executed_count(&self) -> usize {
        self.executed.len()
    }

    /// Get current system value
    pub fn current_value(&self) -> i32 {
        self.executor.current_value()
    }

    /// Get scheduler statistics
    pub fn statistics(&self) -> (usize, usize, u32) {
        let (total, success, _error) = self.executor.statistics();
        (total as usize, self.executed.len(), success)
    }

    /// Set batch size
    pub fn set_batch_size(&mut self, size: usize) {
        self.batch_size = core::cmp::max(1, size);
    }

    /// Get pending operations at priority level
    pub fn pending_at_priority(&self, priority: Priority) -> Vec<&ScheduledOp> {
        self.queue
            .iter()
            .filter(|op| op.priority == priority)
            .collect()
    }

    /// Get executed operations of type
    pub fn executed_of_type(&self, cmd_type: Command) -> Vec<&ScheduledOp> {
        self.executed
            .iter()
            .filter(|op| op.command == cmd_type)
            .collect()
    }

    /// Cancel pending operations
    pub fn cancel_pending(&mut self, command: Command) -> usize {
        let initial_size = self.queue.len();
        self.queue.retain(|op| op.command != command);
        initial_size - self.queue.len()
    }

    /// Get next executable operation
    pub fn next_executable(&self) -> Option<&ScheduledOp> {
        self.queue
            .iter()
            .find(|op| op.scheduled_time <= self.current_time)
    }

    /// Clear all queues and reset
    pub fn reset(&mut self) {
        self.executor.reset();
        self.queue.clear();
        self.executed.clear();
        self.current_time = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = OperationScheduler::new(0);
        assert_eq!(scheduler.queue_size(), 0);
        assert_eq!(scheduler.current_value(), 0);
    }

    #[test]
    fn test_scheduler_schedule() {
        let mut scheduler = OperationScheduler::new(0);
        scheduler.schedule(Command::Increment, Priority::Normal);

        assert_eq!(scheduler.queue_size(), 1);
    }

    #[test]
    fn test_scheduler_schedule_at() {
        let mut scheduler = OperationScheduler::new(0);
        scheduler.set_time(100);
        scheduler.schedule_at(Command::Increment, Priority::Normal, 200);

        assert_eq!(scheduler.queue_size(), 1);
    }

    #[test]
    fn test_scheduler_execute_pending() {
        let mut scheduler = OperationScheduler::new(0);
        scheduler.set_time(100);

        scheduler.schedule_at(Command::Increment, Priority::Normal, 50);
        scheduler.schedule_at(Command::Increment, Priority::Normal, 150);

        let results = scheduler.execute_pending();
        assert_eq!(results.len(), 1); // Only the one scheduled at time 50
        assert_eq!(scheduler.queue_size(), 1); // One still pending
    }

    #[test]
    fn test_scheduler_execute_batch() {
        let mut scheduler = OperationScheduler::new(0);

        scheduler.schedule(Command::Increment, Priority::Normal);
        scheduler.schedule(Command::Increment, Priority::Normal);
        scheduler.schedule(Command::Decrement, Priority::Normal);

        let results = scheduler.execute_batch(2);
        assert_eq!(results.len(), 2);
        assert_eq!(scheduler.queue_size(), 1);
    }

    #[test]
    fn test_scheduler_priority_queue() {
        let mut scheduler = OperationScheduler::new(0);
        scheduler.set_time(0);

        scheduler.schedule_at(Command::Increment, Priority::Low, 0);
        scheduler.schedule_at(Command::Decrement, Priority::High, 0);
        scheduler.schedule_at(Command::Get, Priority::Normal, 0);

        let results = scheduler.execute_batch(1);
        assert_eq!(results[0].command, Command::Decrement); // High priority first
    }

    #[test]
    fn test_scheduler_statistics() {
        let mut scheduler = OperationScheduler::new(0);

        scheduler.schedule(Command::Increment, Priority::Normal);
        scheduler.execute_batch(1);

        let (total, executed, success) = scheduler.statistics();
        assert_eq!(total, 1);
        assert_eq!(executed, 1);
        assert_eq!(success, 1);
    }

    #[test]
    fn test_scheduler_pending_at_priority() {
        let mut scheduler = OperationScheduler::new(0);

        scheduler.schedule(Command::Increment, Priority::High);
        scheduler.schedule(Command::Increment, Priority::High);
        scheduler.schedule(Command::Decrement, Priority::Low);

        let high_priority = scheduler.pending_at_priority(Priority::High);
        assert_eq!(high_priority.len(), 2);
    }

    #[test]
    fn test_scheduler_cancel_pending() {
        let mut scheduler = OperationScheduler::new(0);

        scheduler.schedule(Command::Increment, Priority::Normal);
        scheduler.schedule(Command::Increment, Priority::Normal);
        scheduler.schedule(Command::Decrement, Priority::Normal);

        let cancelled = scheduler.cancel_pending(Command::Increment);
        assert_eq!(cancelled, 2);
        assert_eq!(scheduler.queue_size(), 1);
    }

    #[test]
    fn test_scheduler_next_executable() {
        let mut scheduler = OperationScheduler::new(0);
        scheduler.set_time(100);

        scheduler.schedule_at(Command::Increment, Priority::Normal, 50);
        scheduler.schedule_at(Command::Decrement, Priority::Normal, 200);

        let next = scheduler.next_executable();
        assert!(next.is_some());
        assert_eq!(next.unwrap().command, Command::Increment);
    }

    #[test]
    fn test_scheduler_reset() {
        let mut scheduler = OperationScheduler::new(0);

        scheduler.schedule(Command::Increment, Priority::Normal);
        scheduler.execute_batch(1);

        scheduler.reset();

        assert_eq!(scheduler.queue_size(), 0);
        assert_eq!(scheduler.executed_count(), 0);
        assert_eq!(scheduler.current_value(), 0);
    }

    #[test]
    fn test_scheduler_complex_workflow() {
        let mut scheduler = OperationScheduler::new(5);

        // Schedule operations across time
        scheduler.set_time(0);
        scheduler.schedule_at(Command::Increment, Priority::High, 0);
        scheduler.schedule_at(Command::Increment, Priority::Low, 100);
        scheduler.schedule_at(Command::Decrement, Priority::Normal, 50);

        // Execute pending at time 0
        scheduler.execute_pending();
        assert_eq!(scheduler.current_value(), 6);

        // Move time and execute more
        scheduler.set_time(50);
        scheduler.execute_pending();
        assert_eq!(scheduler.current_value(), 5);

        // Move to end
        scheduler.set_time(100);
        scheduler.execute_pending();
        assert_eq!(scheduler.current_value(), 6);

        assert_eq!(scheduler.executed_count(), 3);
    }
}
