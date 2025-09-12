use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;
use std::sync::Arc;
use std::sync::Mutex;
use rand::Rng;

fn main() {
    test_realistic_seckill_scenario();
}

// 模拟数据库操作
struct Database {
    stock: AtomicU32,
    orders: Mutex<Vec<Order>>,  // 恢复 Mutex
}

#[derive(Debug, Clone)]
struct Order {
    user_id: u32,
    product_id: u32,
    quantity: u32,
    timestamp: std::time::Instant,
}

impl Database {
    fn new(initial_stock: u32) -> Self {
        Self {
            stock: AtomicU32::new(initial_stock),
            orders: Mutex::new(Vec::new()),
        }
    }
    
    // 模拟从数据库读取库存
    fn read_stock(&self) -> u32 {
        // 模拟数据库查询延迟
        thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(1..5)));
        self.stock.load(Ordering::Relaxed)
    }
    
    // 模拟扣减库存的数据库操作
    fn try_purchase(&self, user_id: u32, product_id: u32, quantity: u32) -> Result<u32, String> {
        // 模拟数据库事务开始
        thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(2..8)));
        
        // 使用循环尝试原子操作，确保库存充足
        loop {
            let current_stock = self.stock.load(Ordering::Relaxed);
            
            if current_stock < quantity {
                return Err("库存不足".to_string());
            }
            
            // 尝试原子性地扣减库存
            match self.stock.compare_exchange_weak(
                current_stock,
                current_stock - quantity,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    // 扣减成功，模拟写入订单表
                    thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(1..3)));
                    
                    let order = Order {
                        user_id,
                        product_id,
                        quantity,
                        timestamp: std::time::Instant::now(),
                    };
                    
                    // 模拟写入数据库
                    if let Ok(mut orders) = self.orders.lock() {
                        orders.push(order);
                    }
                    
                    // 模拟数据库事务提交
                    thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(1..2)));
                    
                    return Ok(current_stock - quantity);
                }
                Err(_) => {
                    // 其他线程修改了库存，重试
                    continue;
                }
            }
        }
    }
    
    // 获取最终统计
    fn get_stats(&self) -> (u32, usize) {
        let final_stock = self.stock.load(Ordering::Relaxed);
        let order_count = self.orders.lock().unwrap().len();
        (final_stock, order_count)
    }
    
    // 获取订单详情（用于演示 Order 结构体的使用）
    fn get_orders(&self) -> Vec<Order> {
        self.orders.lock().unwrap().clone()
    }
    
    // 打印订单统计信息
    fn print_order_stats(&self) {
        let orders = self.get_orders();
        if !orders.is_empty() {
            println!("\n=== 订单详情 ===");
            println!("总订单数: {}", orders.len());
            
            // 按用户ID分组统计
            let mut user_orders: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
            for order in &orders {
                *user_orders.entry(order.user_id).or_insert(0) += order.quantity;
            }
            
            println!("购买用户数: {}", user_orders.len());
            
            // 显示前10个订单的详情
            println!("\n前10个订单:");
            for (i, order) in orders.iter().take(10).enumerate() {
                println!("  {}: 用户{} 购买商品{} 数量{} 时间{:?}", 
                    i + 1, order.user_id, order.product_id, order.quantity, order.timestamp);
            }
            
            if orders.len() > 10 {
                println!("  ... 还有 {} 个订单", orders.len() - 10);
            }
        }
    }
}

fn test_realistic_seckill_scenario() {
    println!("=== 真实秒杀场景模拟 ===");
    println!("商品ID: 1001");
    println!("初始库存: 10 个");
    println!("参与用户: 1000 人");
    println!("模拟真实数据库操作、网络延迟等");
    println!("----------------------------------------");
    
    // 模拟数据库
    let db = Arc::new(Database::new(10));
    let success_count = Arc::new(AtomicU32::new(0));
    let fail_count = Arc::new(AtomicU32::new(0));
    
    let start_time = std::time::Instant::now();
    
    thread::scope(|s| {
        // 模拟 1000 个用户同时秒杀
        for user_id in 1..=1000 {
            let db = db.clone();
            let success_count = success_count.clone();
            let fail_count = fail_count.clone();
            
            s.spawn(move || {
                // 模拟用户操作流程
                simulate_user_purchase(user_id, db, success_count, fail_count);
            });
        }
    });
    
    let end_time = std::time::Instant::now();
    let duration = end_time.duration_since(start_time);
    
    // 输出最终结果
    println!("----------------------------------------");
    println!("秒杀结束！");
    println!("总耗时: {:?}", duration);
    
    let (final_stock, order_count) = db.get_stats();
    println!("最终库存: {}", final_stock);
    println!("成功订单数: {}", order_count);
    println!("成功购买人数: {}", success_count.load(Ordering::Relaxed));
    println!("失败人数: {}", fail_count.load(Ordering::Relaxed));
    
    // 打印订单详情，使用 Order 结构体的字段
    db.print_order_stats();
    
    // 验证结果
    let total_attempts = success_count.load(Ordering::Relaxed) + fail_count.load(Ordering::Relaxed);
    println!("总参与人数: {}", total_attempts);
    
    if order_count == 10 {
        println!("✅ 验证通过：成功订单数等于库存数量");
    } else {
        println!("❌ 验证失败：成功订单数不等于库存数量");
    }
    
    if final_stock == 0 {
        println!("✅ 验证通过：库存已售罄");
    } else {
        println!("❌ 验证失败：库存未售罄");
    }
}

fn simulate_user_purchase(
    user_id: u32,
    db: Arc<Database>,
    success_count: Arc<AtomicU32>,
    fail_count: Arc<AtomicU32>,
) {
    // 1. 模拟用户点击秒杀按钮
    // 模拟网络延迟
    thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(1..10)));
    
    // 2. 模拟前端验证（检查用户是否已登录等）
    thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(1..3)));
    
    // 3. 模拟查询库存（前端可能先查一下）
    let _current_stock = db.read_stock();
    
    // 4. 模拟用户提交订单
    thread::sleep(Duration::from_millis(rand::thread_rng().gen_range(1..5)));
    
    // 5. 尝试购买（数据库操作）
    match db.try_purchase(user_id, 1001, 1) {
        Ok(remaining_stock) => {
            success_count.fetch_add(1, Ordering::Relaxed);
            println!("用户 {} 购买成功，剩余库存: {}", user_id, remaining_stock);
        }
        Err(reason) => {
            fail_count.fetch_add(1, Ordering::Relaxed);
            println!("用户 {} 购买失败: {}", user_id, reason);
        }
    }
}

