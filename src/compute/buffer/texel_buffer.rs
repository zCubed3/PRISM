use super::*;
use rgml::vector::Vector;

#[cfg(feature = "image")]
use image::*;

/// A texel (square) shaped buffer
#[cfg_attr(feature = "serialization", derive(Serialize))]
#[derive(Clone)]
#[repr(C)]
pub struct TexelBuffer<T: BufferData> {
    _backing: Vec<T>,
    _width: usize,
    _height: usize,
}

impl<T: BufferData> TexelBuffer<T> {
    pub fn new(width: usize, height: usize) -> Self {
        return Self {
            _backing: vec![T::default(); width * height],
            _width: width,
            _height: height,
        };
    }

    fn copy_slice(&self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let mut slice = Self::buffer_new(width, height, 1);

        for s in 0..width {
            for t in 0..height {
                slice._backing[s + t * width] = self._backing[self.coord_to_index(x + s, y + t)];
            }
        }

        return slice;
    }

    fn coord_to_index(&self, x: usize, y: usize) -> usize {
        return x + (y * self._width);
    }
}

impl<T: BufferData + LerpableData> TexelBuffer<T> {
    /// Bilinearly interpolates this buffer
    pub fn bilinear_sample(&self, u: Real, v: Real) -> T {
        let u = u.rl_saturate();
        let v = v.rl_saturate();

        let width = self.get_buffer_width() as Real - 1.0;
        let height = self.get_buffer_height() as Real - 1.0;

        // Texels
        let nt_x_l = (u * width).rl_floor();
        let nt_x_h = (u * width).rl_ceil();

        let nt_y_l = (v * height).rl_floor();
        let nt_y_h = (v * height).rl_ceil();

        // Texel distance
        let nt_x_d = nt_x_h - (u * width);
        let nt_y_d = nt_y_h - (v * height);

        // Lookup values
        let lv_ul = self.buffer_read(nt_x_l as usize, nt_y_h as usize, 0);
        let lv_ur = self.buffer_read(nt_x_h as usize, nt_y_h as usize, 0);
        let lv_ll = self.buffer_read(nt_x_l as usize, nt_y_l as usize, 0);
        let lv_lr = self.buffer_read(nt_x_h as usize, nt_y_l as usize, 0);

        // Final lookup
        let lv_ui = lv_ur.linear_interpolate(lv_ul, nt_x_d);
        let lv_li = lv_lr.linear_interpolate(lv_ll, nt_x_d);

        return lv_ui.linear_interpolate(lv_li, nt_y_d);
    }
}

impl<T: BufferData> DataBounds for TexelBuffer<T> {}
impl<T: BufferData> Buffer<T> for TexelBuffer<T> {
    fn get_buffer_shape(&self) -> BufferShape {
        return BufferShape::Shape2D;
    }

    fn get_buffer_width(&self) -> usize {
        return self._width;
    }

    fn get_buffer_height(&self) -> usize {
        return self._height;
    }

    fn buffer_new(width: usize, height: usize, _depth: usize) -> Self {
        return TexelBuffer::new(width, height);
    }

    fn buffer_write(&mut self, x: usize, y: usize, _z: usize, value: T) {
        let index = self.coord_to_index(x, y);
        assert!(index < self._width * self._height);
        self._backing[index] = value;
    }

    fn buffer_read(&self, x: usize, y: usize, _z: usize) -> T {
        let index = self.coord_to_index(x, y);
        assert!(index < self._width * self._height);
        return self._backing[index];
    }
}

#[cfg(feature = "image")]
impl TexelBuffer<Vector<Real, 4>> {
    /// Saves this [TexelBuffer] as a colored image
    pub fn save_as_image<P: AsRef<Path>>(&self, path: P) {
        let mut image = RgbaImage::new(self._width as u32, self._height as u32);

        for x in 0..self._width {
            for y in 0..self._height {
                let raw_color = self.buffer_read(x, y, 0usize);
                image.put_pixel(
                    x as u32,
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

        image.save(path).expect("Failed to save image");
    }
}
