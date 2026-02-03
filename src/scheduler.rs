use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
use uuid::Uuid;

use crate::notify::{Notifier, NotifyRequest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRequest {
    #[serde(flatten)]
    pub notify: NotifyRequest,
    // Cron expression: "0 */5 * * * *" (every 5 minutes)
    // or "0 0 9 * * *" (every day at 9:00)
    pub cron: String,
    // 最大执行次数，达到后自动删除。不设置或0表示无限次
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTimeRequest {
    #[serde(flatten)]
    pub notify: NotifyRequest,
    // ISO 8601 format: "2024-01-15T09:00:00Z"
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub cron: Option<String>,
    pub at: Option<DateTime<Utc>>,
    pub notify: NotifyRequest,
    pub created_at: DateTime<Utc>,
    // 最大执行次数，None 表示无限次
    pub max_count: Option<u32>,
    // 用于序列化时隐藏，不暴露给客户端
    #[serde(skip)]
    pub cancelled: Arc<AtomicBool>,
}

pub struct NotificationScheduler {
    scheduler: JobScheduler,
    notifier: Arc<Notifier>,
    jobs: Arc<RwLock<HashMap<String, ScheduledJob>>>,
}

impl NotificationScheduler {
    pub async fn new(notifier: Arc<Notifier>) -> anyhow::Result<Self> {
        let scheduler = JobScheduler::new().await?;
        
        Ok(Self {
            scheduler,
            notifier,
            jobs: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.scheduler.start().await?;
        info!("Scheduler started");
        Ok(())
    }

    pub async fn add_cron_job(&self, req: ScheduleRequest) -> anyhow::Result<String> {
        let job_id = Uuid::new_v4().to_string();
        let notifier = Arc::clone(&self.notifier);
        let notify_req = req.notify.clone();
        let job_id_clone = job_id.clone();
        let jobs = Arc::clone(&self.jobs);
        let max_count = req.max_count;

        // Validate cron expression first
        let cron_str = req.cron.clone();
        let cron_parts: Vec<&str> = cron_str.split_whitespace().collect();
        if cron_parts.len() != 6 {
            return Err(anyhow::anyhow!(
                "Invalid cron expression: expected 6 parts (sec min hour day month day_of_week), got {}",
                cron_parts.len()
            ));
        }

        // 创建取消标志和计数器
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = Arc::clone(&cancelled);
        let current_count = Arc::new(AtomicU32::new(0));

        let job = Job::new_async(cron_str.as_str(), move |_uuid, _l| {
            let notifier = Arc::clone(&notifier);
            let notify_req = notify_req.clone();
            let job_id = job_id_clone.clone();
            let jobs = Arc::clone(&jobs);
            let cancelled = Arc::clone(&cancelled_clone);
            let current_count = Arc::clone(&current_count);
            let max_count = max_count;
            
            Box::pin(async move {
                // 检查是否已被取消
                if cancelled.load(Ordering::SeqCst) {
                    info!("Job {} is cancelled, skipping execution", job_id);
                    return;
                }
                
                // 增加计数
                let count = current_count.fetch_add(1, Ordering::SeqCst) + 1;
                info!("Executing scheduled job {} (count: {})", job_id, count);
                
                // 发送通知
                match notifier.send(&notify_req).await {
                    Ok(_) => info!("Job {} executed successfully (count: {})", job_id, count),
                    Err(e) => error!("Failed to send scheduled notification for job {}: {}", job_id, e),
                }
                
                // 检查是否达到最大次数
                if let Some(max) = max_count {
                    if count >= max {
                        info!("Job {} reached max count ({}), removing", job_id, max);
                        cancelled.store(true, Ordering::SeqCst);
                        jobs.write().await.remove(&job_id);
                        return;
                    }
                }
            })
        })?;

        self.scheduler.add(job).await?;

        let jobs = Arc::clone(&self.jobs);
        let scheduled_job = ScheduledJob {
            id: job_id.clone(),
            cron: Some(req.cron),
            at: None,
            notify: req.notify,
            created_at: Utc::now(),
            max_count,
            cancelled,
        };

        jobs.write().await.insert(job_id.clone(), scheduled_job);
        info!("Added cron job {}, max_count: {:?}", job_id, max_count);

        Ok(job_id)
    }

    pub async fn add_one_time_job(&self, req: OneTimeRequest) -> anyhow::Result<String> {
        let now = Utc::now();
        
        if req.at <= now {
            return Err(anyhow::anyhow!(
                "Scheduled time must be in the future. Now: {}, Scheduled: {}",
                now, req.at
            ));
        }

        let job_id = Uuid::new_v4().to_string();
        let notifier = Arc::clone(&self.notifier);
        let notify_req = req.notify.clone();
        let job_id_clone = job_id.clone();
        let jobs = Arc::clone(&self.jobs);

        // 创建取消标志
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_clone = Arc::clone(&cancelled);

        // 计算延迟时间
        let duration = req.at.signed_duration_since(now);
        let seconds = duration.num_seconds().max(0) as u64;
        
        info!("Scheduling one-time job {} to run in {} seconds at {}", job_id, seconds, req.at);

        // 创建定时任务：使用标准库的延迟执行，更可靠
        let job = Job::new_one_shot_async(
            std::time::Duration::from_secs(seconds),
            move |_uuid, _l| {
                let notifier = Arc::clone(&notifier);
                let notify_req = notify_req.clone();
                let job_id = job_id_clone.clone();
                let jobs = Arc::clone(&jobs);
                let cancelled = Arc::clone(&cancelled_clone);
                
                Box::pin(async move {
                    // 检查是否已被取消
                    if cancelled.load(Ordering::SeqCst) {
                        info!("One-time job {} is cancelled, skipping execution", job_id);
                        jobs.write().await.remove(&job_id);
                        return;
                    }
                    
                    info!("Executing one-time job {}", job_id);
                    match notifier.send(&notify_req).await {
                        Ok(_) => info!("One-time job {} executed successfully", job_id),
                        Err(e) => error!("Failed to send one-time notification for job {}: {}", job_id, e),
                    }
                    // 执行完成后从列表中移除
                    jobs.write().await.remove(&job_id);
                    info!("One-time job {} completed and removed", job_id);
                })
            }
        )?;

        self.scheduler.add(job).await?;

        let jobs = Arc::clone(&self.jobs);
        let scheduled_job = ScheduledJob {
            id: job_id.clone(),
            cron: None,
            at: Some(req.at),
            notify: req.notify,
            created_at: Utc::now(),
            max_count: Some(1), // 一次性任务默认执行1次
            cancelled,
        };

        jobs.write().await.insert(job_id.clone(), scheduled_job);
        info!("Added one-time job {} at {} (in {} seconds)", job_id, req.at, seconds);

        Ok(job_id)
    }

    pub async fn remove_job(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.write().await;
        
        if let Some(job) = jobs.get(job_id) {
            // 设置取消标志，下次执行时会跳过
            job.cancelled.store(true, Ordering::SeqCst);
            jobs.remove(job_id);
            info!("Job {} marked as cancelled and removed", job_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Job {} not found", job_id))
        }
    }

    pub async fn list_jobs(&self) -> Vec<ScheduledJob> {
        self.jobs.read().await.values().cloned().collect()
    }

    pub async fn get_job(&self, job_id: &str) -> Option<ScheduledJob> {
        self.jobs.read().await.get(job_id).cloned()
    }
}
