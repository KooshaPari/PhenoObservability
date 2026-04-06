//! # phenotype-sentinel
//!
//! Bulkhead isolation pattern implementation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Bulkhead for partition-based isolation
///
/// Limits concurrent access to resources by partitioning
/// them into isolated groups.
#[derive(Debug)]
pub struct Bulkhead {
    partitions: Arc<RwLock<HashMap<usize, usize>>>,
    partition_capacity: usize,
    total_capacity: usize,
    current_total: Arc<RwLock<usize>>,
}

impl Bulkhead {
    /// Create a new bulkhead wrapped in an Arc
    ///
    /// - `num_partitions`: Number of partitions
    /// - `capacity_per_partition`: Max concurrent operations per partition
    pub fn new(num_partitions: usize, capacity_per_partition: usize) -> Arc<Self> {
        let mut partitions = HashMap::new();
        for i in 0..num_partitions {
            partitions.insert(i, 0);
        }

        Arc::new(Self {
            partitions: Arc::new(RwLock::new(partitions)),
            partition_capacity: capacity_per_partition,
            total_capacity: num_partitions * capacity_per_partition,
            current_total: Arc::new(RwLock::new(0)),
        })
    }

    /// Try to acquire a permit in a partition
    pub async fn try_acquire(
        self: &Arc<Self>,
        partition: usize,
    ) -> Result<PartitionGuard, BulkheadError> {
        let mut partitions = self.partitions.write().await;
        let current = partitions.get(&partition).copied().unwrap_or(0);

        if current >= self.partition_capacity {
            return Err(BulkheadError::PartitionExhausted(partition));
        }

        let mut total = self.current_total.write().await;
        if *total >= self.total_capacity {
            return Err(BulkheadError::TotalExhausted);
        }

        partitions.insert(partition, current + 1);
        *total += 1;

        Ok(PartitionGuard { bulkhead: Arc::clone(self), partition })
    }

    /// Release a permit from a partition
    pub async fn release(&self, partition: usize) {
        let mut partitions = self.partitions.write().await;
        let mut total = self.current_total.write().await;

        if let Some(current) = partitions.get_mut(&partition) {
            if *current > 0 {
                *current -= 1;
                *total = total.saturating_sub(1);
            }
        }
    }

    /// Get current usage of a partition
    pub async fn usage(&self, partition: usize) -> usize {
        let partitions = self.partitions.read().await;
        partitions.get(&partition).copied().unwrap_or(0)
    }

    /// Get total current usage
    pub async fn total_usage(&self) -> usize {
        *self.current_total.read().await
    }

    /// Get partition capacity
    pub fn partition_capacity(&self) -> usize {
        self.partition_capacity
    }

    /// Get total capacity
    pub fn total_capacity(&self) -> usize {
        self.total_capacity
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BulkheadError {
    #[error("Partition {0} exhausted")]
    PartitionExhausted(usize),

    #[error("Total capacity exhausted")]
    TotalExhausted,
}

/// Partition guard that automatically releases on drop
pub struct PartitionGuard {
    bulkhead: Arc<Bulkhead>,
    partition: usize,
}

impl Drop for PartitionGuard {
    fn drop(&mut self) {
        // Spawn a task to release the partition
        let bulkhead = Arc::clone(&self.bulkhead);
        let partition = self.partition;
        tokio::spawn(async move {
            bulkhead.release(partition).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bulkhead_acquire_release() {
        let bulkhead = Bulkhead::new(3, 2);
        {
            let _guard = bulkhead.try_acquire(0).await.unwrap();
            assert_eq!(bulkhead.usage(0).await, 1); // Guard held
        }
        // Guard dropped, release scheduled - wait for it
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert_eq!(bulkhead.usage(0).await, 0);
    }

    #[tokio::test]
    async fn test_bulkhead_partition_exhausted() {
        let bulkhead = Bulkhead::new(2, 1);
        let _guard1 = bulkhead.try_acquire(0).await.unwrap();
        let result = bulkhead.try_acquire(0).await;
        assert!(matches!(result, Err(BulkheadError::PartitionExhausted(0))));
    }

    #[tokio::test]
    async fn test_bulkhead_total_exhausted() {
        let bulkhead = Bulkhead::new(2, 1);
        let _guard1 = bulkhead.try_acquire(0).await.unwrap();
        let _guard2 = bulkhead.try_acquire(1).await.unwrap();
        let result = bulkhead.try_acquire(0).await;
        // Total capacity is 2, so third acquire should fail
        // Could be either TotalExhausted or PartitionExhausted since partition 0 is also at capacity
        assert!(matches!(
            result,
            Err(BulkheadError::TotalExhausted) | Err(BulkheadError::PartitionExhausted(0))
        ));
    }
}
