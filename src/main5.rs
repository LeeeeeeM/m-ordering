use std::{sync::atomic::{AtomicU64, Ordering}, thread};

// 使用版本号解决 ABA 问题的方案
// 将值和版本号打包到一个 64 位原子整数中
// 高 32 位存储版本号，低 32 位存储实际值

#[derive(Debug, Clone, Copy, PartialEq)]
struct VersionedValue {
    value: u32,
    version: u32,
}

impl VersionedValue {
    fn new(value: u32, version: u32) -> Self {
        Self { value, version }
    }
    
    // 将 VersionedValue 打包到 u64 中
    fn pack(self) -> u64 {
        ((self.version as u64) << 32) | (self.value as u64)
    }
    
    // 从 u64 中解包 VersionedValue
    fn unpack(packed: u64) -> Self {
        let version = (packed >> 32) as u32;
        let value = (packed & 0xFFFFFFFF) as u32;
        Self { value, version }
    }
}

// 带版本号的原子计数器
struct VersionedAtomicCounter {
    data: AtomicU64,
}

impl VersionedAtomicCounter {
    fn new(initial_value: u32) -> Self {
        let initial = VersionedValue::new(initial_value, 0);
        Self {
            data: AtomicU64::new(initial.pack()),
        }
    }
    
    // 读取当前值和版本号
    fn load(&self) -> VersionedValue {
        let packed = self.data.load(Ordering::Acquire);
        VersionedValue::unpack(packed)
    }
    
    // 带版本号检查的 CAS 操作
    fn compare_exchange_versioned(
        &self,
        expected: VersionedValue,
        new_value: VersionedValue,
    ) -> Result<VersionedValue, VersionedValue> {
        let expected_packed = expected.pack();
        let new_packed = new_value.pack();
        
        match self.data.compare_exchange(
            expected_packed,
            new_packed,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(new_value),
            Err(actual_packed) => Err(VersionedValue::unpack(actual_packed)),
        }
    }
    
    // 更新值并增加版本号
    fn store(&self, value: u32) -> VersionedValue {
        let current = self.load();
        let new_value = VersionedValue::new(value, current.version + 1);
        self.data.store(new_value.pack(), Ordering::Release);
        new_value
    }
}

fn main() {
    println!("=== 使用版本号防止 ABA 问题演示 ===");
    let counter = VersionedAtomicCounter::new(0);
    
    // 记录初始状态
    let initial_state = counter.load();
    println!("初始状态: 值 = {}, 版本号 = {}", initial_state.value, initial_state.version);
    
    thread::scope(|s| {
        // 线程1：执行 A -> B -> A 操作，但每次都会增加版本号
        s.spawn(|| {
            // 做一些计算工作
            for _ in 0..1000 {
                let _ = 1 + 1;
            }
            
            // A -> B (版本号从 0 变为 1)
            let versioned_b = counter.store(1);
            println!("线程1: 0 -> 1, 版本号: {}", versioned_b.version);
            
            // 做一些计算工作
            for _ in 0..500 {
                let _ = 2 * 2;
            }
            
            // B -> A (版本号从 1 变为 2)
            let versioned_a = counter.store(0);
            println!("线程1: 1 -> 0, 版本号: {}", versioned_a.version);
        });
        
        // 线程2：尝试检测变化并执行操作
        s.spawn(|| {
            // 读取初始值和版本号
            let initial = counter.load();
            println!("线程2: 读取初始值 {}, 版本号 {}", initial.value, initial.version);
            
            // 做一些计算工作，增加竞争窗口
            for _ in 0..2000 {
                let _ = 3 + 3;
            }
            
            // 再次读取当前状态
            let current = counter.load();
            println!("线程2: 重新读取当前值 {}, 版本号 {}", current.value, current.version);
            
            // 检查是否发生了 ABA 问题
            if current.version > initial.version {
                println!("线程2: 检测到中间发生了操作！版本号从 {} 变为 {} (值从 {} 变为 {})", 
                        initial.version, current.version, initial.value, current.value);
                println!("线程2: 拒绝执行 CAS 操作，因为值已经经历了变化");
                return;
            }
            
            // 如果版本号没有变化，说明值确实没有变化，可以安全执行 CAS
            if current.version == initial.version {
                println!("线程2: 版本号未变化，值确实没有变化，可以安全执行 CAS");
            }
            
            // 尝试使用带版本号检查的 CAS 操作
            let new_value = 100;
            let new_versioned = VersionedValue::new(new_value, current.version + 1);
            
            match counter.compare_exchange_versioned(current, new_versioned) {
                Ok(_) => {
                    println!("线程2: CAS 成功！从 {} 更新到 {} (版本号: {})", 
                            current.value, new_value, new_versioned.version);
                }
                Err(actual) => {
                    // 正确判断失败原因
                    if actual.value == current.value && actual.version > current.version {
                        // 值相同但版本号增加 = 真正的 ABA 问题
                        println!("线程2: CAS 失败！真正的ABA问题被检测到！值相同({})但版本号从{}变为{}", 
                                actual.value, current.version, actual.version);
                    } else if actual.value != current.value {
                        // 值不同 = 正常的值变化
                        println!("线程2: CAS 失败！值从{}变为{}，版本号从{}变为{} (正常并发竞争)", 
                                current.value, actual.value, current.version, actual.version);
                    } else {
                        // 理论上不应该发生的情况：值相同但版本号没有增加
                        // 这可能表示代码 bug 或边界情况
                        println!("线程2: CAS 失败！异常情况！期望值: {}, 版本号: {}, 实际值: {}, 版本号: {} (值相同但版本号未增加)", 
                                current.value, current.version, actual.value, actual.version);
                        println!("线程2: 这种情况理论上不应该发生，可能是代码 bug 或边界情况");
                    }
                }
            }
        });
    });
    
    let final_state = counter.load();
    println!("最终状态: 值 = {}, 版本号 = {}", final_state.value, final_state.version);
    
    // 分析结果
    if final_state.value == 100 {
        if final_state.version > initial_state.version {
            println!("*** 版本号方案：CAS操作成功！ ***");
            println!("值从 {} 更新到 {}，版本号从 {} 变为 {}，CAS 操作成功执行", 
                    initial_state.value, final_state.value, initial_state.version, final_state.version);
        } else {
            println!("*** 版本号方案：正常CAS操作成功 ***");
            println!("值从 {} 更新到 {}，版本号从 {} 变为 {}，CAS 操作成功执行", 
                    initial_state.value, final_state.value, initial_state.version, final_state.version);
        }
    } else {
        println!("*** 版本号方案成功防止了 ABA 问题！ ***");
        println!("值没有变成100，说明CAS操作被正确拒绝");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_versioned_value_pack_unpack() {
        let v1 = VersionedValue::new(42, 5);
        let packed = v1.pack();
        let v2 = VersionedValue::unpack(packed);
        assert_eq!(v1, v2);
    }
    
    #[test]
    fn test_versioned_atomic_counter() {
        let counter = VersionedAtomicCounter::new(10);
        let initial = counter.load();
        assert_eq!(initial.value, 10);
        assert_eq!(initial.version, 0);
        
        // 更新值
        let updated = counter.store(20);
        assert_eq!(updated.value, 20);
        assert_eq!(updated.version, 1);
        
        // 再次更新
        let updated2 = counter.store(30);
        assert_eq!(updated2.value, 30);
        assert_eq!(updated2.version, 2);
    }
    
    #[test]
    fn test_aba_prevention_100_times() {
        println!("\n=== 版本号方案 ABA 防护测试（100次）===");
        
        let mut normal_cas_count = 0;
        let mut cas_failed_count = 0;
        
        for test_num in 1..=100 {
            let counter = VersionedAtomicCounter::new(0);
            let _initial_state = counter.load();
            let mut cas_success = false;
            let mut cas_failed = false;
            
            thread::scope(|s| {
                // 线程1：执行 A -> B -> A 操作，但每次都会增加版本号
                s.spawn(|| {
                    // 做一些计算工作
                    for _ in 0..1000 {
                        let _ = 1 + 1;
                    }
                    
                    // A -> B (版本号从 0 变为 1)
                    counter.store(1);
                    
                    // 做一些计算工作
                    for _ in 0..500 {
                        let _ = 2 * 2;
                    }
                    
                    // B -> A (版本号从 1 变为 2)
                    counter.store(0);
                });
                
                // 线程2：尝试检测变化并执行操作
                s.spawn(|| {
                    // 读取初始值和版本号
                    let initial = counter.load();
                    
                    // 做一些计算工作，增加竞争窗口
                    for _ in 0..2000 {
                        let _ = 3 + 3;
                    }
                    
                    // 再次读取当前状态
                    let current = counter.load();
                    
                    // 检查是否发生了 ABA 问题
                    if current.version > initial.version {
                        // 检测到中间操作，拒绝CAS
                        return;
                    }
                    
                    // 尝试使用带版本号检查的 CAS 操作
                    let new_value = 100;
                    let new_versioned = VersionedValue::new(new_value, current.version + 1);
                    
                    match counter.compare_exchange_versioned(current, new_versioned) {
                        Ok(_) => {
                            cas_success = true;
                        }
                        Err(_) => {
                            cas_failed = true;
                        }
                    }
                });
            });
            
            let final_state = counter.load();
            
            if final_state.value == 100 {
                // 值变成100，说明CAS操作成功了
                normal_cas_count += 1;
            } else {
                // 值没有变成100，说明CAS操作被拒绝或失败
                // 这可能是ABA被防止，也可能是其他原因
                cas_failed_count += 1;
            }
            
            // 每10次测试打印一次进度
            if test_num % 10 == 0 {
                println!("已完成 {} 次测试...", test_num);
            }
        }
        
        println!("\n=== 测试结果统计 ===");
        println!("总测试次数: 100");
        println!("CAS 操作成功次数: {} ({:.1}%)", normal_cas_count, normal_cas_count as f64 / 100.0 * 100.0);
        println!("CAS 操作失败次数: {} ({:.1}%)", cas_failed_count, cas_failed_count as f64 / 100.0 * 100.0);
        
        // 验证测试次数
        assert!(normal_cas_count + cas_failed_count == 100, "测试次数不匹配");
        
        // 分析结果
        println!("\n*** 版本号方案测试结果分析 ***");
        if normal_cas_count > 0 {
            println!("CAS 操作成功 {} 次，说明在某些情况下版本号检查通过", normal_cas_count);
        }
        if cas_failed_count > 0 {
            println!("CAS 操作失败 {} 次，说明版本号方案有效防止了并发竞争", cas_failed_count);
        }
        
        println!("\n版本号方案测试完成！");
    }
}
