use crate::compute::buffer::*;

// A kernel is a method of running code on multiple threads
// Minimal and mostly read only data is exposed to allow for safety

/// Kernel input data
///
/// This is used to read / write data accurately
///
/// Also allows for reconstructing UVW coordinates during kernel execution
#[repr(C)]
#[derive(Copy, Clone)]
pub struct KernelInput {
    /// X texel relative to the output buffer
    pub thread_x: usize,
    /// Y texel relative to the output buffer
    pub thread_y: usize,
    /// Z texel relative to the output buffer
    pub thread_z: usize,

    /// Width of output buffer
    ///
    /// Note: `thread_x / buffer_width` is U coordinate
    pub buffer_width: usize,
    /// Height of output buffer
    ///
    /// Note: `thread_y / buffer_height` is V coordinate
    pub buffer_height: usize,
    /// Depth of output buffer
    ///
    /// Note: `thread_z / buffer_depth` is W coordinate
    pub buffer_depth: usize,
}

/// A Kernel used for distributed computation
///
/// Kernel's provide an abstraction layer over operations, safely distributing a workload across multiple threads
///
/// * `TOut` - The output datatype of this kernel
/// * `TB` - The input buffer of this kernel (by default is `Buffer<TIn>`)
/// * `TIn` - The input datatype of this kernel (by default is equal to `TOut`)
pub trait Kernel<TOut: BufferData, TB: Buffer<TIn>, TIn: BufferData = TOut> {
    /// Kernel execution function
    fn kernel_exec(&self, input: KernelInput, buffer: &TB) -> TOut;
}
