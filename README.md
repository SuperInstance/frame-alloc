# Frame Allocator

**A Rust physical memory frame allocator** using buddy-style bitmap tracking with `BTreeSet`-based free/used lists, designed for operating system kernels and embedded memory managers.

## Why It Matters

Every operating system needs a physical frame allocator — the component that hands out 4 KiB pages of physical RAM to the virtual memory subsystem. Without one, you cannot allocate page tables, kernel heaps, or user processes. This implementation uses `BTreeSet<usize>` for both free and used frames, providing **O(log n)** allocation and dealellation with ordered iteration (important for debugging memory layouts). The `lazy_static` global state pattern mirrors how real kernels expose the frame allocator to all subsystems through a global singleton.

## How It Works

Frames are tracked by index (physical address divided by `FRAME_SIZE = 4096`). The allocator maintains two global `BTreeSet<usize>` instances behind `Mutex` guards: `FREE_FRAMES` and `USED_FRAMES`.

- **`init_frame_range(base, len)`**: Seeds the allocator by computing frame indices for every page-aligned address in `[base, base+len)` and inserting them into the free set. Clears both sets first.

- **`alloc_frame()`**: Pops the smallest free frame index (via `iter().next()`) — **O(log n)** due to `BTreeSet` operations. Moves it from free to used.

- **`dealloc_frame(idx)`**: Moves a frame index from used back to free — **O(log n)**. Safely handles double-free by checking membership first.

- **`free_count()` / `used_count()`**: **O(1)** reads of the set length.

The use of `BTreeSet` (rather than `HashSet`) ensures that frames are allocated in ascending physical address order, which improves cache locality and TLB performance on real hardware.

## Quick Start

```rust
use frame_alloc::{init_frame_range, alloc_frame, dealloc_frame, free_count, used_count, FRAME_SIZE};

fn main() {
    // Initialize with 10 pages of physical memory starting at address 0x10000
    init_frame_range(0x10000, FRAME_SIZE * 10);
    assert_eq!(free_count(), 10);

    // Allocate 3 frames
    let f1 = alloc_frame().unwrap();
    let f2 = alloc_frame().unwrap();
    let f3 = alloc_frame().unwrap();
    assert_eq!(used_count(), 3);
    assert_eq!(free_count(), 7);

    // Frame indices correspond to physical page numbers
    println!("Frame indices: {}, {}, {}", f1, f2, f3);

    // Free a frame
    dealloc_frame(f1);
    assert_eq!(free_count(), 8);
}
```

## API

| Function | Complexity | Description |
|---|---|---|
| `init_frame_range(base, len)` | **O(n)** | Seed allocator with page-aligned physical address range |
| `alloc_frame()` | **O(log n)** | Allocate the lowest-indexed free frame |
| `dealloc_frame(idx)` | **O(log n)** | Return a frame to the free pool |
| `free_count()` | **O(1)** | Number of available frames |
| `used_count()` | **O(1)** | Number of allocated frames |

## Architecture Notes

Part of the SuperInstance systems programming collection, designed for the OS kernel and hypervisor components. See the [Architecture Guide](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
