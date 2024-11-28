use prism::compute::kernel::*;
use prism::prelude::*;

#[derive(Default)]
pub struct MandelbrotKernel {
    pub scale: Vector2,
    pub offset: Vector2,
    pub max_iter: Real,
    pub smoothbrot: bool,
}

impl Kernel<Vector4, TexelBuffer<Vector4>> for MandelbrotKernel {
    fn kernel_exec(&self, input: KernelInput, _buffer: &TexelBuffer<Vector4>) -> Vector4 {
        let u = {
            let u = input.thread_x as Real / (input.buffer_width as Real - Real::ONE);
            (u * 2.0) - 1.0
        };

        let v = {
            let v = input.thread_y as Real / (input.buffer_height as Real - Real::ONE);
            (v * 2.0) - 1.0
        };

        let texel_x = 1.0 / input.buffer_width as Real;
        let texel_y = 1.0 / input.buffer_height as Real;

        let mut sum = Vector3::default();

        let samples = MSAASample::get_offsets(MSAASample::X4);

        for sample in &samples {
            let bx = (u + sample[0] * texel_x) / self.scale.x() + self.offset.x();
            let by = (v + sample[1] * texel_y) / self.scale.y() + self.offset.y();

            let mut ix = 0.0;
            let mut iy = 0.0;
            let mut iter = 0.0;

            if !self.smoothbrot {
                while (ix * ix + iy * iy <= 4.0) && iter < self.max_iter {
                    let f = ix * ix - iy * iy + bx;

                    iy = 2.0 * ix * iy + by;
                    ix = f;

                    iter += 1.0;
                }
            } else {
                while (ix * ix + iy * iy <= 256.0) && iter < self.max_iter {
                    let f = ix * ix - iy * iy + bx;

                    iy = 2.0 * ix * iy + by;
                    ix = f;

                    iter += 1.0;
                }

                if iter < self.max_iter {
                    let log_zn = (ix * ix + iy * iy).rl_log10() / 2.0;
                    let nu = (log_zn / (2.0).rl_log10()).rl_log10() / (2.0).rl_log10();

                    iter += 1.0 - nu;
                }
            }

            let l = iter / self.max_iter;

            let f = 1.0 - l;

            let rgb = Vector3::from_scalar(1.0 - f);

            sum += rgb;
        }

        Vector4::from_w(sum / samples.len() as Real, 1.0)
    }
}
