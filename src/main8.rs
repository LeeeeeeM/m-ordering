use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

fn main() {
    println!("=== AcqRel 排序示例 ===");
    
    // 示例1: 简单的计数器
    test_counter_example();

    // for _ in 0..10 {
    //     test_counter_example();
    // }
    
    
    // 示例2: 版本号方案
    // test_versioned_example();
    
    // 示例3: 多线程竞争
    // test_competitive_example();
}

// 示例1: 简单的计数器
fn test_counter_example() {
    println!("\n--- 示例1: 简单计数器 ---");
    
    let counter = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1: 增加计数器
        s.spawn(|| {
            for _ in 0..50 {
                let current = counter.load(Ordering::Relaxed);
                let new_value = current + 1;
                
                // 使用 AcqRel 的 CAS 操作
                match counter.compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => println!("线程1: 成功增加计数器到 {}", new_value),
                    Err(actual) => println!("线程1: CAS 失败，期望 {}, 实际 {}", current, actual),
                }
                
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        
        // 线程2: 增加计数器
        s.spawn(|| {
            for _ in 0..50 {
                let current = counter.load(Ordering::Relaxed);
                let new_value = current + 1;
                
                // 使用 AcqRel 的 CAS 操作
                match counter.compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => println!("线程2: 成功增加计数器到 {}", new_value),
                    Err(actual) => println!("线程2: CAS 失败，期望 {}, 实际 {}", current, actual),
                }
                
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    });
    
    println!("最终计数器值: {}", counter.load(Ordering::Relaxed));
}

// 示例2: 版本号方案
fn test_versioned_example() {
    println!("\n--- 示例2: 版本号方案 ---");
    
    let data = AtomicU32::new(0);
    let version = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1: 更新数据和版本
        s.spawn(|| {
            for _i in 1..=3 {
                let current_data = data.load(Ordering::Relaxed);
                let current_version = version.load(Ordering::Relaxed);
                let new_data = current_data + 100;
                let new_version = current_version + 1;
                
                // 使用 AcqRel 的 CAS 操作更新数据
                match data.compare_exchange(current_data, new_data, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => {
                        // 数据更新成功，更新版本号
                        version.store(new_version, Ordering::Release);
                        println!("线程1: 更新数据 {} -> {}, 版本 {} -> {}", 
                                current_data, new_data, current_version, new_version);
                    }
                    Err(actual) => {
                        println!("线程1: 数据更新失败，期望 {}, 实际 {}", current_data, actual);
                    }
                }
                
                thread::sleep(std::time::Duration::from_millis(50));
            }
        });
        
        // 线程2: 读取数据和版本
        s.spawn(|| {
            for _ in 0..3 {
                let current_data = data.load(Ordering::Acquire);
                let current_version = version.load(Ordering::Acquire);
                
                println!("线程2: 读取到数据 {}, 版本 {}", current_data, current_version);
                
                thread::sleep(std::time::Duration::from_millis(30));
            }
        });
    });
}

// 示例3: 多线程竞争
fn test_competitive_example() {
    println!("\n--- 示例3: 多线程竞争 ---");
    
    let shared_value = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1
        s.spawn(|| {
            for _ in 0..3 {
                let current = shared_value.load(Ordering::Relaxed);
                let new_value = current + 1;
                
                // 使用 AcqRel 的 CAS 操作
                match shared_value.compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => {
                        println!("线程1: 成功更新 {} -> {}", current, new_value);
                    }
                    Err(actual) => {
                        println!("线程1: CAS 失败，期望 {}, 实际 {}", current, actual);
                    }
                }
                
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        
        // 线程2
        s.spawn(|| {
            for _ in 0..3 {
                let current = shared_value.load(Ordering::Relaxed);
                let new_value = current + 1;
                
                // 使用 AcqRel 的 CAS 操作
                match shared_value.compare_exchange(current, new_value, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => {
                        println!("线程2: 成功更新 {} -> {}", current, new_value);
                    }
                    Err(actual) => {
                        println!("线程2: CAS 失败，期望 {}, 实际 {}", current, actual);
                    }
                }
                
                thread::sleep(std::time::Duration::from_millis(10));
            }
        });
    });
    
    println!("最终值: {}", shared_value.load(Ordering::Relaxed));
}
