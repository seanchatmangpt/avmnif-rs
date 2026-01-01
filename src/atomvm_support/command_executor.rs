// Working command execution system for high-level operation control
// Executes commands with error handling, result tracking, and context management

use alloc::vec::Vec;
use crate::atomvm_support::{
    message_dispatch::MessageOp,
    integration::CounterPortSystem,
};

/// Command types with parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Increment,
    Decrement,
    Get,
    Reset,
}

impl Command {
    /// Convert to MessageOp for system processing
    pub fn to_operation(&self) -> MessageOp {
        match self {
            Command::Increment => MessageOp::Inc,
            Command::Decrement => MessageOp::Dec,
            Command::Get => MessageOp::Get,
            Command::Reset => MessageOp::Reset,
        }
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: Command,
    pub success: bool,
    pub value: i32,
    pub error_message: Option<&'static str>,
}

impl CommandResult {
    pub fn ok(command: Command, value: i32) -> Self {
        CommandResult {
            command,
            success: true,
            value,
            error_message: None,
        }
    }

    pub fn error(command: Command, message: &'static str) -> Self {
        CommandResult {
            command,
            success: false,
            value: 0,
            error_message: Some(message),
        }
    }
}

/// Command execution history entry
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub result: CommandResult,
    pub sequence: u32,
}

/// High-level command executor
pub struct CommandExecutor {
    system: CounterPortSystem,
    history: Vec<CommandEntry>,
    sequence: u32,
    success_count: u32,
    error_count: u32,
}

impl CommandExecutor {
    pub fn new(initial_value: i32) -> Self {
        CommandExecutor {
            system: CounterPortSystem::new(initial_value),
            history: Vec::new(),
            sequence: 0,
            success_count: 0,
            error_count: 0,
        }
    }

    /// Execute a single command
    pub fn execute(&mut self, cmd: Command) -> CommandResult {
        self.sequence += 1;

        let result = match self.system.process_operation(cmd.to_operation()) {
            Ok(value) => CommandResult::ok(cmd, value),
            Err(err) => CommandResult::error(cmd, err),
        };

        if result.success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }

        let entry = CommandEntry {
            result: result.clone(),
            sequence: self.sequence,
        };

        self.history.push(entry);
        result
    }

    /// Execute multiple commands
    pub fn execute_batch(&mut self, commands: &[Command]) -> Vec<CommandResult> {
        commands
            .iter()
            .map(|&cmd| self.execute(cmd))
            .collect()
    }

    /// Get current system state
    pub fn current_value(&self) -> i32 {
        self.system.get_value()
    }

    /// Get execution statistics
    pub fn statistics(&self) -> (u32, u32, u32) {
        (self.sequence, self.success_count, self.error_count)
    }

    /// Get command history
    pub fn history(&self) -> &[CommandEntry] {
        &self.history
    }

    /// Clear history but keep system state
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.sequence = 0;
        self.success_count = 0;
        self.error_count = 0;
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> u32 {
        if self.sequence == 0 {
            0
        } else {
            (self.success_count as u32 * 100) / self.sequence
        }
    }

    /// Find commands by type in history
    pub fn commands_of_type(&self, cmd_type: Command) -> Vec<&CommandEntry> {
        self.history
            .iter()
            .filter(|e| e.result.command == cmd_type)
            .collect()
    }

    /// Get last N commands
    pub fn last_commands(&self, count: usize) -> Vec<&CommandEntry> {
        self.history
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Reset executor and system
    pub fn reset(&mut self) {
        self.system.reset();
        self.history.clear();
        self.sequence = 0;
        self.success_count = 0;
        self.error_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_to_operation() {
        assert_eq!(Command::Increment.to_operation(), MessageOp::Inc);
        assert_eq!(Command::Decrement.to_operation(), MessageOp::Dec);
        assert_eq!(Command::Get.to_operation(), MessageOp::Get);
        assert_eq!(Command::Reset.to_operation(), MessageOp::Reset);
    }

    #[test]
    fn test_command_result_ok() {
        let result = CommandResult::ok(Command::Increment, 42);
        assert!(result.success);
        assert_eq!(result.value, 42);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_command_result_error() {
        let result = CommandResult::error(Command::Increment, "test error");
        assert!(!result.success);
        assert_eq!(result.error_message, Some("test error"));
    }

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new(0);
        assert_eq!(executor.current_value(), 0);
        assert_eq!(executor.sequence, 0);
    }

    #[test]
    fn test_executor_single_command() {
        let mut executor = CommandExecutor::new(0);
        let result = executor.execute(Command::Increment);

        assert!(result.success);
        assert_eq!(result.value, 1);
        assert_eq!(executor.current_value(), 1);
        assert_eq!(executor.sequence, 1);
        assert_eq!(executor.success_count, 1);
    }

    #[test]
    fn test_executor_multiple_commands() {
        let mut executor = CommandExecutor::new(0);

        executor.execute(Command::Increment);
        executor.execute(Command::Increment);
        executor.execute(Command::Decrement);

        assert_eq!(executor.current_value(), 1);
        assert_eq!(executor.sequence, 3);
        assert_eq!(executor.success_count, 3);
    }

    #[test]
    fn test_executor_batch() {
        let mut executor = CommandExecutor::new(5);
        let commands = [
            Command::Increment,
            Command::Increment,
            Command::Decrement,
        ];

        let results = executor.execute_batch(&commands);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success));
        // 5 + 1 + 1 - 1 = 6
        assert_eq!(executor.current_value(), 6);
    }

    #[test]
    fn test_executor_statistics() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Increment);

        let (seq, success, error) = executor.statistics();
        assert_eq!(seq, 2);
        assert_eq!(success, 2);
        assert_eq!(error, 0);
    }

    #[test]
    fn test_executor_success_rate() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Increment);
        executor.execute(Command::Increment);

        assert_eq!(executor.success_rate(), 100);
    }

    #[test]
    fn test_executor_history() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Get);
        executor.execute(Command::Decrement);

        assert_eq!(executor.history().len(), 3);
        assert_eq!(executor.history()[0].result.command, Command::Increment);
    }

    #[test]
    fn test_executor_commands_of_type() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Decrement);
        executor.execute(Command::Increment);

        let increments = executor.commands_of_type(Command::Increment);
        assert_eq!(increments.len(), 2);
    }

    #[test]
    fn test_executor_last_commands() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Decrement);
        executor.execute(Command::Get);
        executor.execute(Command::Reset);

        let last_two = executor.last_commands(2);
        assert_eq!(last_two.len(), 2);
        assert_eq!(last_two[0].result.command, Command::Get);
        assert_eq!(last_two[1].result.command, Command::Reset);
    }

    #[test]
    fn test_executor_clear_history() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Increment);

        assert_eq!(executor.history().len(), 2);
        executor.clear_history();
        assert_eq!(executor.history().len(), 0);
        assert_eq!(executor.sequence, 0);
    }

    #[test]
    fn test_executor_reset() {
        let mut executor = CommandExecutor::new(0);
        executor.execute(Command::Increment);
        executor.execute(Command::Increment);

        assert_eq!(executor.current_value(), 2);
        executor.reset();
        assert_eq!(executor.current_value(), 0);
        assert_eq!(executor.history().len(), 0);
    }
}
