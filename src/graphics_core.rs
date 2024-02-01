use alloc::vec::Vec;
use crate::Ili9341;
use embedded_graphics_core::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
    primitives::Rectangle,
};

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
            let x0 = drawable_area.top_left.x as u16;
            let y0 = drawable_area.top_left.y as u16;
            let x1 = drawable_bottom_right.x as u16;
            let y1 = drawable_bottom_right.y as u16;

            const HLINES: usize = 8;
            let line = x1 - x0;
            let rows = (y1 - y0).max(HLINES as u16);
            let mut line_buffer: Vec<u16> = Vec::with_capacity((line * rows) as usize);

            if area == &drawable_area {
                let mut i = 0;
                for (point, color) in area.points().zip(colors) {
                    let c = RawU16::from(color).into_inner().to_be();
                    line_buffer.push(c);
                    i = i + 1;

                    if (i == line * rows) {
                        // All pixels are on screen
                        self.draw_raw_slice_ne(
                            x0,
                            y0,
                            x1,
                            y1,
                            line_buffer.as_mut_slice(),
                        ).unwrap();

                        line_buffer.clear();
                        i = 0;
                    }
                }
                Ok(())
            } else {
                let mut i = 0;
                for (point, color) in area.points().zip(colors).filter(|(point, _)| drawable_area.contains(*point)) {
                    let c = RawU16::from(color).into_inner().to_be();
                    line_buffer.push(c);
                    i = i + 1;
                    if (i == line * rows) {
                        // All pixels are on screen
                        self.draw_raw_slice_ne(
                            x0,
                            y0,
                            x1,
                            y1,
                            line_buffer.as_mut_slice(),
                        ).unwrap();

                        line_buffer.clear();
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
