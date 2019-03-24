use crate::config::Config;
use crate::operations::transformations::apply_operations_on_image;
use crate::operations::Operation;
use crate::processor::ProcessMutWithConfig;

pub struct ImageOperationsProcessor<'a> {
    buffer: &'a mut image::DynamicImage,
    operations: &'a [Operation],
}

impl<'a> ImageOperationsProcessor<'a> {
    pub fn new(
        buffer: &'a mut image::DynamicImage,
        operations: &'a [Operation],
    ) -> ImageOperationsProcessor<'a> {
        ImageOperationsProcessor { buffer, operations }
    }

    fn apply_operations(&mut self) -> Result<(), String> {
        apply_operations_on_image(&mut self.buffer, self.operations)
    }
}

impl<'a> ProcessMutWithConfig<Result<(), String>> for ImageOperationsProcessor<'a> {
    fn process_mut(&mut self, _config: &Config) -> Result<(), String> {
        self.apply_operations()
    }
}
