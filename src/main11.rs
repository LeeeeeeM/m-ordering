use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    test_spinlock();
}

// 基于内存序的自旋锁
pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
    
    // 获取锁 - 使用 Acquire 排序
    pub fn lock(&self) {
        loop {
            // 尝试获取锁
            if self.locked.compare_exchange_weak(
                false,  // 期望值：未锁定
                true,   // 新值：锁定
                Ordering::Acquire,  // 成功时：Acquire 排序
                Ordering::Relaxed   // 失败时：Relaxed 排序
            ).is_ok() {
                // 成功获取锁，退出
                break;
            }
            
            // 获取锁失败，自旋等待锁被释放
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
            // 锁被释放了，重新尝试获取
        }
    }
    
    // 释放锁 - 使用 Release 排序
    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
    
    // 尝试获取锁
    pub fn try_lock(&self) -> bool {
        self.locked.compare_exchange_weak(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_ok()
    }
}

// 测试基本的锁功能
fn test_spinlock() {
    println!("=== 自旋锁基本功能测试 ===");
    
    let lock = Arc::new(SpinLock::new());
    let counter = Arc::new(AtomicU32::new(0));
    let data = Arc::new(Mutex::new(Vec::new()));
    
    thread::scope(|s| {
        for i in 0..5 {
            let lock = lock.clone();
            let counter = counter.clone();
            let data = data.clone();
            
            s.spawn(move || {
                for j in 0..100 {
                    lock.lock();
                    {
                        // 复杂的临界区操作：需要锁保护
                        let current = counter.load(Ordering::Relaxed);
                        let new_value = current + 1;
                        counter.store(new_value, Ordering::Relaxed);
                        
                        // 模拟复杂的业务逻辑
                        let mut data_vec = data.lock().unwrap();
                        data_vec.push(format!("线程{}第{}次操作", i, j));
                        
                        println!("线程 {} 获取锁，计数器: {}, 数据长度: {}", i, new_value, data_vec.len());
                    }
                    lock.unlock();
                    
                    // 模拟一些工作
                    thread::sleep(Duration::from_millis(1));
                }
            });
        }
    });
    
    let final_count = counter.load(Ordering::Relaxed);
    let final_data_len = data.lock().unwrap().len();
    println!("最终计数器值: {}", final_count);
    println!("最终数据长度: {}", final_data_len);
    println!("预期值: 500 (5线程 × 100次)");
    
    if final_count == 500 && final_data_len == 500 {
        println!("✅ 自旋锁功能正常");
    } else {
        println!("❌ 自旋锁功能异常");
    }
    println!();
}
