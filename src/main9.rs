use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

fn main() {
    test_fetch_add_example();
}

fn test_fetch_add_example() {
    let counter = AtomicU32::new(0);
    
    println!("开始测试 fetch_add 操作...");
    println!("两个线程，每个线程执行 500 次 fetch_add 操作");
    println!("使用 Relaxed 内存排序");
    println!("----------------------------------------");
    
    thread::scope(|s| {
        // 线程1: 执行 500 次 fetch_add
        s.spawn(|| {
            for i in 1..=500 {
                let current_value = counter.fetch_add(1, Ordering::Relaxed);
                println!("线程1: 第{}次操作，fetch_add前值: {}, fetch_add后值: {}", 
                        i, current_value, current_value + 1);
            }
            println!("线程1: 完成所有 500 次操作");
        });
        
        // 线程2: 执行 500 次 fetch_add
        s.spawn(|| {
            for i in 1..=500 {
                let current_value = counter.fetch_add(1, Ordering::Relaxed);
                println!("线程2: 第{}次操作，fetch_add前值: {}, fetch_add后值: {}", 
                        i, current_value, current_value + 1);
            }
            println!("线程2: 完成所有 500 次操作");
        });
    });
    
    // 等待所有线程完成后，打印最终结果
    let final_value = counter.load(Ordering::Relaxed);
    println!("----------------------------------------");
    println!("最终计数器值: {}", final_value);
    println!("预期值: 1000 (500 + 500)");
    
    if final_value == 1000 {
        println!("✅ 测试通过：计数器值正确");
    } else {
        println!("❌ 测试失败：计数器值不正确");
    }
}
