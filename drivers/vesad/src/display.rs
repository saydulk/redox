extern crate rusttype;

use std::cmp;

use primitive::{fast_set32, fast_set64, fast_copy64};

use self::rusttype::{Font, FontCollection, Scale, point};

static FONT: &'static [u8] = include_bytes!("../../../res/fonts/DejaVuSansMono.ttf");
static FONT_BOLD: &'static [u8] = include_bytes!("../../../res/fonts/DejaVuSansMono-Bold.ttf");
static FONT_BOLD_ITALIC: &'static [u8] = include_bytes!("../../../res/fonts/DejaVuSansMono-BoldOblique.ttf");
static FONT_ITALIC: &'static [u8] = include_bytes!("../../../res/fonts/DejaVuSansMono-Oblique.ttf");

/// A display
pub struct Display {
    pub width: usize,
    pub height: usize,
    pub onscreen: &'static mut [u32],
    pub offscreen: &'static mut [u32],
    pub font: Font<'static>,
    pub font_bold: Font<'static>,
    pub font_bold_italic: Font<'static>,
    pub font_italic: Font<'static>
}

impl Display {
    pub fn new(width: usize, height: usize, onscreen: &'static mut [u32], offscreen: &'static mut [u32]) -> Display {
        Display {
            width: width,
            height: height,
            onscreen: onscreen,
            offscreen: offscreen,
            font: FontCollection::from_bytes(FONT).into_font().unwrap(),
            font_bold: FontCollection::from_bytes(FONT_BOLD).into_font().unwrap(),
            font_bold_italic: FontCollection::from_bytes(FONT_BOLD_ITALIC).into_font().unwrap(),
            font_italic: FontCollection::from_bytes(FONT_ITALIC).into_font().unwrap()
        }
    }

    /// Draw a rectangle
    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize, color: u32) {
        let start_y = cmp::min(self.height - 1, y);
        let end_y = cmp::min(self.height, y + h);

        let start_x = cmp::min(self.width - 1, x);
        let len = cmp::min(self.width, x + w) - start_x;

        let mut offscreen_ptr = self.offscreen.as_mut_ptr() as usize;
        let mut onscreen_ptr = self.onscreen.as_mut_ptr() as usize;

        let stride = self.width * 4;

        let offset = y * stride + start_x * 4;
        offscreen_ptr += offset;
        onscreen_ptr += offset;

        let mut rows = end_y - start_y;
        while rows > 0 {
            unsafe {
                fast_set32(offscreen_ptr as *mut u32, color, len);
                fast_set32(onscreen_ptr as *mut u32, color, len);
            }
            offscreen_ptr += stride;
            onscreen_ptr += stride;
            rows -= 1;
        }
    }

    /// Draw a character
    pub fn char(&mut self, x: usize, y: usize, character: char, color: u32, bold: bool, italic: bool) {
        let width = self.width;
        let height = self.height;
        let offscreen = self.offscreen.as_mut_ptr();
        let onscreen = self.onscreen.as_mut_ptr();

        let font = if bold && italic {
            &self.font_bold_italic
        } else if bold {
            &self.font_bold
        } else if italic {
            &self.font_italic
        } else {
            &self.font
        };

        if let Some(glyph) = font.glyph(character){
            let scale = Scale::uniform(16.0);
            let v_metrics = font.v_metrics(scale);
            let point = point(0.0, v_metrics.ascent);
            glyph.scaled(scale).positioned(point).draw(|off_x, off_y, v| {
                let off_x = x + off_x as usize;
                let off_y = y + off_y as usize;
                // There's still a possibility that the glyph clips the boundaries of the bitmap
                if off_x < width && off_y < height {
                    let v_u = (v * 255.0) as u32;
                    let r = ((color >> 16) & 0xFF * v_u)/255;
                    let g = ((color >> 8) & 0xFF * v_u)/255;
                    let b = (color & 0xFF * v_u)/255;
                    let c = (r << 16) | (g << 8) | b;

                    let index = (off_y * width + off_x) as isize;
                    unsafe { *offscreen.offset(index) = c; }
                    unsafe { *onscreen.offset(index) = c; }
                }
            });
        }
    }

    /// Scroll display
    pub fn scroll(&mut self, rows: usize, color: u32) {
        let data = (color as u64) << 32 | color as u64;

        let width = self.width/2;
        let height = self.height;
        if rows > 0 && rows < height {
            let off1 = rows * width;
            let off2 = height * width - off1;
            unsafe {
                let data_ptr = self.offscreen.as_mut_ptr() as *mut u64;
                fast_copy64(data_ptr, data_ptr.offset(off1 as isize), off2);
                fast_set64(data_ptr.offset(off2 as isize), data, off1);

                fast_copy64(self.onscreen.as_mut_ptr() as *mut u64, data_ptr, off1 + off2);
            }
        }
    }
}