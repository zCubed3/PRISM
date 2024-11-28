use prism::compute::kernel::*;
use prism::prelude::*;

#[derive(Default)]
#[doc(hidden)]
pub struct VoxelKernel {}

impl Kernel<Vector4, VoxelBuffer<Vector4>> for VoxelKernel {
    fn kernel_exec(&self, input: KernelInput, _buffer: &VoxelBuffer<Vector4>) -> Vector4 {
        // Liam
        // - My particular LUT application requires V to be flipped, undo it if you don't need that!

        let u = input.thread_x as Real / (input.buffer_width as Real - Real::ONE);
        let v = 1.0 - (input.thread_y as Real / (input.buffer_height as Real - Real::ONE));
        let w = input.thread_z as Real / (input.buffer_depth as Real - Real::ONE);

        Vector4::new(u, v, w, Real::ONE)
    }
}
