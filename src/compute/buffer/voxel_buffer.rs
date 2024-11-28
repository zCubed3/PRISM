use super::*;
use rgml::vector::Vector;

#[cfg(feature = "image")]
use image::*;

/// A voxel (cube) shaped buffer
#[cfg_attr(feature = "serialization", derive(Serialize))]
#[derive(Clone)]
#[repr(C)]
pub struct VoxelBuffer<T: BufferData> {
    _backing: Vec<T>,
    _width: usize,
    _height: usize,
    _depth: usize,
}

impl<T: BufferData> VoxelBuffer<T> {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        return Self {
            _backing: vec![T::default(); width * height * depth],
            _width: width,
            _height: height,
            _depth: depth,
        };
    }

    fn copy_slice(
        &self,
        x: usize,
        y: usize,
        z: usize,
        width: usize,
        height: usize,
        depth: usize,
    ) -> Self {
        let mut slice = Self::buffer_new(width, height, depth);

        for s in 0..width {
            for t in 0..height {
                for q in 0..depth {
                    let copy = self._backing[self.coord_to_index(x + s, y + t, z + q)];
                    slice._backing[s + t * width + q * width * height] = copy;
                }
            }
        }

        return slice;
    }

    fn coord_to_index(&self, x: usize, y: usize, z: usize) -> usize {
        return x + (y * self._width) + (z * self._width * self._height);
    }
}

impl<T: BufferData> DataBounds for VoxelBuffer<T> {}
impl<T: BufferData> Buffer<T> for VoxelBuffer<T> {
    fn get_buffer_shape(&self) -> BufferShape {
        return BufferShape::Shape3D;
    }

    fn get_buffer_width(&self) -> usize {
        return self._width;
    }

    fn get_buffer_height(&self) -> usize {
        return self._height;
    }

    fn get_buffer_depth(&self) -> usize {
        return self._depth;
    }

    fn buffer_new(width: usize, height: usize, depth: usize) -> Self {
        return VoxelBuffer::new(width, height, depth);
    }

    fn buffer_write(&mut self, x: usize, y: usize, z: usize, value: T) {
        let index = self.coord_to_index(x, y, z);
        assert!(index < self._width * self._height * self._depth);
        self._backing[index] = value;
    }

    fn buffer_read(&self, x: usize, y: usize, z: usize) -> T {
        let index = self.coord_to_index(x, y, z);
        assert!(index < self._width * self._height * self._depth);
        return self._backing[index];
    }
}

impl<T: BufferData + LerpableData> VoxelBuffer<T> {
    /// Trilinearly interpolates this buffer
    pub fn trilinear_sample(&self, u: Real, v: Real, w: Real) -> T {
        // TODO: Wrapping?
        let u = u.rl_saturate();
        let v = v.rl_saturate();
        let w = w.rl_saturate();

        let width = self.get_buffer_width() as Real - 1.0;
        let height = self.get_buffer_height() as Real - 1.0;
        let depth = self.get_buffer_depth() as Real - 1.0;

        // Texels
        let nt_x_l = (u * width).rl_floor();
        let nt_x_h = (u * width).rl_ceil();

        let nt_y_l = (v * height).rl_floor();
        let nt_y_h = (v * height).rl_ceil();

        let nt_z_l = (w * depth).rl_floor();
        let nt_z_h = (w * depth).rl_ceil();

        // Texel distance
        let nt_x_d = nt_x_h - (u * width);
        let nt_y_d = nt_y_h - (v * height);
        let nt_z_d = nt_z_h - (w * depth);

        // Lookup values
        let lv_ful = self.buffer_read(nt_x_l as usize, nt_y_h as usize, nt_z_l as usize);
        let lv_fur = self.buffer_read(nt_x_h as usize, nt_y_h as usize, nt_z_l as usize);
        let lv_fll = self.buffer_read(nt_x_l as usize, nt_y_l as usize, nt_z_l as usize);
        let lv_flr = self.buffer_read(nt_x_h as usize, nt_y_l as usize, nt_z_l as usize);

        let lv_bul = self.buffer_read(nt_x_l as usize, nt_y_h as usize, nt_z_h as usize);
        let lv_bur = self.buffer_read(nt_x_h as usize, nt_y_h as usize, nt_z_h as usize);
        let lv_bll = self.buffer_read(nt_x_l as usize, nt_y_l as usize, nt_z_h as usize);
        let lv_blr = self.buffer_read(nt_x_h as usize, nt_y_l as usize, nt_z_h as usize);

        // Final lookups
        let lv_fui = lv_fur.linear_interpolate(lv_ful, nt_x_d);
        let lv_fli = lv_flr.linear_interpolate(lv_fll, nt_x_d);
        let lv_bui = lv_bur.linear_interpolate(lv_bul, nt_x_d);
        let lv_bli = lv_blr.linear_interpolate(lv_bll, nt_x_d);

        let lv_fi = lv_fui.linear_interpolate(lv_fli, nt_y_d);
        let lv_bi = lv_bui.linear_interpolate(lv_bli, nt_y_d);

        return lv_bi.linear_interpolate(lv_fi, nt_z_d);
    }
}

#[cfg(feature = "image")]
impl VoxelBuffer<Vector<Real, 4>> {
    /// Saves this [VoxelBuffer] as a colored image
    pub fn save_as_image<P: AsRef<Path>>(&self, path: P) {
        let width = self._width * self._depth;
        let mut image = RgbaImage::new(width as u32, self._height as u32);

        for z in 0..self._depth {
            let shift = z * self._width;
            for x in 0..self._width {
                for y in 0..self._height {
                    let wx = x + shift;

                    let raw_color = self.buffer_read(x, y, z);
                    image.put_pixel(
                        wx as u32,
                        y as u32,
                        Rgba([
                            (raw_color[0].rl_saturate() * 255.0).rl_round() as u8,
                            (raw_color[1].rl_saturate() * 255.0).rl_round() as u8,
                            (raw_color[2].rl_saturate() * 255.0).rl_round() as u8,
                            (raw_color[3].rl_saturate() * 255.0).rl_round() as u8,
                        ]),
                    );
                }
            }
        }

        image.save(path).expect("Failed to save image");
    }
}
