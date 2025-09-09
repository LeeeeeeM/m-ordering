use std::{sync::atomic::{AtomicUsize, Ordering}, thread};

fn main() {
    println!("=== 真正的 ABA 问题演示 ===");
    let counter = AtomicUsize::new(0);
    
    thread::scope(|s| {
        // 线程1：执行 A -> B -> A 操作
        s.spawn(|| {
            // 做一些计算工作
            for _ in 0..1000 {
                let _ = 1 + 1;
            }
            
            // A -> B
            counter.store(1, Ordering::Relaxed);
            
            // 做一些计算工作
            for _ in 0..500 {
                let _ = 2 * 2;
            }
            
            // B -> A
            counter.store(0, Ordering::Relaxed);
        });
        
        // 线程2：尝试检测变化并执行操作
        s.spawn(|| {
            // 读取初始值
            let initial_value = counter.load(Ordering::Relaxed);
            
            // 做一些计算工作，增加竞争窗口
            for _ in 0..2000 {
                let _ = 3 + 3;
            }
            
            // 尝试使用 CAS 操作：如果值还是 initial_value，就设置为 100
            let new_value = 100;
            match counter.compare_exchange(initial_value, new_value, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => {
                    println!("CAS 成功！从 {} 更新到 {} (可能是ABA问题！)", initial_value, new_value);
                }
                Err(actual) => {
                    println!("CAS 失败！期望: {}, 实际: {} (检测到并发修改，这是好的)", initial_value, actual);
                }
            }
        });
    });
    
    let final_value = counter.load(Ordering::Relaxed);
    println!("最终计数器值: {}", final_value);
    
    if final_value == 100 {
        println!("*** 发生了 ABA 问题！ ***");
        println!("线程2 的 CAS 操作被欺骗了，认为值没有变化");
    } else {
        println!("没有发生 ABA 问题");
    }
}