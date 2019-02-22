use image::{DynamicImage, FilterType, GenericImageView};

use super::Operation;

pub trait ApplyOperation<O, T, E> {
    fn apply_operation(&self, operation: &O) -> Result<T, E>;
}

// TODO take &mut DynImage as param?
impl ApplyOperation<Operation, DynamicImage, String> for DynamicImage {
    fn apply_operation(&self, operation: &Operation) -> Result<DynamicImage, String> {
        match *operation {
            Operation::Blur(sigma) => Ok(self.blur(sigma)),
            Operation::Brighten(amount) => Ok(self.brighten(amount)),
            Operation::Contrast(c) => Ok(self.adjust_contrast(c)),
            Operation::Crop(lx, ly, rx, ry) => {
                // 1. verify that the top left anchor is smaller than the bottom right anchor
                // 2. verify that the selection is within the bounds of the image
                verify_crop_selection(lx, ly, rx, ry)
                    .and_then(|_| verify_crop_selection_within_image_bounds(&self, lx, ly, rx, ry))
                    .map(|_| {
                        let cropped = {
                            let mut buffer = self.clone();
                            buffer.crop(lx, ly, rx - lx, ry - ly)
                        };

                        cropped
                    })
            }
            // We need to ensure here that Filter3x3's `it` (&[f32]) has length 9.
            // Otherwise it will panic, see: https://docs.rs/image/0.19.0/src/image/dynimage.rs.html#349
            // This check already happens within the `parse` module.
            Operation::Filter3x3(ref it) => Ok(self.filter3x3(&it)),
            Operation::FlipHorizontal => Ok(self.fliph()),
            Operation::FlipVertical => Ok(self.flipv()),
            Operation::GrayScale => Ok(self.grayscale()),
            Operation::HueRotate(degree) => Ok(self.huerotate(degree)),
            // TODO this is rather sub optimal with the double clone
            Operation::Invert => {
                let inverted = {
                    let mut buffer = self.clone();
                    buffer.invert();
                    buffer
                };

                Ok(inverted)
            }
            Operation::Resize(new_x, new_y) => {
                Ok(self.resize_exact(new_x, new_y, FilterType::Gaussian))
            }
            Operation::Rotate90 => Ok(self.rotate90()),
            Operation::Rotate270 => Ok(self.rotate270()),
            Operation::Rotate180 => Ok(self.rotate180()),
            Operation::Unsharpen(sigma, threshold) => Ok(self.unsharpen(sigma, threshold)),
        }
    }
}

fn verify_crop_selection(lx: u32, ly: u32, rx: u32, ry: u32) -> Result<(), String> {
    if (rx <= lx) || (ry <= ly) {
        Err(format!(
            "Operation: crop -- Top selection coordinates are smaller than bottom selection coordinates. \
            Required top selection < bottom selection but given coordinates are: [top anchor: (x={}, y={}), bottom anchor: (x={}, y={})].",
            lx, ly, rx, ry
        ))
    } else {
        Ok(())
    }
}

fn verify_crop_selection_within_image_bounds(
    image: &DynamicImage,
    lx: u32,
    ly: u32,
    rx: u32,
    ry: u32,
) -> Result<(), String> {
    let (dim_x, dim_y) = image.dimensions();

    match (lx <= dim_x, ly <= dim_y, rx <= dim_x, ry <= dim_y) {
        (true, true, true, true) => Ok(()),
        _ => {
            println!("error expected");
            Err(format!("Operation: crop -- Top or bottom selection coordinates out of bounds: selection is [top anchor: \
                (x={}, y={}), bottom anchor: (x={}, y={})] but max selection range is: (x={}, y={}).", lx, ly, rx, ry, dim_x, dim_y))
        }
    }
}

pub fn apply_operations_on_image(
    image: &mut DynamicImage,
    operations: &[Operation],
) -> Result<(), String> {
    // this should be possible clean and nice and functional, but right now, I can't come up with it.
    for op in operations.iter() {
        *image = image.apply_operation(op)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;
    use image::GenericImageView;

    use crate::operations::mod_test_includes::*;

    use super::*;

    #[test]
    fn test_blur() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Blur(25.0);

        let done = img.apply_operation(&operation);
        assert!(done.is_ok());

        output_test_image_for_manual_inspection(&done.unwrap(), "target/test_blur.png")
    }

    #[test]
    fn test_brighten_pos() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Brighten(25);

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_ly_larger_than_ry() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        // not rx >= lx
        let operation = Operation::Crop(0, 1, 0, 0);

        let done = img.apply_operation(&operation);
        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_lx() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(3, 0, 1, 1);

        let done = img.apply_operation(&operation);
        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_ly() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 3, 1, 1);

        let done = img.apply_operation(&operation);
        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_rx() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 3, 1);

        let done = img.apply_operation(&operation);
        assert!(done.is_err());
    }

    #[test]
    fn test_crop_err_out_of_image_bounds_top_ry() {
        let img: DynamicImage = setup_test_image("resources/blackwhite_2x2.bmp");

        let operation = Operation::Crop(0, 0, 1, 3);

        let done = img.apply_operation(&operation);
        assert!(done.is_err());
    }

    #[test]
    fn test_filter3x3() {
        let img: DynamicImage = setup_default_test_image();
        let cmp: DynamicImage = setup_default_test_image();

        let operation = Operation::Filter3x3(ArrayVec::from([
            1.0, 0.5, 0.0, 1.0, 0.5, 0.0, 1.0, 0.5, 0.0,
        ]));

        let done = img.apply_operation(&operation);
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
        let done = img.apply_operation(&operation);

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
        let done = img.apply_operation(&operation);

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

        let done = img.apply_operation(&operation);

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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);

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
        let operation = Operation::Resize(300, 500);

        let (xa, ya) = img.dimensions();

        assert_eq!(xa, 217);
        assert_eq!(ya, 447);

        let done = img.apply_operation(&operation);

        assert!(done.is_ok());

        let img_result = done.unwrap();
        let (xb, yb) = img_result.dimensions();

        assert_eq!(xb, 300);
        assert_eq!(yb, 500);

        output_test_image_for_manual_inspection(&img_result, "target/test_scale_400x500.png")
    }

    #[test]
    fn test_rotate90() {
        let img: DynamicImage = setup_default_test_image();
        let operation = Operation::Rotate90;

        let (xa, ya) = img.dimensions();
        let done = img.apply_operation(&operation);

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
        let done = img.apply_operation(&operation);

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
        let done = img.apply_operation(&operation);

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

        let done = img.apply_operation(&operation);
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

        let done = img.apply_operation(&operation);
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
        let mut img: DynamicImage = setup_default_test_image();
        let operations = vec![
            Operation::Resize(80, 100),
            Operation::Blur(5.0),
            Operation::FlipHorizontal,
            Operation::FlipVertical,
            Operation::Rotate90,
        ];

        let (xa, ya) = img.dimensions();

        assert_eq!(ya, 447);
        assert_eq!(xa, 217);

        let done = apply_operations_on_image(&mut img, &operations);

        assert!(done.is_ok());

        let (xb, yb) = img.dimensions();

        // dim original => 80x100 => 100x80
        assert_eq!(xb, 100);
        assert_eq!(yb, 80);

        output_test_image_for_manual_inspection(&img, "target/test_multi.png")
    }

}
