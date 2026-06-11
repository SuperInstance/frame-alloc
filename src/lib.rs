//! Physical frame allocator — tracks free physical memory frames.

use std::collections::BTreeSet;
use std::sync::Mutex;

const FRAME_SIZE: usize = 4096;

lazy_static::lazy_static! {
    static ref FREE_FRAMES: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());
    static ref USED_FRAMES: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());
}

/// Initialize the frame allocator with a range of physical addresses.
pub fn init_frame_range(base: usize, len: usize) {
    let mut free = FREE_FRAMES.lock().unwrap();
    let mut used = USED_FRAMES.lock().unwrap();
    used.clear();
    free.clear();
    for i in (base..base + len).step_by(FRAME_SIZE) {
        free.insert(i / FRAME_SIZE);
    }
}

/// Allocate a single physical frame. Returns the frame index.
pub fn alloc_frame() -> Option<usize> {
    let mut free = FREE_FRAMES.lock().unwrap();
    let mut used = USED_FRAMES.lock().unwrap();
    let idx = free.iter().next().copied()?;
    free.remove(&idx);
    used.insert(idx);
    Some(idx)
}

/// Deallocate a physical frame by index.
pub fn dealloc_frame(idx: usize) {
    let mut free = FREE_FRAMES.lock().unwrap();
    let mut used = USED_FRAMES.lock().unwrap();
    if used.remove(&idx) {
        free.insert(idx);
    }
}

/// Return number of free frames.
pub fn free_count() -> usize {
    FREE_FRAMES.lock().unwrap().len()
}

/// Return number of used frames.
pub fn used_count() -> usize {
    USED_FRAMES.lock().unwrap().len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_dealloc() {
        init_frame_range(0, FRAME_SIZE * 10);
        assert_eq!(free_count(), 10);
        let f = alloc_frame().unwrap();
        assert_eq!(free_count(), 9);
        assert_eq!(used_count(), 1);
        dealloc_frame(f);
        assert_eq!(free_count(), 10);
    }
}
