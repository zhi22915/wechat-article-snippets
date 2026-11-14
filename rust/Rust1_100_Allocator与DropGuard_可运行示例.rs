// Rust 1.100.0 · Allocator API 与 DropGuard API 可运行示例集
// 配套文章：Rust1.100_Allocator与DropGuard_深度实战.md
// 编译：rustc --edition 2021 -o rust1100_verify Rust1_100_Allocator与DropGuard_可运行示例.rs
// 运行：./rust1100_verify
// 验证环境：rustc 1.100.0 (0ef1bc386 2026-11-09)，macOS aarch64
// 共 11 组示例，全部编译通过并运行成功。
// 说明：demo_panic 会向 stderr 打印一次预期内的 panic 信息，属正常输出。

use std::alloc::{AllocError, Allocator, Global, Layout, System};
use std::mem::DropGuard;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

// ============ 示例 1：Global / System 显式构造 ============
fn demo_global() {
    let mut v: Vec<u8, Global> = Vec::new_in(Global);
    v.extend_from_slice(b"rust 1.100");
    let b: Box<u32, Global> = Box::new_in(7, Global);
    let s: Vec<u8, System> = Vec::with_capacity_in(8, System);
    println!("[1] v={} box={} sys_cap={}", String::from_utf8_lossy(&v), *b, s.capacity());
    println!("[1] allocator size = {}", std::mem::size_of_val(v.allocator()));
}

// ============ 示例 2：计数分配器 ============
#[derive(Default)]
struct Counting {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    live_bytes: AtomicUsize,
}

unsafe impl Allocator for Counting {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let block = Global.allocate(layout)?;
        self.allocations.fetch_add(1, Ordering::Relaxed);
        self.live_bytes.fetch_add(layout.size(), Ordering::Relaxed);
        Ok(block)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { Global.deallocate(ptr, layout) };
        self.deallocations.fetch_add(1, Ordering::Relaxed);
        self.live_bytes.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

fn demo_counting() {
    let c = Counting::default();
    {
        let mut v: Vec<u8, &Counting> = Vec::with_capacity_in(4, &c);
        v.extend_from_slice(b"rust");
        v.push(b'!'); // 触发一次 grow
        assert_eq!(v.as_slice(), b"rust!");
        println!(
            "[2] inside alloc={} live={} cap={}",
            c.allocations.load(Ordering::Relaxed),
            c.live_bytes.load(Ordering::Relaxed),
            v.capacity()
        );
    }
    println!(
        "[2] after alloc={} dealloc={} live={}",
        c.allocations.load(Ordering::Relaxed),
        c.deallocations.load(Ordering::Relaxed),
        c.live_bytes.load(Ordering::Relaxed)
    );
    let b: Box<u32, &Counting> = Box::new_in(42, &c);
    let boxed_alloc: &Counting = *Box::allocator(&b);
    println!("[2] box={} uses same allocator={}", *b, std::ptr::eq(boxed_alloc as *const Counting, &c as *const Counting));
}

// ============ 示例 3：失败注入 + try_reserve ============
struct Budget {
    max: usize,
    used: AtomicUsize,
}

unsafe impl Allocator for Budget {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if self.used.load(Ordering::Relaxed) + layout.size() > self.max {
            return Err(AllocError);
        }
        let block = Global.allocate(layout)?;
        self.used.fetch_add(layout.size(), Ordering::Relaxed);
        Ok(block)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { Global.deallocate(ptr, layout) };
        self.used.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

fn demo_budget() {
    let budget = Budget { max: 32, used: AtomicUsize::new(0) };
    let mut v: Vec<u8, &Budget> = Vec::new_in(&budget);
    println!("[3] try_reserve(16) ok={}", v.try_reserve(16).is_ok());
    println!("[3] try_reserve(64) ok={}", v.try_reserve(64).is_ok());
}

// ============ 示例 4：零大小分配 ============
fn demo_zst() {
    let c = Counting::default();
    let layout = Layout::new::<()>();
    let block = c.allocate(layout).unwrap();
    println!("[4] zero-sized block len={} size={}", block.len(), layout.size());
    let ptr = NonNull::new(block.as_ptr() as *mut u8).unwrap();
    unsafe { c.deallocate(ptr, layout) };
}

// ============ 示例 5：raw parts 往返，分配器必须一起带 ============
fn demo_raw() {
    let c = Counting::default();
    let mut v: Vec<u8, &Counting> = Vec::new_in(&c);
    v.extend_from_slice(b"abc");
    let (ptr, len, cap, alloc) = v.into_raw_parts_with_allocator();
    let back = unsafe { Vec::from_raw_parts_in(ptr, len, cap, alloc) };
    println!("[5] vec back = {:?}", back.as_slice());

    let b: Box<u32, &Counting> = Box::new_in(9, &c);
    let (raw, a) = Box::into_raw_with_allocator(b);
    let b2 = unsafe { Box::from_raw_in(raw, a) };
    println!("[5] box back = {}", *b2);
}

// ============ 示例 6：DropGuard 事务回滚（博客原例）============
fn parse_all(nums: &mut Vec<u32>, strings: &[&str]) -> Result<(), std::num::ParseIntError> {
    let len = nums.len();
    let mut nums = DropGuard::new(nums, move |nums| nums.truncate(len));
    for s in strings {
        nums.push(s.parse()?);
    }
    DropGuard::dismiss(nums);
    Ok(())
}

fn demo_parse() {
    let mut nums = vec![1, 2];
    println!("[6] ok={:?} nums={:?}", parse_all(&mut nums, &["3", "4"]).is_ok(), nums);
    println!("[6] err={:?} nums={:?}", parse_all(&mut nums, &["5", "oops"]).is_err(), nums);
}

// ============ 示例 7：临时改配置，自动恢复 ============
struct Config {
    verbose: bool,
}

fn demo_config() {
    let mut cfg = Config { verbose: false };
    {
        let old = cfg.verbose;
        let mut g = DropGuard::new(&mut cfg, move |c| c.verbose = old);
        g.verbose = true;
        println!("[7] inside verbose = {}", g.verbose);
    }
    println!("[7] after verbose = {}", cfg.verbose);
}

// ============ 示例 8：dismiss 拿回值，闭包不执行 ============
fn demo_dismiss() {
    let g = DropGuard::new(String::from("keep"), |s| println!("[8] closure ran: {s}"));
    let s = DropGuard::dismiss(g);
    println!("[8] dismissed value = {s}");
}

// ============ 示例 9：panic 也走清理 ============
fn demo_panic() {
    let mut flag = false;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut g = DropGuard::new(&mut flag, |f| *f = true);
        let _ = &mut g;
        panic!("boom");
    }));
    println!("[9] panicked={} flag_reset={}", result.is_err(), flag);
}

// ============ 示例 10：Arc / Rc 作为分配器 ============
fn demo_smart_allocator() {
    use std::rc::Rc;
    use std::sync::Arc;

    let arc = Arc::new(Counting::default());
    let mut v: Vec<u8, Arc<Counting>> = Vec::new_in(Arc::clone(&arc));
    v.extend_from_slice(b"arc");
    println!("[10] arc allocations={}", arc.allocations.load(Ordering::Relaxed));
    println!("[10] vec allocator points to arc = {}", Arc::ptr_eq(v.allocator(), &arc));

    let rc = Rc::new(Counting::default());
    let b: Box<u8, Rc<Counting>> = Box::new_in(1, Rc::clone(&rc));
    println!("[10] rc allocations={} box={}", rc.allocations.load(Ordering::Relaxed), *b);
}

// ============ 示例 11：dismiss 会释放闭包捕获的资源 ============
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("[11] drop {}", self.0);
    }
}

fn demo_forget_vs_dismiss() {
    let token = Noisy("forgotten");
    let g = DropGuard::new(0u8, move |_| {
        let _ = &token;
    });
    std::mem::forget(g);
    println!("[11] after mem::forget (上面不应有 drop 输出)");

    let token2 = Noisy("dismissed");
    let g2 = DropGuard::new(0u8, move |_| {
        let _ = &token2;
    });
    let _ = DropGuard::dismiss(g2);
    println!("[11] after dismiss (上面应有 drop dismissed)");
}

fn main() {
    demo_global();
    demo_counting();
    demo_budget();
    demo_zst();
    demo_raw();
    demo_parse();
    demo_config();
    demo_dismiss();
    demo_panic();
    demo_smart_allocator();
    demo_forget_vs_dismiss();
}
