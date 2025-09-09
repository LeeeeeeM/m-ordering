use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

fn main() {
    println!("=== Relaxed 排序 1000 次测试 ===");
    test_without_ordering_1000_times();
    test_acquire_release_1000_times();
}

fn test_without_ordering_1000_times() {
    println!("\n--- Relaxed 排序 1000 次测试（重排序挑战版）---");
    
    let mut success_count = 0;
    let mut failure_count = 0;
    let total_tests = 1000;
    
    for test_num in 1..=total_tests {
        let data1 = AtomicU32::new(0);
        let data2 = AtomicU32::new(0);
        let data3 = AtomicU32::new(0);
        let ready = AtomicU32::new(0);
        let mut test_success = false;
        let mut test_failure_reason = String::new();
        
        thread::scope(|s| {
            // 线程1: 写入多个数据
            s.spawn(|| {
                // 模拟一些计算工作，增加竞争窗口
                for _ in 0..500 { let _ = 1 + 1; }
                
                // 写入多个数据，增加重排序的可能性
                data1.store(100, Ordering::Relaxed);
                data2.store(200, Ordering::Relaxed);
                data3.store(300, Ordering::Relaxed);
                
                // 使用 Relaxed 排序标记数据准备完成
                ready.store(1, Ordering::Relaxed);
            });
            
            // 线程2: 读取数据
            s.spawn(|| {
                // 使用 Relaxed 排序等待数据准备完成
                while ready.load(Ordering::Relaxed) == 0 {
                    // 等待数据准备完成
                }
                
                // 读取多个数据
                let value1 = data1.load(Ordering::Relaxed);
                let value2 = data2.load(Ordering::Relaxed);
                let value3 = data3.load(Ordering::Relaxed);
                
                // 检查是否读取到正确的数据
                if value1 == 100 && value2 == 200 && value3 == 300 {
                    test_success = true;
                } else {
                    test_failure_reason = format!("读取到错误数据: data1={}, data2={}, data3={}", value1, value2, value3);
                }
            });
        });
        
        if test_success {
            success_count += 1;
        } else {
            failure_count += 1;
            if failure_count <= 5 { // 只打印前5次失败的原因
                println!("测试 {} 失败: {}", test_num, test_failure_reason);
            }
        }
        
        // 每100次测试打印一次进度
        if test_num % 100 == 0 {
            println!("已完成 {} 次测试...", test_num);
        }
    }
    
    println!("\n=== 测试结果统计 ===");
    println!("总测试次数: {}", total_tests);
    println!("成功次数: {} ({:.1}%)", success_count, success_count as f64 / total_tests as f64 * 100.0);
    println!("失败次数: {} ({:.1}%)", failure_count, failure_count as f64 / total_tests as f64 * 100.0);
    
    if failure_count > 0 {
        println!("\n⚠️  发现 Relaxed 排序的问题！");
        println!("在 {} 次测试中，有 {} 次失败", total_tests, failure_count);
        println!("这说明 Relaxed 排序在某些情况下可能读取到错误数据");
    } else {
        println!("\n✅ 在这个测试中，Relaxed 排序工作正常");
        println!("但这不意味着 Relaxed 排序在所有情况下都安全");
        println!("在更复杂的场景中，Relaxed 排序仍可能导致问题");
    }
    
    // 对比 Acquire-Release 排序
    println!("\n--- 对比：Acquire-Release 排序 1000 次测试 ---");
    test_acquire_release_1000_times();
}

fn test_acquire_release_1000_times() {
    let mut success_count = 0;
    let mut failure_count = 0;
    let total_tests = 1000;
    
    for test_num in 1..=total_tests {
        let data1 = AtomicU32::new(0);
        let data2 = AtomicU32::new(0);
        let data3 = AtomicU32::new(0);
        let ready = AtomicU32::new(0);
        let mut test_success = false;
        
        thread::scope(|s| {
            // 线程1: 写入多个数据
            s.spawn(|| {
                // 模拟一些计算工作，增加竞争窗口
                for _ in 0..1000 { let _ = 1 + 1; }
                
                // 写入多个数据
                data1.store(1000, Ordering::Relaxed);
                data2.store(200, Ordering::Relaxed);
                data3.store(300, Ordering::Relaxed);
                
                // 使用 Release 排序标记数据准备完成
                ready.store(1, Ordering::Release);
            });
            
            // 线程2: 读取数据
            s.spawn(|| {
                // 使用 Acquire 排序等待数据准备完成
                while ready.load(Ordering::Acquire) == 0 {
                    // 等待数据准备完成
                }
                
                // 读取多个数据
                let value1 = data1.load(Ordering::Relaxed);
                let value2 = data2.load(Ordering::Relaxed);
                let value3 = data3.load(Ordering::Relaxed);
                
                // 检查是否读取到正确的数据
                if value1 == 1000 && value2 == 200 && value3 == 300 {
                    test_success = true;
                }
            });
        });
        
        if test_success {
            success_count += 1;
        } else {
            failure_count += 1;
        }
        
        // 每100次测试打印一次进度
        if test_num % 100 == 0 {
            println!("已完成 {} 次测试...", test_num);
        }
    }
    
    println!("\n=== Acquire-Release 测试结果统计 ===");
    println!("总测试次数: {}", total_tests);
    println!("成功次数: {} ({:.1}%)", success_count, success_count as f64 / total_tests as f64 * 100.0);
    println!("失败次数: {} ({:.1}%)", failure_count, failure_count as f64 / total_tests as f64 * 100.0);
    
    if failure_count == 0 {
        println!("\n✅ Acquire-Release 排序 100% 成功！");
        println!("这证明了 Acquire-Release 排序的可靠性");
    } else {
        println!("\n❌ Acquire-Release 排序也有问题！");
        println!("这可能是测试环境的问题");
    }
}

