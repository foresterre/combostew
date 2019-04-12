use crate::operations::Operation;
use image::DynamicImage;
use image::GenericImageView;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Formatter};

/// This version of the operations module will use an AST like structure.
/// Instead of evaluating a program, we apply 'a language' on an image.
///
/// Checklist:
/// - TODO: replace transformations with v2/'engine'
/// - think about naming, especially of the module

trait EnvironmentKey {
    fn key(&self) -> &'static str;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentOption {
    ResizeSamplingFilter(FilterTypeWrap),
}

impl EnvironmentOption {
    pub fn resize_sampling_filter(&self) -> Option<FilterTypeWrap> {
        match self {
            EnvironmentOption::ResizeSamplingFilter(k) => Some(k.clone()),
            // _ => None, // not needed right now, but will be needed when adding other options.
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentFlag {
    // later
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnvironmentItem {
    Opt(EnvironmentOption),
    Flag(EnvironmentFlag),
}

impl EnvironmentItem {
    pub fn opt(&self) -> Option<EnvironmentOption> {
        match &self {
            EnvironmentItem::Opt(k) => Some(k.clone()),
            EnvironmentItem::Flag(_) => None,
        }
    }

    pub fn flag(&self) -> Option<EnvironmentFlag> {
        match &self {
            EnvironmentItem::Flag(k) => Some(k.clone()),
            EnvironmentItem::Opt(_) => None,
        }
    }
}

impl EnvironmentKey for EnvironmentItem {
    fn key(&self) -> &'static str {
        match self {
            EnvironmentItem::Opt(EnvironmentOption::ResizeSamplingFilter(_)) => {
                "Resize_SamplingFilter"
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Environment {
    store: HashMap<&'static str, EnvironmentItem>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            store: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn insert_or_update(&mut self, item: EnvironmentItem) {
        let key = item.key();
        if self.store.contains_key(key) {
            *self.store.get_mut(key).unwrap() = item;
        } else {
            self.store.insert(key, item);
        }
    }

    pub fn get(&mut self, key: &'static str) -> Option<&EnvironmentItem> {
        self.store.get(key)
    }
}

pub enum Statement {
    Operation(Operation),
    RegisterEnvironmentItem(EnvironmentItem),
}

pub type Program = Vec<Statement>;

pub struct ImageEngine {
    environment: Box<Environment>,
    image: Box<DynamicImage>,
}

impl ImageEngine {
    pub fn new(image: DynamicImage) -> Self {
        Self {
            environment: Box::from(Environment::default()),
            image: Box::from(image),
        }
    }

    pub fn ignite(&mut self, statements: Program) -> Result<&DynamicImage, Box<dyn Error>> {
        for stmt in statements {
            match self.process_statement(stmt) {
                Ok(_) => continue,
                Err(err) => return Err(err),
            }
        }

        Ok(&self.image)
    }

    pub fn process_statement(&mut self, statement: Statement) -> Result<(), Box<dyn Error>> {
        match statement {
            Statement::Operation(op) => self.process_operation(op),
            Statement::RegisterEnvironmentItem(item) => self.process_register_env(item),
        }
    }

    pub fn process_operation(&mut self, operation: Operation) -> Result<(), Box<dyn Error>> {
        match operation {
            Operation::Blur(sigma) => {
                *self.image = self.image.blur(sigma);
                Ok(())
            }
            Operation::Brighten(amount) => {
                *self.image = self.image.brighten(amount);
                Ok(())
            }
            Operation::Contrast(c) => {
                *self.image = self.image.adjust_contrast(c);
                Ok(())
            }
            Operation::Crop(lx, ly, rx, ry) => {
                // 1. verify that the top left anchor is smaller than the bottom right anchor
                // 2. verify that the selection is within the bounds of the image
                Verify::crop_selection_box_can_exist(lx, ly, rx, ry)
                    .and_then(|_| {
                        Verify::crop_selection_within_image_bounds(&self.image, lx, ly, rx, ry)
                    })
                    .map(|_| {
                        *self.image = self.image.crop(lx, ly, rx - lx, ry - ly);
                    })
            }
            // We need to ensure here that Filter3x3's `it` (&[f32]) has length 9.
            // Otherwise it will panic, see: https://docs.rs/image/0.19.0/src/image/dynimage.rs.html#349
            // This check already happens within the `parse` module.
            Operation::Filter3x3(ref it) => {
                *self.image = self.image.filter3x3(it);
                Ok(())
            }
            Operation::FlipHorizontal => {
                *self.image = self.image.fliph();
                Ok(())
            }
            Operation::FlipVertical => {
                *self.image = self.image.flipv();
                Ok(())
            }
            Operation::GrayScale => {
                *self.image = self.image.grayscale();
                Ok(())
            }
            Operation::HueRotate(degree) => {
                *self.image = self.image.huerotate(degree);
                Ok(())
            }
            Operation::Invert => {
                self.image.invert();
                Ok(())
            }
            Operation::Resize(new_x, new_y) => {
                const DEFAULT_RESIZE_FILTER: image::FilterType = image::FilterType::Gaussian;

                let filter = self
                    .environment
                    .get("Resize_SamplingFilter")
                    .and_then(|item| {
                        item.opt().and_then(|opt| {
                            opt.resize_sampling_filter().map(|filter_wrap| {
                                println!("resize filter: {:?}", filter_wrap);
                                image::FilterType::from(filter_wrap)
                            })
                        })
                    })
                    .unwrap_or(DEFAULT_RESIZE_FILTER);

                *self.image = self.image.resize_exact(new_x, new_y, filter);
                Ok(())
            }
            Operation::Rotate90 => {
                *self.image = self.image.rotate90();
                Ok(())
            }
            Operation::Rotate180 => {
                *self.image = self.image.rotate180();
                Ok(())
            }
            Operation::Rotate270 => {
                *self.image = self.image.rotate270();
                Ok(())
            }
            Operation::Unsharpen(sigma, threshold) => {
                *self.image = self.image.unsharpen(sigma, threshold);
                Ok(())
            }
        }
    }

    pub fn process_register_env(&mut self, item: EnvironmentItem) -> Result<(), Box<dyn Error>> {
        self.environment.insert_or_update(item);

        // todo{}: remove
        println!("env: {:?}", self.environment.store);

        Ok(())
    }
}

struct Verify;

impl Verify {
    fn crop_selection_box_can_exist(
        lx: u32,
        ly: u32,
        rx: u32,
        ry: u32,
    ) -> Result<(), Box<dyn Error>> {
        if (rx <= lx) || (ry <= ly) {
            Err(format!(
                "Operation: crop -- Top selection coordinates are smaller than bottom selection coordinates. \
            Required top selection < bottom selection but given coordinates are: [top anchor: (x={}, y={}), bottom anchor: (x={}, y={})].",
                lx, ly, rx, ry
            ).into())
        } else {
            Ok(())
        }
    }

    fn crop_selection_within_image_bounds(
        image: &DynamicImage,
        lx: u32,
        ly: u32,
        rx: u32,
        ry: u32,
    ) -> Result<(), Box<dyn Error>> {
        let (dim_x, dim_y) = image.dimensions();

        match (lx <= dim_x, ly <= dim_y, rx <= dim_x, ry <= dim_y) {
            (true, true, true, true) => Ok(()),
            _ => {
                println!("error expected");
                Err(format!("Operation: crop -- Top or bottom selection coordinates out of bounds: selection is [top anchor: \
                (x={}, y={}), bottom anchor: (x={}, y={})] but max selection range is: (x={}, y={}).", lx, ly, rx, ry, dim_x, dim_y).into())
            }
        }
    }
}

// Wrapper for image::FilterType.
// Does only exists, because image::FilterType does not implement PartialEq and Debug.
pub enum FilterTypeWrap {
    Inner(image::FilterType),
}

impl PartialEq<FilterTypeWrap> for FilterTypeWrap {
    fn eq(&self, other: &FilterTypeWrap) -> bool {
        match (self, other) {
            (
                FilterTypeWrap::Inner(image::FilterType::CatmullRom),
                FilterTypeWrap::Inner(image::FilterType::CatmullRom),
            ) => true,
            (
                FilterTypeWrap::Inner(image::FilterType::Gaussian),
                FilterTypeWrap::Inner(image::FilterType::Gaussian),
            ) => true,
            (
                FilterTypeWrap::Inner(image::FilterType::Lanczos3),
                FilterTypeWrap::Inner(image::FilterType::Lanczos3),
            ) => true,
            (
                FilterTypeWrap::Inner(image::FilterType::Nearest),
                FilterTypeWrap::Inner(image::FilterType::Nearest),
            ) => true,
            (
                FilterTypeWrap::Inner(image::FilterType::Triangle),
                FilterTypeWrap::Inner(image::FilterType::Triangle),
            ) => true,
            _ => false,
        }
    }
}

impl Clone for FilterTypeWrap {
    fn clone(&self) -> Self {
        match self {
            FilterTypeWrap::Inner(a) => FilterTypeWrap::Inner(*a),
        }
    }
}

impl Eq for FilterTypeWrap {}

impl Debug for FilterTypeWrap {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let msg = match self {
            FilterTypeWrap::Inner(image::FilterType::CatmullRom) => {
                "image::FilterType::CatmullRom (Wrapper)"
            }
            FilterTypeWrap::Inner(image::FilterType::Gaussian) => {
                "image::FilterType::Gaussian (Wrapper)"
            }
            FilterTypeWrap::Inner(image::FilterType::Lanczos3) => {
                "image::FilterType::Lanczos3 (Wrapper)"
            }
            FilterTypeWrap::Inner(image::FilterType::Nearest) => {
                "image::FilterType::Nearest (Wrapper)"
            }
            FilterTypeWrap::Inner(image::FilterType::Triangle) => {
                "image::FilterType::Triangle (Wrapper)"
            }
        };

        f.write_str(msg)
    }
}

impl From<FilterTypeWrap> for image::FilterType {
    fn from(wrap: FilterTypeWrap) -> Self {
        match wrap {
            FilterTypeWrap::Inner(w) => w,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::mod_test_includes::*;
    use image::GenericImageView;

    #[test]
    fn resize_with_sampling_filter_nearest() {
        let img: DynamicImage = setup_default_test_image();

        let mut engine = ImageEngine::new(img);
        let done = engine.ignite(vec![
            Statement::Operation(Operation::Invert),
            Statement::Operation(Operation::Brighten(10)),
            Statement::Operation(Operation::HueRotate(170)),
            Statement::RegisterEnvironmentItem(EnvironmentItem::Opt(
                EnvironmentOption::ResizeSamplingFilter(FilterTypeWrap::Inner(
                    image::FilterType::Triangle,
                )),
            )),
            Statement::RegisterEnvironmentItem(EnvironmentItem::Opt(
                EnvironmentOption::ResizeSamplingFilter(FilterTypeWrap::Inner(
                    image::FilterType::Nearest,
                )),
            )),
            Statement::Operation(Operation::Resize(100, 100)),
        ]);

        assert!(done.is_ok());

        output_test_image_for_manual_inspection(
            &done.unwrap(),
            "target/test_resize_sampling_filter_nearest.png",
        )
    }

    #[test]
    fn test_blur() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Blur(10.0);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        output_test_image_for_manual_inspection(&done.unwrap(), "target/test_blur.png")
    }

    #[test]
    fn test_brighten_pos() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Brighten(25);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_brighten_pos_25.png")
    }

    #[test]
    fn test_brighten_zero() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();
        let operation = Operation::Brighten(0);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_eq!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_brighten_zero.png")
    }

    #[test]
    fn test_brighten_neg() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Brighten(-25);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_brighten_neg_25.png")
    }

    #[test]
    fn test_contrast_pos() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Contrast(150.9);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_contrast_pos_15_9.png")
    }

    #[test]
    fn test_contrast_neg() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Contrast(-150.9);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_contrast_pos_15_9.png")
    }

    #[test]
    fn test_crop_ok_no_change() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");
        let cmp: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 2, 2);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_eq!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_crop_no_change.bmp")
    }

    #[test]
    fn test_crop_ok_to_one_pixel() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");
        let cmp: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 1, 1);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        let result_dim = result_img.dimensions();
        assert_eq!(1, result_dim.0);
        assert_eq!(1, result_dim.1);

        assert_eq!(image::Rgba([0, 0, 0, 255]), result_img.get_pixel(0, 0));

        output_test_image_for_manual_inspection(&result_img, "target/test_crop_ok_to_one_pixel.bmp")
    }

    #[test]
    fn test_crop_ok_to_half_horizontal() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");
        let cmp: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 2, 1);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        let result_dim = result_img.dimensions();
        assert_eq!(2, result_dim.0);
        assert_eq!(1, result_dim.1);

        assert_eq!(image::Rgba([0, 0, 0, 255]), result_img.get_pixel(0, 0));
        assert_eq!(
            image::Rgba([255, 255, 255, 255]),
            result_img.get_pixel(1, 0)
        );

        output_test_image_for_manual_inspection(
            &result_img,
            "target/test_crop_ok_to_half_horizontal.bmp",
        )
    }

    #[test]
    fn test_crop_err_lx_larger_than_rx() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        // not rx >= lx
        let operation = Operation::Crop(1, 0, 0, 0);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_ly_larger_than_ry() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        // not rx >= lx
        let operation = Operation::Crop(0, 1, 0, 0);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_lx() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(3, 0, 1, 1);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_ly() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 3, 1, 1);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_rx() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 3, 1);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_ry() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 1, 3);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_err());
    }

    #[test]
    fn test_filter3x3() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Filter3x3([1.0, 0.5, 0.0, 1.0, 0.5, 0.0, 1.0, 0.5, 0.0]);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_filter3x3.png")
    }

    #[test]
    fn test_flip_h() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::FlipHorizontal;

        let (xa, ya) = img.dimensions();
        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, xb);
        assert_eq!(ya, yb);

        output_test_image_for_manual_inspection(&img_result, "target/test_fliph.png")
    }

    #[test]
    fn test_flip_v() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::FlipVertical;

        let (xa, ya) = img.dimensions();
        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, xb);
        assert_eq!(ya, yb);

        output_test_image_for_manual_inspection(&img_result, "target/test_flipv.png")
    }

    #[test]
    fn test_gray_scale() {
        use image::Pixel;

        let img: DynamicImage = setup_test_image("resources/rainbow_8x6.bmp");
        let operation = Operation::GrayScale;

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();

        // The color type isn't actually changed to luma, so instead of checking color type,
        // here pixels are checked to have equal (r, g, b) components.
        for i in 0..8 {
            for j in 0..6 {
                let pixel = img_result.get_pixel(i, j);
                let channels_result = pixel.channels();
                let r_component = channels_result[0];
                let g_component = channels_result[1];
                let b_component = channels_result[2];

                assert_eq!(r_component, g_component);
                assert_eq!(g_component, b_component);
            }
        }

        output_test_image_for_manual_inspection(&img_result, "target/test_gray_scale.png")
    }

    #[test]
    fn test_hue_rotate_neg() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::HueRotate(-100);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_hue_rot_neg_100.png")
    }

    #[test]
    fn test_hue_rotate_pos() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::HueRotate(100);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_hue_rot_pos_100.png")
    }

    #[test]
    fn test_hue_rotate_zero() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::HueRotate(0);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_eq!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_hue_rot_0.png")
    }

    #[test]
    fn test_hue_rotate_360() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::HueRotate(360);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        // https://docs.rs/image/0.19.0/image/enum.DynamicImage.html#method.huerotate
        // huerotate(0) should be huerotate(360), but this doesn't seem the case
        assert_eq!(cmp.huerotate(360).raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_hue_rot_pos_360.png")
    }

    #[test]
    fn test_hue_rotate_over_rotate_pos() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::HueRotate(460);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.huerotate(100).raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_hue_rot_pos_460.png")
    }

    #[test]
    fn test_invert() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Invert;

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_invert.png")
    }

    #[test]
    fn test_resize_down_gaussian() {
        // 217x447px => 100x200
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Resize(100, 200);

        let (xa, ya) = img.dimensions();

        assert_eq!(xa, 217);
        assert_eq!(ya, 447);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xb, 100);
        assert_eq!(yb, 200);

        output_test_image_for_manual_inspection(&img_result, "target/test_scale_100x200.png")
    }

    #[test]
    fn test_resize_up_gaussian() {
        // 217x447px => 300x500
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Resize(250, 500);

        let (xa, ya) = img.dimensions();

        assert_eq!(xa, 217);
        assert_eq!(ya, 447);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xb, 250);
        assert_eq!(yb, 500);

        output_test_image_for_manual_inspection(&img_result, "target/test_scale_250x500.png")
    }

    #[test]
    fn test_rotate90() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Rotate90;

        let (xa, ya) = img.dimensions();
        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, yb);
        assert_eq!(xb, ya);

        output_test_image_for_manual_inspection(&img_result, "target/test_rotate90.png")
    }

    #[test]
    fn test_rotate180() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Rotate180;

        let (xa, ya) = img.dimensions();
        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, xb);
        assert_eq!(ya, yb);

        output_test_image_for_manual_inspection(&img_result, "target/test_rotate180.png")
    }

    #[test]
    fn test_rotate270() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Rotate270;

        let (xa, ya) = img.dimensions();
        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xa, yb);
        assert_eq!(xb, ya);

        output_test_image_for_manual_inspection(&img_result, "target/test_rotate270.png")
    }

    #[test]
    fn test_unsharpen_pos() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Unsharpen(20.1, 20);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);
        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(&result_img, "target/test_unsharpen_20_1_20.png")
    }

    #[test]
    fn test_unsharpen_neg() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Unsharpen(-20.1, -20);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(vec![Statement::Operation(operation)]);
        assert!(done.is_ok());

        let result_img = done.unwrap();

        assert_ne!(cmp.raw_pixels(), result_img.raw_pixels());

        output_test_image_for_manual_inspection(
            &result_img,
            "target/test_unsharpen_neg20_1_neg20.png",
        )
    }

    #[test]
    fn test_multi() {
        // 217x447px original
        let img: DynamicImage = setup_default_test_image();
        let operations = vec![
            Statement::Operation(Operation::Resize(80, 100)),
            Statement::Operation(Operation::Blur(5.0)),
            Statement::Operation(Operation::FlipHorizontal),
            Statement::Operation(Operation::FlipVertical),
            Statement::Operation(Operation::Rotate90),
        ];
        let (xa, ya) = img.dimensions();

        assert_eq!(ya, 447);
        assert_eq!(xa, 217);

        let mut operator = ImageEngine::new(img);
        let done = operator.ignite(operations);

        assert!(done.is_ok());

        let done_image = done.unwrap();
        let (xb, yb) = done_image.dimensions();

        // dim original => 80x100 => 100x80
        assert_eq!(xb, 100);
        assert_eq!(yb, 80);

        output_test_image_for_manual_inspection(&done_image, "target/test_multi.png")
    }
}
