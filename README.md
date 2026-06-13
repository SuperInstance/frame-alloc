# frame-alloc: Physical Frame Allocator for Memory Management

A buddy-system-free frame allocator that tracks physical memory pages using a `BTreeSet`-backed free list. It provides O(log N) allocation and deallocation of fixed-size (4 KiB) physical frames with mutex-protected concurrent access.

## Why It Matters

Every operating system kernel needs a physical frame allocator. The page fault handler asks "give me a free 4 KiB page," and the allocator must respond in sub-microsecond time. This implementation demonstrates the simplest correct approach: a sorted set of free frame indices. While production kernels use buddy allocators or zone allocators (e.g., Linux's `alloc_pages()`), the free-list approach is pedagogically clear and shows up in embedded systems, hypervisors, and unikernels.

## How It Works

### Data Structure

Two `BTreeSet<usize>` instances partition the frame space:

```
FREE_FRAMES: BTreeSet<usize>  // indices of available frames
USED_FRAMES: BTreeSet<usize>  // indices of allocated frames
```

Each index represents one 4096-byte frame. Frame index `i` maps to physical address `i × 4096`.

### Allocation

```rust
fn alloc_frame() -> Option<usize> {
    // Pop the minimum element from FREE_FRAMES
    let idx = free.iter().next().copied()?;
    free.remove(&idx);
    used.insert(idx);
    Some(idx)
}
```

**Complexity**: O(log N) for the `BTreeSet` minimum lookup + removal.

### Deallocation

```rust
fn dealloc_frame(idx: usize) {
    if used.remove(&idx) {
        free.insert(idx);
    }
}
```

**Complexity**: O(log N) for set removal + insertion.

### Initialization

Given a physical address range [base, base + len), the allocator divides it into 4 KiB frames:

```
for addr in (base..base+len).step_by(4096):
    free.insert(addr / 4096)
```

### Concurrency

All operations go through `Mutex<BTreeSet>`, making the allocator safe for concurrent access from multiple threads. The locking strategy is coarse (one mutex per free list), which is simple but may become a bottleneck under heavy contention.

### Complexity Summary

| Operation | Time | Notes |
|-----------|------|-------|
| `init_frame_range(base, len)` | O(N) | N = len / 4096 frames |
| `alloc_frame()` | O(log N) | BTreeSet min + remove |
| `dealloc_frame(idx)` | O(log N) | BTreeSet remove + insert |
| `free_count()` / `used_count()` | O(1) | BTreeSet len |

## Quick Start

```rust
use frame_alloc::{init_frame_range, alloc_frame, dealloc_frame, free_count, used_count};

const FRAME_SIZE: usize = 4096;

// Initialize 10 frames starting at physical address 0
init_frame_range(0, FRAME_SIZE * 10);
assert_eq!(free_count(), 10);

// Allocate
let f = alloc_frame().unwrap();
assert_eq!(free_count(), 9);
assert_eq!(used_count(), 1);

// Deallocate
dealloc_frame(f);
assert_eq!(free_count(), 10);
```

## API

| Function | Signature | Description |
|----------|-----------|-------------|
| `init_frame_range` | `(base: usize, len: usize)` | Initialize allocator with physical range |
| `alloc_frame` | `() -> Option<usize>` | Allocate one frame, returns index |
| `dealloc_frame` | `(idx: usize)` | Free a frame by index |
| `free_count` | `() -> usize` | Number of free frames |
| `used_count` | `() -> usize` | Number of allocated frames |

Constants: `FRAME_SIZE = 4096` bytes.

## Architecture Notes

This is a **γ (gamma)** module — deterministic, stateful, and side-effect-free (the state is the allocation bitmap). In the γ + η = C framework, it provides the physical memory substrate upon which **η** orchestration layers (virtual memory, demand paging, memory-mapped I/O) are built. The `BTreeSet` choice is deliberate: it maintains frames in sorted order at O(log N) cost, enabling efficient contiguous-frame queries if needed later.

## References

- Tanenbaum, A. S. & Bos, H. (2014). *Modern Operating Systems* (4th ed.). Pearson. Chapter 3: Memory Management.
- Bovet, D. P. & Cesati, M. (2005). *Understanding the Linux Kernel* (3rd ed.). O'Reilly. Chapter 8: Memory Management.
- Knuth, D. E. (1997). *The Art of Computer Programming, Vol. 1* (3rd ed.), §2.5. Addison-Wesley.

## License

MIT
