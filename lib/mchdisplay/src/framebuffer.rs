use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    pixelcolor::Rgb565,
    prelude::{IntoStorage, Pixel, Size},
};

use display_interface::DisplayError;


type Result<T = (), E = DisplayError> = core::result::Result<T, E>;

pub trait DrawRawSlice {
    fn draw_raw_slice(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, data: &[u16]) -> Result;
}


struct DirtyArea {
    xmin: u16,
    xmax: u16, // exclusive
    ymin: u16,
    ymax: u16, // exclusive
}

impl DirtyArea {
    fn new() -> Self {
        let mut area = Self { xmin: 0, xmax: 0, ymin: 0, ymax: 0 };
        Self::clear(&mut area);
        area
    }

    fn x0(&self) -> u16 { self.xmin }
    fn y0(&self) -> u16 { self.ymin }
    fn width(&self) -> u16 { self.xmax - self.xmin }
    fn height(&self) -> u16 { self.ymax - self.ymin }

    fn clear(&mut self) {
        self.xmax = 0
    }

    fn is_dirty(&self) -> bool {
        self.xmax != 0
    }

    fn mark_dirty(&mut self, x: u16, y: u16) {
        match self.is_dirty() {
            true => {
                self.xmin = self.xmin.min(x);
                self.xmax = self.xmax.max(x + 1);
                self.ymin = self.ymin.min(y);
                self.ymax = self.ymax.max(y + 1);
            },
            false => {
                self.xmin = x;
                self.xmax = x + 1;
                self.ymin = y;
                self.ymax = y + 1;
            },
        }
    }
}


pub struct Framebuffer<const WIDTH: u16, const HEIGHT: u16> {
    // 320x240*2 == 150KiB, which is very very very much of the limited memory we have. We
    // definitely cannot allocate much more.
    // When updating the screen, we have to choose between:
    // - Doing contiguous updates (i.e. ignore dirty width and use full width).
    // - Doing N updates for N rows, so that we can use slices for each row.
    // We need it in a Box. Otherwise this would end up on the heap
    // where it definitely doesn't fit, and crashes the app before we
    // even start.
    current: Box<[u16]>,
    dirty_area: DirtyArea,
}

impl<const WIDTH: u16, const HEIGHT: u16> Framebuffer<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self {
            current: vec![0u16; (WIDTH as usize) * (HEIGHT as usize)].into_boxed_slice(),
            dirty_area: DirtyArea::new(),
        }
    }

    fn index(x: u16, y: u16) -> usize {
        (y as usize) * (WIDTH as usize) + (x as usize)
    }

    fn mark_dirty(&mut self, x: u16, y: u16) {
        self.dirty_area.mark_dirty(x, y)
    }

    pub fn flush<D>(&mut self, display: &mut D) -> Result
    where
        D: DrawRawSlice,
    {
        if self.dirty_area.is_dirty() {
            #[allow(dead_code)]
            enum DrawMethod {
                // Forget about xmin/xmax and draw all dirty lines. We have contiguous memory and
                // can draw with one command.
                // Test timings:
                // - 167 ms - full screen update
                // -  40 ms - small text move down
                // -  59 ms - text moved up a few lines
                Contiguous,
                // Draw slices for each line.
                // Test timings:
                // - 206 ms - full screen update
                // -  40 ms - small text move down
                // -  63 ms - text moved up a few lines
                LineSlices,
                // Conclusion, only in rare cases (a vertical line)
                // would it make sense to use LineSlices.
            }

            // The unused LineSlices is kept for reference only.
            let draw_method = DrawMethod::Contiguous;

            let x0: u16;
            let y0: u16 = self.dirty_area.y0();
            let w: u16;
            let h: u16 = self.dirty_area.height();

            match draw_method {
                DrawMethod::Contiguous => {
                    x0 = 0;
                    w = WIDTH;
                },
                DrawMethod::LineSlices => {
                    x0 = self.dirty_area.x0();
                    w = self.dirty_area.width();
                }
            };

            assert!(x0 < WIDTH);
            assert!(y0 < HEIGHT);
            assert!(w > 0 && w <= WIDTH);
            assert!(h > 0 && h <= HEIGHT);

            match draw_method {
                DrawMethod::Contiguous => {
                    let start = Self::index(x0, y0);
                    let end = Self::index(x0 + w - 1, y0 + h - 1);
                    let slice = &self.current[start..end + 1];
                    display.draw_raw_slice(x0, y0, x0 + w - 1, y0 + h - 1, &slice)?;
                },
                DrawMethod::LineSlices => {
                    for y in y0..y0 + h {
                        let start = Self::index(x0, y);
                        let end = Self::index(x0 + w - 1, y);
                        let slice = &self.current[start..end + 1];
                        display.draw_raw_slice(x0, y, x0 + w - 1, y, &slice)?;
                    }
                }
            }

            self.dirty_area.clear();
        }
        Ok(())
    }
}

impl<const WIDTH: u16, const HEIGHT: u16> DrawTarget for Framebuffer<WIDTH, HEIGHT> {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> core::result::Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            if x < 0 || y < 0 || (x >= WIDTH as i32) || y >= (HEIGHT as i32) {
                continue;
            }
            let idx = Self::index(x as u16, y as u16);
            let raw = color.into_storage();
            if self.current[idx] != raw {
                self.current[idx] = raw;
                self.mark_dirty(x as u16, y as u16);
            }
        }
        Ok(())
    }
}

impl<const WIDTH: u16, const HEIGHT: u16> OriginDimensions for Framebuffer<WIDTH, HEIGHT> {
    fn size(&self) -> Size {
        (WIDTH as u32, HEIGHT as u32).into()
    }
}
