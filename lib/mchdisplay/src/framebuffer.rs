#[cfg(feature = "with-psram")]
use core::{mem, slice};

#[cfg(feature = "with-psram")]
use esp_idf_svc::sys::{heap_caps_malloc, MALLOC_CAP_SPIRAM, MALLOC_CAP_8BIT};

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


/// Allocates a buffer of `W` * `H` elements of type `T`, using PSRAM if enabled.
///
/// NOTE: The vector is single dimensional even though two size
/// parameters are provided.
fn allocate_box<T: Default + Clone, const W: u16, const H: u16>() -> Box<[T]> {
    let n: usize = (W as usize * H as usize).into(); // compiler won't allow const here

    #[cfg(feature = "with-psram")]
    unsafe {
        let size = n * mem::size_of::<T>();
        let raw = heap_caps_malloc(size, MALLOC_CAP_SPIRAM | MALLOC_CAP_8BIT) as *mut T;
        if raw.is_null() {
            panic!("Failed to allocate framebuffer in PSRAM");
        }
        let slice = slice::from_raw_parts_mut(raw, n);
        Box::from_raw(slice)
    }

    #[cfg(not(feature = "with-psram"))]
    {
        vec![T::default(); n].into_boxed_slice()
    }
}


pub struct Framebuffer<const WIDTH: u16, const HEIGHT: u16> {
    // For one buffer, we need 320x240*2 == 150KiB. for two buffers, we
    // need double that. If we have PSRAM, we can do that.
    //
    // When updating the screen, we have to choose between:
    // - Doing contiguous updates (i.e. ignore dirty width and use full width).
    // - Doing N updates for N rows, so that we can use slices for each row.
    // - Writing to a temporary buffer and update a smaller rectangle of
    //   the screen. This option requires double memory.
    // We need it in a Box. Otherwise this would end up on the stack
    // where it definitely doesn't fit, and crashes the app before we
    // even start.
    current: Box<[u16]>,
    partial: Option<Box<[u16]>>,
    dirty_area: DirtyArea,
}

impl<const WIDTH: u16, const HEIGHT: u16> Framebuffer<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self {
            current: allocate_box::<u16, WIDTH, HEIGHT>(),
            partial: None,
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
                // Forget about xmin/xmax and draw all dirty lines. We
                // have contiguous memory and can draw with one command.
                // Test timings:
                // - 140 ms - full screen update
                // -  34 ms - small text move down
                // -  50 ms - text moved up a few lines
                Contiguous,
                // Draw slices for each line.
                // Test timings:
                // - 168 ms - full screen update
                // -  34 ms - small text move down
                // -  52 ms - text moved up a few lines
                // Only in rare cases (a vertical line) would it make
                // sense to use this.
                LineSlices,
                // Copy slices into a contiguous buffer. We need more
                // memory for this.
                // - 156 ms - full screen update
                // -  32 ms - small text move down
                // -  48 ms - text moved up a few lines
                // Updating the entire screen is slower, as expected,
                // but otherwise it can be faster.
                UseExtraBuffer,
            }

            // Select Contiguous for full width updates and use the
            // extra buffer for other cases, now that we have sufficient
            // RAM.
            // (The unused LineSlices method above is kept for reference only.)
            let draw_method: DrawMethod;
            if self.dirty_area.width() == WIDTH {
                draw_method = DrawMethod::Contiguous;
            } else {
                draw_method = DrawMethod::UseExtraBuffer;
            }

            let x0: u16;
            let y0: u16 = self.dirty_area.y0();
            let w: u16;
            let h: u16 = self.dirty_area.height();

            match draw_method {
                DrawMethod::Contiguous => {
                    x0 = 0;
                    w = WIDTH;
                },
                DrawMethod::LineSlices | DrawMethod::UseExtraBuffer => {
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
                },
                DrawMethod::UseExtraBuffer => {
                    if self.partial == None {
                        log::info!("framebuffer: alloced a second framebuffer");
                        self.partial = Some(allocate_box::<u16, WIDTH, HEIGHT>());
                    }
                    let partial = self.partial.as_mut().unwrap();
                    let mut dest: usize = 0;
                    for y in y0..y0 + h {
                        let start = Self::index(x0, y);
                        let slice = &self.current[start..start + (w as usize)];
                        partial[dest..dest + (w as usize)].copy_from_slice(slice);
                        dest += w as usize;
                    }
                    let slice = &partial[0..(h as usize * w as usize)];
                    display.draw_raw_slice(x0, y0, x0 + w - 1, y0 + h - 1, &slice)?;
                },
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
