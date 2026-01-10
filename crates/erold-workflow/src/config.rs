//! Workflow configuration
//!
//! Immutable configuration with secure defaults.

use std::time::Duration;

/// Workflow configuration
///
/// All security features are enabled by default.
/// Use `WorkflowConfigBuilder` for customization.
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    // Security settings (all true by default)
    require_plan: bool,
    require_approval: bool,
    require_read_before_edit: bool,
    auto_enrich: bool,

    // Timeouts
    approval_timeout: Duration,
    approval_poll_interval: Duration,
    api_timeout: Duration,

    // Limits
    max_subtasks: usize,
    max_knowledge_results: usize,
    max_file_size_bytes: usize,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            // Security: ALL ON by default
            require_plan: true,
            require_approval: true,
            require_read_before_edit: true,
            auto_enrich: true,

            // Reasonable timeouts
            approval_timeout: Duration::from_secs(300),      // 5 minutes
            approval_poll_interval: Duration::from_secs(5),  // 5 seconds
            api_timeout: Duration::from_secs(30),            // 30 seconds

            // Sensible limits
            max_subtasks: 20,
            max_knowledge_results: 50,
            max_file_size_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl WorkflowConfig {
    /// Create a new builder
    #[must_use]
    pub fn builder() -> WorkflowConfigBuilder {
        WorkflowConfigBuilder::default()
    }

    // Getters (immutable access only)

    #[must_use]
    pub fn require_plan(&self) -> bool {
        self.require_plan
    }

    #[must_use]
    pub fn require_approval(&self) -> bool {
        self.require_approval
    }

    #[must_use]
    pub fn require_read_before_edit(&self) -> bool {
        self.require_read_before_edit
    }

    #[must_use]
    pub fn auto_enrich(&self) -> bool {
        self.auto_enrich
    }

    #[must_use]
    pub fn approval_timeout(&self) -> Duration {
        self.approval_timeout
    }

    #[must_use]
    pub fn approval_poll_interval(&self) -> Duration {
        self.approval_poll_interval
    }

    #[must_use]
    pub fn api_timeout(&self) -> Duration {
        self.api_timeout
    }

    #[must_use]
    pub fn max_subtasks(&self) -> usize {
        self.max_subtasks
    }

    #[must_use]
    pub fn max_knowledge_results(&self) -> usize {
        self.max_knowledge_results
    }

    #[must_use]
    pub fn max_file_size_bytes(&self) -> usize {
        self.max_file_size_bytes
    }
}

/// Builder for `WorkflowConfig`
///
/// Allows customization while maintaining secure defaults.
#[derive(Debug, Default)]
pub struct WorkflowConfigBuilder {
    config: WorkflowConfig,
}

impl WorkflowConfigBuilder {
    /// Set approval timeout
    #[must_use]
    pub fn approval_timeout(mut self, timeout: Duration) -> Self {
        self.config.approval_timeout = timeout;
        self
    }

    /// Set approval poll interval
    #[must_use]
    pub fn approval_poll_interval(mut self, interval: Duration) -> Self {
        self.config.approval_poll_interval = interval;
        self
    }

    /// Set API timeout
    #[must_use]
    pub fn api_timeout(mut self, timeout: Duration) -> Self {
        self.config.api_timeout = timeout;
        self
    }

    /// Set maximum subtasks per plan
    #[must_use]
    pub fn max_subtasks(mut self, max: usize) -> Self {
        self.config.max_subtasks = max;
        self
    }

    /// Set maximum knowledge results
    #[must_use]
    pub fn max_knowledge_results(mut self, max: usize) -> Self {
        self.config.max_knowledge_results = max;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> WorkflowConfig {
        self.config
    }

    // NOTE: Security settings cannot be disabled through the builder.
    // This is intentional - security features should always be on.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_secure() {
        let config = WorkflowConfig::default();

        assert!(config.require_plan());
        assert!(config.require_approval());
        assert!(config.require_read_before_edit());
        assert!(config.auto_enrich());
    }

    #[test]
    fn test_builder_maintains_security() {
        let config = WorkflowConfig::builder()
            .approval_timeout(Duration::from_secs(600))
            .build();

        // Security settings still on
        assert!(config.require_plan());
        assert!(config.require_approval());
        assert!(config.require_read_before_edit());

        // Custom value applied
        assert_eq!(config.approval_timeout(), Duration::from_secs(600));
    }
}
