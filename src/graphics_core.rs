use alloc::vec::Vec;

use embedded_graphics_core::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
    primitives::Rectangle,
};
use crate::Ili9341;

impl<IFACE, RESET> OriginDimensions for Ili9341<IFACE, RESET> {
    fn size(&self) -> Size {
        Size::new(self.width() as u32, self.height() as u32)
    }
}

impl<IFACE, RESET> DrawTarget for Ili9341<IFACE, RESET>
where
    IFACE: display_interface::WriteOnlyDataCommand,
{
    type Error = display_interface::DisplayError;

    type Color = Rgb565;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if self.bounding_box().contains(point) {
                let x = point.x as u16;
                let y = point.y as u16;

                self.draw_raw_iter(
                    x,
                    y,
                    x,
                    y,
                    core::iter::once(RawU16::from(color).into_inner()),
                )?;
            }
        }
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let drawable_area = area.intersection(&self.bounding_box());

        if let Some(drawable_bottom_right) = drawable_area.bottom_right() {
            let x0 = drawable_area.top_left.x as u32;
            let y0 = drawable_area.top_left.y as u32;
            let x1 = drawable_bottom_right.x as u32;
            let y1 = drawable_bottom_right.y as u32;

            const HLINES: u32 = 16u32;
            let area_lines = x1 - x0 + 1;
            let area_rows = y1 - y0 + 1;
            let buffer_rows = HLINES;
            let area_points = area_lines * area_rows;
            let buffer_points = area_lines * buffer_rows;
            let mut line_buffer: Vec<u16> = Vec::with_capacity(buffer_points as usize);

            if area == &drawable_area {
                let mut i = 0u32;
                let mut n = 0u32;
                let mut lines_count = 0;
                for (point, color) in area.points().zip(colors) {
                    let c = RawU16::from(color).into_inner().to_be();
                    line_buffer.push(c);
                    i = i + 1;
                    n = n + 1;

                    if (i == buffer_points || n == area_points) {
                        // All pixels are on screen
                        self.draw_raw_slice_ne(
                            x0 as u16,
                            (y0 + lines_count * buffer_rows).min(y1) as u16,
                            x1 as u16,
                            (y0 + lines_count * buffer_rows + buffer_rows).min(y1) as u16,
                            line_buffer.as_mut_slice(),
                        ).unwrap();

                        line_buffer.clear();
                        lines_count = lines_count + 1;
                        i = 0;
                    }
                }
                Ok(())
            } else {
                let mut i = 0u32;
                let mut lines_count = 0;
                for (point, color) in area.points().zip(colors).filter(|(point, _)| drawable_area.contains(*point)) {
                    let c = RawU16::from(color).into_inner().to_be();
                    line_buffer.push(c);
                    i = i + 1;
                    let mut n = 0u32;

                    if (i == buffer_points || n == area_points) {
                        // All pixels are on screen
                        self.draw_raw_slice_ne(
                            x0 as u16,
                            (y0 + lines_count * buffer_rows).min(y1) as u16,
                            x1 as u16,
                            (y0 + lines_count * buffer_rows + buffer_rows).min(y1) as u16,
                            line_buffer.as_mut_slice(),
                        ).unwrap();

                        line_buffer.clear();
                        lines_count = lines_count + 1;
                        i = 0;
                    }
                }
                Ok(())
            }
        } else {
            // No pixels are on screen
            Ok(())
        }
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.clear_screen(color)
    }
}
