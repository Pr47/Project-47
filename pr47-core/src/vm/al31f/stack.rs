use std::ptr::{NonNull, slice_from_raw_parts_mut};

use unchecked_unwrap::UncheckedUnwrap;

use crate::data::Value;

#[cfg(debug_assertions)]
#[derive(Copy, Clone)]
pub struct StackSlice(*mut [Option<Value>]);

#[cfg(debug_assertions)]
impl StackSlice {
    pub unsafe fn set_value(&self, idx: usize, value: Value) {
        (*self.0)[idx].replace(value);
    }

    pub unsafe fn get_value(&self, idx: usize) -> Value {
        (*self.0)[idx].unwrap()
    }

    pub unsafe fn get_value_mut_ref(&self, idx: usize) -> *mut Value {
        (*self.0)[idx].as_mut().unwrap() as *mut Value
    }
}

#[cfg(not(debug_assertions))]
#[derive(Copy, Clone)]
pub struct StackSlice(*mut [Value]);

#[cfg(not(debug_assertions))]
impl StackSlice {
    #[inline(always)] pub unsafe fn set_value(&mut self, idx: usize, value: Value) {
        let dest: &mut Value = (*self.0).get_unchecked_mut(idx);
        *dest = value;
    }

    #[inline(always)] pub unsafe fn get_value(&mut self, idx: usize) -> Value {
        let src: &Value = (*self.0).get_unchecked(idx);
        *src
    }

    #[inline(always)] pub unsafe fn get_value_mut_ref(&self, idx: usize) -> *mut Value {
        (*self.0).get_unchecked_mut(idx)
    }
}

#[derive(Debug)]
pub struct FrameInfo {
    pub frame_start: usize,
    pub frame_end: usize,
    pub ret_value_locs: NonNull<[usize]>,
    pub ret_addr: usize,

    pub func_id: usize
}

impl FrameInfo {
    pub fn new(
        frame_start: usize,
        frame_end: usize,
        ret_value_locs: NonNull<[usize]>,
        ret_addr: usize,
        func_id: usize
    ) -> Self {
        Self {
            frame_start,
            frame_end,
            ret_value_locs,
            ret_addr,
            func_id
        }
    }
}

#[cfg(debug_assertions)]
pub struct Stack {
    pub values: Vec<Option<Value>>,
    pub frames: Vec<FrameInfo>
}

pub const EMPTY_RET_LOCS_SLICE: &'static [usize] = &[];

#[cfg(debug_assertions)]
impl Stack {
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(64),
            frames: Vec::with_capacity(4)
        }
    }

    pub unsafe fn ext_func_call_grow_stack(
        &mut self,
        func_id: usize,
        frame_size: usize,
        args: &[Value]
    ) -> StackSlice {
        assert_eq!(self.values.len(), 0);
        assert_eq!(self.frames.len(), 0);

        self.values.resize(frame_size, None);
        for (i /*: usize*/, arg /*: &Value*/) in args.iter().enumerate() {
            self.values[i].replace(*arg);
        }
        self.frames.push(FrameInfo::new(
            0, frame_size, NonNull::from(EMPTY_RET_LOCS_SLICE), 0, func_id
        ));
        StackSlice(&mut self.values[..] as *mut [Option<Value>])
    }

    pub unsafe fn func_call_grow_stack(
        &mut self,
        func_id: usize,
        frame_size: usize,
        arg_locs: &[usize],
        ret_value_locs: NonNull<[usize]>,
        ret_addr: usize
    ) -> StackSlice {
        let this_frame: &FrameInfo = self.frames.last().unwrap();
        let (this_frame_start, this_frame_end): (usize, usize)
            = (this_frame.frame_start, this_frame.frame_end);

        assert_eq!(this_frame_end, self.values.len());
        let new_frame_end: usize = this_frame_end + frame_size;
        self.values.resize(new_frame_end, None);
        self.frames.push(
            FrameInfo::new(this_frame_end, new_frame_end, ret_value_locs, ret_addr, func_id)
        );
        let old_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_start..this_frame_end] as *mut _);
        let new_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_end..new_frame_end] as *mut _);
        for (i /*: usize*/, arg_loc/*: &usize*/) in arg_locs.iter().enumerate() {
            new_slice.set_value(i, old_slice.get_value(*arg_loc));
        }
        new_slice
    }

    pub unsafe fn done_func_call_shrink_stack0(&mut self) -> Option<(StackSlice, usize)> {
        self.done_func_call_shrink_stack(&[])
    }

    pub unsafe fn done_func_call_shrink_stack1(
        &mut self,
        ret_value_src: usize
    ) -> Option<(StackSlice, usize)> {
        self.done_func_call_shrink_stack(&[ret_value_src])
    }

    pub unsafe fn done_func_call_shrink_stack(
        &mut self,
        ret_values: &[usize]
    ) -> Option<(StackSlice, usize)> {
        let frame_count: usize = self.frames.len();
        if frame_count == 1 {
            return None;
        }

        let this_frame: &FrameInfo = &self.frames[frame_count - 1];
        let prev_frame: &FrameInfo = &self.frames[frame_count - 2];
        assert_eq!(prev_frame.frame_end, this_frame.frame_start);
        let this_slice =
            StackSlice(&mut self.values[this_frame.frame_start..this_frame.frame_end] as *mut _);
        let prev_slice =
            StackSlice(&mut self.values[prev_frame.frame_start..prev_frame.frame_end] as *mut _);

        assert_eq!(ret_values.len(), this_frame.ret_value_locs.as_ref().len());
        for (ret_value /*: &usize*/, ret_value_loc /*: &usize*/) in
            ret_values.iter().zip(this_frame.ret_value_locs.as_ref().iter())
        {
            prev_slice.set_value(*ret_value_loc, this_slice.get_value(*ret_value))
        }

        let ret_addr: usize = this_frame.ret_addr;
        self.values.truncate(prev_frame.frame_end);
        self.frames.pop().unwrap();
        Some((prev_slice, ret_addr))
    }

    pub unsafe fn last_frame_slice(&mut self) -> StackSlice {
        let frame: &FrameInfo = self.frames.last().unchecked_unwrap();
        StackSlice(&mut self.values[frame.frame_start..frame.frame_end] as *mut _)
    }

    pub unsafe fn unwind_shrink_slice(&mut self) {
        let frame: FrameInfo = self.frames.pop().unchecked_unwrap();
        self.values.truncate(frame.frame_start);
    }
}

#[cfg(any(test, feature = "bench"))]
impl Stack {
    pub fn trace(&self) {
        eprintln!("[STACK-TRACE] Begin stack tracing");
        eprintln!("[STACK-TRACE] {{");
        for (i, frame) /*: (usize, &FrameInfo)*/ in self.frames.iter().enumerate() {
            eprintln!("[STACK-TRACE]     <frame {}: size = {}, ret_addr = {}, ret_val_locs = {:?}>",
                      i,
                      frame.frame_end - frame.frame_start,
                      frame.ret_addr,
                      unsafe { frame.ret_value_locs.as_ref() });
            eprintln!("[STACK-TRACE]     [");
            #[cfg(debug_assertions)]
            for i /*: usize*/ in frame.frame_start..frame.frame_end {
                if let Some(value /*: &Value*/) = &self.values[i] {
                    eprintln!("[STACK-TRACE]         [{}] = {:?}", i - frame.frame_start, value);
                } else {
                    eprintln!("[STACK-TRACE]         [{}] = UNINIT", i - frame.frame_start);
                }
            }
            #[cfg(not(debug_assertions))]
            for i /*: usize*/ in frame.frame_start..frame.frame_end {
                let value: &Value = &self.values[i];
                eprintln!("[STACK-TRACE]         [{}] = {:?}", i - frame.frame_start, value);
            }
            eprintln!("[STACK-TRACE]     ]");
        }
        eprintln!("[STACK-TRACE] }}");
        eprintln!("[STACK-TRACE] End stack tracing");
    }
}

#[cfg(not(debug_assertions))]
pub struct Stack {
    pub values: Vec<Value>,
    pub frames: Vec<FrameInfo>
}

#[cfg(not(debug_assertions))]
impl Stack {
    pub fn new() -> Self {
        Self {
            values: Vec::with_capacity(64),
            frames: Vec::with_capacity(4)
        }
    }

    pub unsafe fn ext_func_call_grow_stack(
        &mut self,
        func_id: usize,
        frame_size: usize,
        args: &[Value]
    ) -> StackSlice {
        self.values.resize(frame_size, Value::new_null());
        for (i /*: usize*/, arg /*: &Value*/) in args.iter().enumerate() {
            let dest: &mut Value = self.values.get_unchecked_mut(i);
            *dest = *arg;
        }
        self.frames.push(FrameInfo::new(
            0, frame_size, NonNull::from(EMPTY_RET_LOCS_SLICE), 0, func_id)
        );
        StackSlice(&mut self.values[..] as *mut _)
    }

    pub unsafe fn func_call_grow_stack(
        &mut self,
        func_id: usize,
        frame_size: usize,
        arg_locs: &[usize],
        ret_value_locs: NonNull<[usize]>,
        ret_addr: usize
    ) -> StackSlice {
        let this_frame: &FrameInfo = self.frames.last().unchecked_unwrap();
        let (this_frame_start, this_frame_end): (usize, usize)
            = (this_frame.frame_start, this_frame.frame_end);
        let new_frame_end: usize = this_frame_end + frame_size;
        self.values.resize(new_frame_end, Value::new_null());
        self.frames.push(
            FrameInfo::new(this_frame_end, new_frame_end, ret_value_locs, ret_addr, func_id)
        );
        let mut old_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_start..this_frame_end] as *mut _);
        let mut new_slice: StackSlice =
            StackSlice(&mut self.values[this_frame_end..new_frame_end] as *mut _);

        for i /*: usize*/ in 0..arg_locs.len() {
            let arg_loc: usize = *arg_locs.get_unchecked(i);
            new_slice.set_value(i, old_slice.get_value(arg_loc));
        }

        new_slice
    }

    #[inline] pub unsafe fn done_func_call_shrink_stack0(&mut self) -> Option<(StackSlice, usize)> {
        let frame_count = self.frames.len();
        if frame_count == 1 {
            return None;
        }

        let this_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 1);
        let prev_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 2);
        let prev_slice: StackSlice =
            StackSlice(&mut self.values[prev_frame.frame_start..prev_frame.frame_end] as *mut _);

        let ret_addr: usize = this_frame.ret_addr;
        self.values.truncate(prev_frame.frame_end);
        self.frames.pop().unchecked_unwrap();
        Some((prev_slice, ret_addr))
    }

    #[inline] pub unsafe fn done_func_call_shrink_stack1(
        &mut self,
        ret_value_src: usize
    ) -> Option<(StackSlice, usize)> {
        let frame_count = self.frames.len();
        if frame_count == 1 {
            return None;
        }

        let this_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 1);
        let prev_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 2);

        let this_slice_ptr = self.values.as_ptr().offset(this_frame.frame_start as isize);
        let prev_slice_ptr = self.values.as_mut_ptr().offset(prev_frame.frame_start as isize);
        let prev_slice = slice_from_raw_parts_mut(
            prev_slice_ptr,
            prev_frame.frame_end - prev_frame.frame_start
        );

        let ret_value_loc: usize = *this_frame.ret_value_locs.as_ref().get_unchecked(0);
        *prev_slice_ptr.offset(ret_value_loc as isize)
            = *this_slice_ptr.offset(ret_value_src as isize);

        let ret_addr: usize = this_frame.ret_addr;
        self.values.truncate(prev_frame.frame_end);
        self.frames.pop().unchecked_unwrap();
        Some((StackSlice(prev_slice), ret_addr))
    }

    pub unsafe fn done_func_call_shrink_stack(
        &mut self,
        ret_values: &[usize]
    ) -> Option<(StackSlice, usize)> {
        let frame_count = self.frames.len();
        if frame_count == 1 {
            return None;
        }

        let this_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 1);
        let prev_frame: &FrameInfo = self.frames.get_unchecked(frame_count - 2);
        let mut this_slice: StackSlice =
            StackSlice(&mut self.values[this_frame.frame_start..this_frame.frame_end] as *mut _);
        let mut prev_slice: StackSlice =
            StackSlice(&mut self.values[prev_frame.frame_start..prev_frame.frame_end] as *mut _);

        let len: usize = ret_values.len();
        for i /*: usize*/ in 0..len {
            let ret_value_loc: usize = *this_frame.ret_value_locs.as_ref().get_unchecked(i);
            let ret_value_src: usize = *ret_values.get_unchecked(i);
            prev_slice.set_value(ret_value_loc, this_slice.get_value(ret_value_src));
        }

        let ret_addr: usize = this_frame.ret_addr;
        self.values.truncate(prev_frame.frame_end);
        self.frames.pop().unchecked_unwrap();
        Some((prev_slice, ret_addr))
    }

    #[inline] pub unsafe fn last_frame_slice(&mut self) -> StackSlice {
        let frame: &FrameInfo = self.frames.last().unchecked_unwrap();
        StackSlice(&mut self.values[frame.frame_start..frame.frame_end] as *mut _)
    }

    #[inline] pub unsafe fn unwind_shrink_slice(&mut self) {
        let frame: FrameInfo = self.frames.pop().unchecked_unwrap();
        self.values.truncate(frame.frame_start);
    }
}
