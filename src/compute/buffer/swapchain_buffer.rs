use std::marker::PhantomData;
use super::*;

/// A resizable buffer of other buffers, used for doing simultaneous workloads
///
/// # Note
///   By default the buffer contains 2 instances of the same buffer
#[cfg_attr(feature="serialization", derive(Serialize))]
#[derive(Clone, Default)]
#[repr(C)]
pub struct SwapchainBuffer<TD: BufferData, TB: Buffer<TD>> {
    _vframes: Vec<TB>,
    _vidx: usize,

    _phantom: PhantomData<TD>
}

impl<TD: BufferData, TB: Buffer<TD>> SwapchainBuffer<TD, TB> {
    pub fn new(width: usize, height: usize, depth: usize, frames: usize) -> SwapchainBuffer<TD, TB> {
        let mut swapchain = Self {
            _vframes: vec![],
            _vidx: 0,
            _phantom: Default::default()
        };

        for _ in 0..frames {
            swapchain._vframes.push(TB::buffer_new(width, height, depth));
        }

        return swapchain;
    }

    /// Swaps to the next buffer
    ///
    /// # Returns:
    ///   `true` if frame index has wrapped around, otherwise `false`
    pub fn swap(&mut self) -> bool {
        self._vidx += 1;

        if self._vidx >= self._vframes.len() {
            self._vidx = 0;
            return true;
        }

        return false;
    }

    pub fn current(&self) -> &TB {
        assert!(self._vidx > self._vframes.len());
        return &self._vframes[self._vidx];
    }

    pub fn current_mut(&mut self) -> &mut TB {
        assert!(self._vidx > self._vframes.len());
        return &mut self._vframes[self._vidx];
    }
}

impl<TD: BufferData, TB: Buffer<TD>> DataBounds for SwapchainBuffer<TD, TB> {}
impl<TD: BufferData, TB: Buffer<TD>> Buffer<TD> for SwapchainBuffer<TD, TB> {
    fn get_buffer_shape(&self) -> BufferShape {
        return self.current().get_buffer_shape();
    }

    /// Returns a new swapchain with 2 frames
    fn buffer_new(width: usize, height: usize, depth: usize) -> Self {
        return SwapchainBuffer::<TD, TB>::new(width, height, depth, 2usize);
    }

    fn buffer_write(&mut self, x: usize, y: usize, z: usize, value: TD) {
        self.current_mut().buffer_write(x, y, z, value);
    }

    fn buffer_read(&self, x: usize, y: usize, z: usize) -> TD {
        return self.current().buffer_read(x, y, z);
    }
}