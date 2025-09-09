use std::{sync::atomic::{AtomicUsize, Ordering}, thread};

fn main() {
    println!("=== ABA 问题多次测试演示（执行50次）===");
    
    let mut aba_count = 0;
    let mut normal_count = 0;
    
    for test_num in 1..=50 {
        let counter = AtomicUsize::new(0);
        let mut cas_success = false;
        let mut cas_failed = false;
        
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
                        cas_success = true;
                    }
                    Err(_) => {
                        cas_failed = true;
                    }
                }
            });
        });
        
        let final_value = counter.load(Ordering::Relaxed);
        
        if final_value == 100 {
            aba_count += 1;
            println!("测试 {}: ABA 问题发生！最终值: {}", test_num, final_value);
        } else {
            normal_count += 1;
            if cas_success {
                println!("测试 {}: 正常情况，CAS 成功，最终值: {}", test_num, final_value);
            } else if cas_failed {
                println!("测试 {}: 正常情况，CAS 失败，最终值: {}", test_num, final_value);
            } else {
                println!("测试 {}: 其他情况，最终值: {}", test_num, final_value);
            }
        }
    }
    
    println!("\n=== 统计结果 ===");
    println!("总测试次数: 50");
    println!("ABA 问题发生次数: {} ({:.1}%)", aba_count, aba_count as f64 / 50.0 * 100.0);
    println!("正常情况次数: {} ({:.1}%)", normal_count, normal_count as f64 / 50.0 * 100.0);
    
    if aba_count > 0 {
        println!("\n*** 检测到 ABA 问题！ ***");
        println!("在 {} 次测试中，有 {} 次发生了 ABA 问题", 50, aba_count);
        println!("这说明 ABA 问题确实存在，需要采取措施防止");
    } else {
        println!("\n*** 没有检测到 ABA 问题 ***");
        println!("在 50 次测试中都没有发生 ABA 问题");
    }
}
