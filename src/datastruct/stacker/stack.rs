use std::mem;
use super::heap::Heap;


#[inline]
pub fn is_power_of_2(val: u32) -> bool {
    val & (val - 1) == 0
}

#[inline]
pub fn jump_needed(val: u32) -> bool {
    val > 3 && is_power_of_2(val) 
}


#[derive(Debug)]
pub struct Stack {
    len: u32,
    end: u32,
    val0: u32,
    val1: u32,
    val2: u32,
    next: u32, // inline  of the first block
}

impl Stack {

    pub fn iterate<'a>(&self, addr: u32, heap: &'a Heap) -> StackIterator<'a> {
        StackIterator {
            heap: heap,
            addr: addr + 2u32 * (mem::size_of::<u32>() as u32),
            len: self.len,
            consumed: 0,
        }
    }

    pub fn push(&mut self, val: u32, heap: &Heap) {
        self.len += 1;
        if jump_needed(self.len) {
            // we need to allocate another block.
            // ... As we want to grow block exponentially
            // the next block as a size of (length so far),
            // and we need to add 1u32 to store the pointer
            // to the next element.
            let new_block_size: usize = (self.len as usize + 1) * mem::size_of::<u32>(); 
            let new_block_addr: u32 = heap.allocate(new_block_size);
            heap.set(self.end, &new_block_addr);
            self.end = new_block_addr;     
        }
        heap.set(self.end, &val);
        self.end += mem::size_of::<u32>() as u32;
    }
}


impl From<u32> for Stack {
    fn from(addr: u32) -> Stack {
        let last_addr = addr + mem::size_of::<u32>() as u32 * 2u32;
        Stack {
            len: 0u32,
            end: last_addr,
            val0: 0u32,
            val1: 0u32,
            val2: 0u32,
            next: 0u32,
        }
    }
}



impl Default for Stack {
    fn default() -> Stack {
        Stack {
            len: 0u32,
            end: 0u32,
            val0: 0u32,
            val1: 0u32,
            val2: 0u32,
            next: 0u32,
        }
    }
}


pub struct StackIterator<'a> {
    heap: &'a Heap,
    addr: u32,
    len: u32,
    consumed: u32,
}

impl<'a> Iterator for StackIterator<'a> {
    type Item = u32;

    fn next(&mut self,) -> Option<u32> {
        if self.consumed == self.len {
            None
        }
        else {
            let addr: u32;
            self.consumed += 1;
            if jump_needed(self.consumed) {
                addr = *self.heap.get_mut_ref(self.addr);
            }
            else {
                addr = self.addr;
            }
            self.addr = addr + mem::size_of::<u32>() as u32;
            Some(*self.heap.get_mut_ref(addr))
        }   
        
    }
}





#[cfg(test)]
mod tests {
    
    use super::*;
    use super::super::heap::Heap;
    use test::Bencher;

    const NUM_STACK: usize = 10_000;
    const STACK_SIZE: u32 = 1000;

    #[test]
    fn test_stack() {
        let heap = Heap::with_capacity(1_000_000);
        let (addr, stack) = heap.new::<Stack>();
        stack.push(1u32, &heap);
        stack.push(2u32, &heap);
        stack.push(4u32, &heap);
        stack.push(8u32, &heap);
        {
            let mut it = stack.iterate(addr, &heap);
            assert_eq!(it.next().unwrap(), 1u32);
            assert_eq!(it.next().unwrap(), 2u32);
            assert_eq!(it.next().unwrap(), 4u32);
            assert_eq!(it.next().unwrap(), 8u32);
            assert!(it.next().is_none());
        }
    }

    #[bench]
    fn bench_push_vec(bench: &mut Bencher) {
        bench.iter(|| {
            let mut vecs = Vec::with_capacity(100);
            for _ in 0..NUM_STACK {
                vecs.push(Vec::new());
            }
            for s in 0..NUM_STACK {
                for i in 0u32..STACK_SIZE {
                    let t = s * 392017 % NUM_STACK;
                    vecs[t].push(i);
                }
            }
        });
    }

    #[bench]
    fn bench_push_stack(bench: &mut Bencher) {
        let heap = Heap::with_capacity(64_000_000); 
        bench.iter(|| {
            let mut stacks = Vec::with_capacity(100);
            for _ in 0..NUM_STACK {
                let (_, stack) = heap.new::<Stack>();
                stacks.push(stack);
            }
            for s in 0..NUM_STACK {
                for i in 0u32..STACK_SIZE {
                    let t = s * 392017 % NUM_STACK;
                    stacks[t].push(i, &heap);
                }
            }
            heap.clear();
        });
    }
}