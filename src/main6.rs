use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

fn main() {
    println!("=== Acquire 和 Release 内存序演示 ===");
    
    // 演示1: Acquire-Release 配对
    test_acquire_release_pairing();
    
    // 演示2: 没有内存序的问题
    test_without_ordering();
    
    // 演示3: 版本号方案中的实际应用
    test_versioned_scenario();
    
    // 演示4: 内存序的具体作用
    demonstrate_memory_ordering();
}

fn test_acquire_release_pairing() {
    println!("\n--- 演示1: Acquire-Release 配对 ---");
    
    let data = AtomicU32::new(0);
    let ready = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1: 写入数据
        s.spawn(|| {
            // 准备数据
            data.store(42, Ordering::Relaxed);
            println!("线程1: 写入数据 42");
            
            // 使用 Release 排序标记数据准备完成
            ready.store(1, Ordering::Release);
            println!("线程1: 标记数据准备完成 (Release)");
        });
        
        // 线程2: 读取数据
        s.spawn(|| {
            // 使用 Acquire 排序等待数据准备完成
            while ready.load(Ordering::Acquire) == 0 {
                // 等待数据准备完成
            }
            println!("线程2: 检测到数据准备完成 (Acquire)");
            
            // 读取数据
            let value = data.load(Ordering::Relaxed);
            println!("线程2: 读取到数据 {}", value);
        });
    });
}

fn test_without_ordering() {
    println!("\n--- 演示2: 没有内存序的问题 ---");
    
    let data = AtomicU32::new(0);
    let ready = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1: 写入数据
        s.spawn(|| {
            // 准备数据
            data.store(42, Ordering::Relaxed);
            println!("线程1: 写入数据 42");
            
            // 使用 Relaxed 排序标记数据准备完成
            ready.store(1, Ordering::Relaxed);
            println!("线程1: 标记数据准备完成 (Relaxed)");
        });
        
        // 线程2: 读取数据
        s.spawn(|| {
            // 使用 Relaxed 排序等待数据准备完成
            while ready.load(Ordering::Relaxed) == 0 {
                // 等待数据准备完成
            }
            println!("线程2: 检测到数据准备完成 (Relaxed)");
            
            // 读取数据
            let value = data.load(Ordering::Relaxed);
            println!("线程2: 读取到数据 {}", value);
        });
    });
}

fn test_versioned_scenario() {
    println!("\n--- 演示3: 版本号方案中的实际应用 ---");
    
    let counter = AtomicU32::new(0);
    let version = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1: 执行 A -> B -> A 操作
        s.spawn(|| {
            // A -> B
            counter.store(1, Ordering::Relaxed);
            version.store(1, Ordering::Release);  // 使用 Release 排序
            println!("线程1: 0 -> 1, version=1 (Release)");
            
            // 一些计算工作
            for _ in 0..1000 { let _ = 1 + 1; }
            
            // B -> A
            counter.store(0, Ordering::Relaxed);
            version.store(2, Ordering::Release);  // 使用 Release 排序
            println!("线程1: 1 -> 0, version=2 (Release)");
        });
        
        // 线程2: 检测ABA问题
        s.spawn(|| {
            // 读取初始状态 - 使用 Acquire 排序
            let initial_counter = counter.load(Ordering::Relaxed);
            let initial_version = version.load(Ordering::Acquire);
            println!("线程2: 初始读取 counter={}, version={} (Acquire)", 
                    initial_counter, initial_version);
            
            // 一些计算工作
            for _ in 0..2000 { let _ = 1 + 1; }
            
            // 重新读取状态 - 使用 Acquire 排序
            let current_counter = counter.load(Ordering::Relaxed);
            let current_version = version.load(Ordering::Acquire);
            println!("线程2: 重新读取 counter={}, version={} (Acquire)", 
                    current_counter, current_version);
            
            // 检测ABA问题
            if current_counter == initial_counter && current_version != initial_version {
                println!("线程2: 检测到ABA问题！值相同但版本号不同");
            } else if current_version > initial_version {
                println!("线程2: 检测到版本号变化，拒绝CAS操作");
            } else {
                println!("线程2: 版本号未变化，可以安全执行CAS");
            }
        });
    });
}

// 演示内存序的具体作用
fn demonstrate_memory_ordering() {
    println!("\n--- 内存序的具体作用演示 ---");
    
    let data1 = AtomicU32::new(0);
    let data2 = AtomicU32::new(0);
    let sync_point = AtomicU32::new(0);
    
    thread::scope(|s| {
        // 线程1: 写入数据
        s.spawn(|| {
            data1.store(100, Ordering::Relaxed);
            data2.store(200, Ordering::Relaxed);
            println!("线程1: 写入 data1=100, data2=200");
            
            // 使用 Release 排序建立同步点
            sync_point.store(1, Ordering::Release);
            println!("线程1: 建立同步点 (Release)");
        });
        
        // 线程2: 读取数据
        s.spawn(|| {
            // 使用 Acquire 排序等待同步点
            while sync_point.load(Ordering::Acquire) == 0 {
                // 等待同步点
            }
            println!("线程2: 检测到同步点 (Acquire)");
            
            // 读取数据
            let value1 = data1.load(Ordering::Relaxed);
            let value2 = data2.load(Ordering::Relaxed);
            println!("线程2: 读取到 data1={}, data2={}", value1, value2);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_acquire_release_synchronization() {
        let data = AtomicU32::new(0);
        let ready = AtomicU32::new(0);
        
        thread::scope(|s| {
            // 线程1: 写入数据
            s.spawn(|| {
                data.store(42, Ordering::Relaxed);
                ready.store(1, Ordering::Release);
            });
            
            // 线程2: 读取数据
            s.spawn(|| {
                while ready.load(Ordering::Acquire) == 0 {
                    // 等待数据准备完成
                }
                let value = data.load(Ordering::Relaxed);
                assert_eq!(value, 42);
            });
        });
    }
}
