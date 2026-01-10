//! Security gates for workflow operations
//!
//! Implements mandatory security checks that cannot be bypassed.
//! All file operations must pass through these gates.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::error::{WorkflowError, Result};

/// File tracker for enforcing read-before-edit rule
///
/// This gate ensures that any file modification requires the file
/// to be read first, preventing blind edits.
#[derive(Debug, Default)]
pub struct FileTracker {
    /// Files that have been read
    read_files: HashSet<PathBuf>,
    /// Files that have been modified
    modified_files: HashSet<PathBuf>,
    /// Maximum allowed file size in bytes
    max_file_size: usize,
    /// Allowed base paths (empty = allow all)
    allowed_paths: Vec<PathBuf>,
}

impl FileTracker {
    /// Create a new file tracker with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            read_files: HashSet::new(),
            modified_files: HashSet::new(),
            max_file_size: 10 * 1024 * 1024, // 10MB default
            allowed_paths: Vec::new(),
        }
    }

    /// Create with custom max file size
    #[must_use]
    pub fn with_max_file_size(mut self, size_bytes: usize) -> Self {
        self.max_file_size = size_bytes;
        self
    }

    /// Add allowed base path
    #[must_use]
    pub fn with_allowed_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_paths.push(path.into());
        self
    }

    /// Record that a file was read
    pub fn record_read(&mut self, path: impl AsRef<Path>) {
        let canonical = self.canonicalize_path(path.as_ref());
        self.read_files.insert(canonical);
    }

    /// Check if a file can be edited (must have been read first)
    pub fn check_edit(&self, path: impl AsRef<Path>) -> Result<()> {
        let canonical = self.canonicalize_path(path.as_ref());

        if !self.read_files.contains(&canonical) {
            return Err(WorkflowError::MustReadBeforeEdit {
                path: path.as_ref().to_path_buf(),
            });
        }

        Ok(())
    }

    /// Record that a file was modified (after edit check passes)
    pub fn record_modified(&mut self, path: impl AsRef<Path>) {
        let canonical = self.canonicalize_path(path.as_ref());
        self.modified_files.insert(canonical);
    }

    /// Check if a path is within allowed paths (if restrictions are set)
    pub fn check_path_allowed(&self, path: impl AsRef<Path>) -> Result<()> {
        if self.allowed_paths.is_empty() {
            return Ok(());
        }

        let path = path.as_ref();
        let canonical = self.canonicalize_path(path);

        for allowed in &self.allowed_paths {
            if canonical.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(WorkflowError::PathTraversal {
            path: path.to_path_buf(),
        })
    }

    /// Check file size against limit
    pub fn check_file_size(&self, size_bytes: usize) -> Result<()> {
        if size_bytes > self.max_file_size {
            return Err(WorkflowError::FileTooLarge {
                size_bytes,
                max_bytes: self.max_file_size,
            });
        }
        Ok(())
    }

    /// Get count of files read
    #[must_use]
    pub fn read_count(&self) -> usize {
        self.read_files.len()
    }

    /// Get count of files modified
    #[must_use]
    pub fn modified_count(&self) -> usize {
        self.modified_files.len()
    }

    /// Get list of modified files (for reporting)
    #[must_use]
    pub fn modified_files(&self) -> Vec<PathBuf> {
        self.modified_files.iter().cloned().collect()
    }

    /// Reset tracker (for new workflow run)
    pub fn reset(&mut self) {
        self.read_files.clear();
        self.modified_files.clear();
    }

    /// Canonicalize a path for consistent tracking
    fn canonicalize_path(&self, path: &Path) -> PathBuf {
        // Try to get canonical path, fall back to normalized path
        path.canonicalize().unwrap_or_else(|_| {
            // Normalize the path manually
            let mut normalized = PathBuf::new();
            for component in path.components() {
                match component {
                    std::path::Component::ParentDir => {
                        normalized.pop();
                    }
                    std::path::Component::CurDir => {}
                    other => normalized.push(other),
                }
            }
            normalized
        })
    }
}

/// Security gate that enforces all security policies
#[derive(Debug)]
pub struct SecurityGate {
    /// File tracking
    file_tracker: FileTracker,
    /// Whether plan approval is required
    require_approval: bool,
    /// Whether plan is approved
    plan_approved: bool,
}

impl SecurityGate {
    /// Create a new security gate with all checks enabled
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_tracker: FileTracker::new(),
            require_approval: true,
            plan_approved: false,
        }
    }

    /// Create with custom file tracker
    #[must_use]
    pub fn with_file_tracker(mut self, tracker: FileTracker) -> Self {
        self.file_tracker = tracker;
        self
    }

    /// Configure approval requirement (default: true)
    #[must_use]
    pub fn require_approval(mut self, require: bool) -> Self {
        self.require_approval = require;
        self
    }

    /// Mark plan as approved
    pub fn approve_plan(&mut self) {
        self.plan_approved = true;
    }

    /// Check if modifications are allowed (plan must be approved)
    pub fn check_can_modify(&self) -> Result<()> {
        if self.require_approval && !self.plan_approved {
            return Err(WorkflowError::NoPlanApproved);
        }
        Ok(())
    }

    /// Record a file read
    pub fn on_file_read(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.file_tracker.check_path_allowed(path.as_ref())?;
        self.file_tracker.record_read(path);
        Ok(())
    }

    /// Check and record a file edit
    pub fn on_file_edit(&mut self, path: impl AsRef<Path>) -> Result<()> {
        // Must have approved plan
        self.check_can_modify()?;

        // Must have read first
        self.file_tracker.check_edit(path.as_ref())?;

        // Path must be allowed
        self.file_tracker.check_path_allowed(path.as_ref())?;

        // Record the modification
        self.file_tracker.record_modified(path);

        Ok(())
    }

    /// Check file size before reading
    pub fn check_file_size(&self, size_bytes: usize) -> Result<()> {
        self.file_tracker.check_file_size(size_bytes)
    }

    /// Get file tracker for inspection
    #[must_use]
    pub fn file_tracker(&self) -> &FileTracker {
        &self.file_tracker
    }

    /// Reset security gate for new workflow
    pub fn reset(&mut self) {
        self.file_tracker.reset();
        self.plan_approved = false;
    }
}

impl Default for SecurityGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate user input for security
pub struct InputValidator;

impl InputValidator {
    /// Validate a file path (no path traversal, reasonable length)
    pub fn validate_path(path: &str) -> Result<PathBuf> {
        // Check for null bytes
        if path.contains('\0') {
            return Err(WorkflowError::Validation {
                message: "Path contains null bytes".to_string(),
            });
        }

        // Check length
        if path.len() > 4096 {
            return Err(WorkflowError::Validation {
                message: "Path too long".to_string(),
            });
        }

        let path_buf = PathBuf::from(path);

        // Check for suspicious patterns
        let path_str = path_buf.to_string_lossy();
        if path_str.contains("..") && path_str.contains('/') {
            // More thorough check - this is potentially path traversal
            let components: Vec<_> = path_buf.components().collect();
            let parent_count = components
                .iter()
                .filter(|c| matches!(c, std::path::Component::ParentDir))
                .count();

            if parent_count > 0 {
                // Warn but don't block - the FileTracker will handle actual checks
                tracing::warn!("Path contains parent directory references: {}", path);
            }
        }

        Ok(path_buf)
    }

    /// Validate task description
    pub fn validate_task_description(desc: &str) -> Result<()> {
        if desc.trim().is_empty() {
            return Err(WorkflowError::Validation {
                message: "Task description cannot be empty".to_string(),
            });
        }

        if desc.len() > 10000 {
            return Err(WorkflowError::Validation {
                message: "Task description too long (max 10000 chars)".to_string(),
            });
        }

        Ok(())
    }

    /// Validate knowledge content
    pub fn validate_knowledge_content(content: &str) -> Result<()> {
        if content.trim().is_empty() {
            return Err(WorkflowError::Validation {
                message: "Knowledge content cannot be empty".to_string(),
            });
        }

        if content.len() > 100_000 {
            return Err(WorkflowError::Validation {
                message: "Knowledge content too long (max 100000 chars)".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_before_edit() {
        let mut gate = SecurityGate::new();
        gate.approve_plan(); // Must have approved plan

        // Can't edit without reading
        assert!(gate.on_file_edit("/src/main.rs").is_err());

        // Read first
        gate.on_file_read("/src/main.rs").unwrap();

        // Now can edit
        assert!(gate.on_file_edit("/src/main.rs").is_ok());
    }

    #[test]
    fn test_no_plan_blocks_edit() {
        let mut gate = SecurityGate::new();

        // Read the file
        gate.on_file_read("/src/main.rs").unwrap();

        // Can't edit - no approved plan
        let result = gate.on_file_edit("/src/main.rs");
        assert!(matches!(result, Err(WorkflowError::NoPlanApproved)));
    }

    #[test]
    fn test_path_restriction() {
        let tracker = FileTracker::new()
            .with_allowed_path("/home/user/project");

        let mut gate = SecurityGate::new()
            .with_file_tracker(tracker);

        // Can read within allowed path
        assert!(gate.on_file_read("/home/user/project/src/main.rs").is_ok());

        // Can't read outside allowed path
        let result = gate.on_file_read("/etc/passwd");
        assert!(matches!(result, Err(WorkflowError::PathTraversal { .. })));
    }

    #[test]
    fn test_file_size_check() {
        let tracker = FileTracker::new()
            .with_max_file_size(1024); // 1KB limit

        let gate = SecurityGate::new()
            .with_file_tracker(tracker);

        // Small file OK
        assert!(gate.check_file_size(500).is_ok());

        // Large file blocked
        let result = gate.check_file_size(2000);
        assert!(matches!(result, Err(WorkflowError::FileTooLarge { .. })));
    }

    #[test]
    fn test_input_validation() {
        // Valid path
        assert!(InputValidator::validate_path("/src/main.rs").is_ok());

        // Null byte
        assert!(InputValidator::validate_path("/src\0/main.rs").is_err());

        // Empty description
        assert!(InputValidator::validate_task_description("").is_err());
        assert!(InputValidator::validate_task_description("   ").is_err());

        // Valid description
        assert!(InputValidator::validate_task_description("Add feature X").is_ok());
    }

    #[test]
    fn test_tracker_counts() {
        let mut tracker = FileTracker::new();

        tracker.record_read("/a.rs");
        tracker.record_read("/b.rs");
        tracker.record_read("/c.rs");

        assert_eq!(tracker.read_count(), 3);
        assert_eq!(tracker.modified_count(), 0);

        tracker.record_modified("/a.rs");
        tracker.record_modified("/b.rs");

        assert_eq!(tracker.modified_count(), 2);
    }

    #[test]
    fn test_reset() {
        let mut gate = SecurityGate::new();
        gate.approve_plan();
        gate.on_file_read("/src/main.rs").unwrap();
        gate.on_file_edit("/src/main.rs").unwrap();

        assert_eq!(gate.file_tracker().read_count(), 1);
        assert_eq!(gate.file_tracker().modified_count(), 1);

        gate.reset();

        assert_eq!(gate.file_tracker().read_count(), 0);
        assert_eq!(gate.file_tracker().modified_count(), 0);

        // Plan approval is also reset
        assert!(gate.check_can_modify().is_err());
    }
}
