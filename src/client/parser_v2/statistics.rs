use std::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use serde::Serialize;

#[derive(Default)]
pub struct Statistics {
    current_running_threads: AtomicUsize,
    started_parsing_tasks: AtomicUsize,
    failed_parsing_tasks: AtomicUsize,
    other_errors: AtomicUsize,
    access_failed_parsing_tasks: AtomicUsize,
    successful_parsing_tasks: AtomicUsize,
    total_number_of_accounts: AtomicUsize,
    threads_waiting_for_refresh: AtomicUsize
}

#[derive(Serialize, Clone, Debug)]
pub struct StatisticsSnapshot {
    current_running_threads: usize,
    started_parsing_tasks: usize,
    failed_parsing_tasks: usize,
    other_errors: usize,
    access_failed_parsing_tasks: usize,
    successful_parsing_tasks: usize,
    total_number_of_accounts: usize,
    threads_waiting_for_refresh: usize
}

lazy_static! {
    pub static ref STATISTICS: Statistics = Statistics::default();
}

impl Statistics {

    pub fn increase_current_running_threads(&self) -> usize {
        return self.current_running_threads.fetch_add(1, Ordering::Relaxed) + 1;
    } 

    pub fn decrease_current_running_threads(&self) -> usize {
        return self.current_running_threads.fetch_sub(1, Ordering::Relaxed) - 1;
    }

    pub fn increase_started_parsing_tasks(&self) -> usize {
        return self.started_parsing_tasks.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn increase_failed_parsing_tasks(&self) -> usize {
        return self.failed_parsing_tasks.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn increase_other_errors(&self) -> usize {
        return self.other_errors.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn increase_access_failed_parsing_tasks(&self) -> usize {
        return self.access_failed_parsing_tasks.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn increase_successful_parsing_tasks(&self) -> usize {
        return self.successful_parsing_tasks.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn increase_total_number_of_accounts(&self) -> usize {
        return self.total_number_of_accounts.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn decrease_total_number_of_accounts(&self) -> usize {
        return self.total_number_of_accounts.fetch_sub(1, Ordering::Relaxed) - 1;
    }

    pub fn increase_threads_waiting_for_refresh(&self) -> usize {
        return self.threads_waiting_for_refresh.fetch_add(1, Ordering::Relaxed) + 1;
    }

    pub fn decrease_threads_waiting_for_refresh(&self) -> usize {
        return self.threads_waiting_for_refresh.fetch_sub(1, Ordering::Relaxed) - 1;
    }

    pub fn get_snapshot(&self) -> StatisticsSnapshot {
        return StatisticsSnapshot {
            current_running_threads: self.current_running_threads.load(Ordering::Relaxed),
            started_parsing_tasks: self.started_parsing_tasks.load(Ordering::Relaxed),
            failed_parsing_tasks: self.failed_parsing_tasks.load(Ordering::Relaxed),
            other_errors: self.other_errors.load(Ordering::Relaxed),
            access_failed_parsing_tasks: self.access_failed_parsing_tasks.load(Ordering::Relaxed),
            successful_parsing_tasks: self.successful_parsing_tasks.load(Ordering::Relaxed),
            total_number_of_accounts: self.total_number_of_accounts.load(Ordering::Relaxed),
            threads_waiting_for_refresh: self.threads_waiting_for_refresh.load(Ordering::Relaxed)
        }
    } 

}