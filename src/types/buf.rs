use std::alloc::{alloc, dealloc, Layout};

pub struct Buf {
    raw: *mut u8,
    layout: Layout,
    len: usize
}

impl Buf {
    pub fn new(size: usize) -> Option<Buf> {
        unsafe {
            let layout = Layout::from_size_align(size, 8);
            match layout {
                Ok(layout) => {
                    let ptr = alloc(layout);
                    Some(Buf { raw: ptr as *mut u8, layout: layout, len: size})
                },
                Err(e) => {
                    println!("create layout failed: {:?}", e);
                    return None;
                }
            }
        }
    }

    pub fn from_vec(vec: &Vec<u8>) -> Option<Buf> {
        let data_len = vec.len();
        let buf = Buf::new(data_len);
        match buf {
            Some(buf) => {
                let ptr = buf.get_raw_mut();
                for i in 0..data_len {
                    unsafe {
                        *ptr = vec[i];
                    }
                }
                Some(buf)
            }
            None => None
        }

    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get_data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.raw, self.len())
        }
    }

    pub fn get_raw_mut(&mut self) -> *mut u8 {
        self.raw
    }

    pub fn release(&mut self) {
        unsafe {
            dealloc(self.raw, self.layout)
        }
    }
}

unsafe impl Sync for Buf {}

unsafe impl Send for Buf {}

//impl Drop for Buf {
//    fn drop(&mut self) {
//        unsafe {
//            dealloc(self.raw, self.layout)
//        }
//    }
//}