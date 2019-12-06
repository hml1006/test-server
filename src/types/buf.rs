use std::alloc::{alloc, dealloc, Layout};

pub struct Buf {
    raw: *mut u8,
    layout: Layout
}

impl Buf {
    fn new(size: usize) -> Option<Buf> {
        unsafe {
            let layout = Layout::from_size_align(size, 8);
            match layout {
                Ok(layout) => {
                    let ptr = alloc(layout);
                    Some(Buf { raw: ptr as *mut u8, layout: layout})
                },
                Err(e) => {
                    println!("create layout failed: {:?}", e);
                    return None;
                }
            }
        }
    }
}

impl AsMut for Buf {
    fn as_mut(&mut self) -> &mut u8 {
        unimplemented!()
    }
}

impl Drop for Buf {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.raw, self.layout)
        }
    }
}