use prism::compute::kernel::*;
use prism::prelude::*;

#[derive(Default)]
#[doc(hidden)]
pub struct VoxelKernel {}

impl Kernel<Vector4, VoxelBuffer<Vector4>> for VoxelKernel {
    fn kernel_exec(&self, input: KernelInput, _buffer: &VoxelBuffer<Vector4>) -> Vector4 {
        let mut u = input.thread_x as Real / (input.buffer_width as Real - Real::ONE);
        let mut v = input.thread_y as Real / (input.buffer_height as Real - Real::ONE);
        let mut w = input.thread_z as Real / (input.buffer_depth as Real - Real::ONE);

        // Liam
        // - My particular LUT application requires V to be flipped, undo it if you don't need that!

        v = 1.0 - v;

        //https://x.com/iquilezles/status/1440847977560494084

        u = u.rl_pow(3.0 / 2.0);
        v = v.rl_pow(4.0 / 5.0);
        w = w.rl_pow(3.0 / 2.0);

        Vector4::new(u, v, w, Real::ONE)
    }
}
